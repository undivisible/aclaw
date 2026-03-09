use async_trait::async_trait;
use super::traits::*;
use serde_json::json;
use reqwest::{Client, Method};
use anyhow::{anyhow, Context};
use std::time::Duration;
use std::net::IpAddr;

pub struct HttpRequestTool {
    client: Client,
    allowed_domains: Vec<String>,
}

impl HttpRequestTool {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            allowed_domains,
        }
    }

    fn is_private_ip(&self, ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                // 127.x.x.x (loopback)
                octets[0] == 127 ||
                // 10.x.x.x (private)
                octets[0] == 10 ||
                // 172.16.x.x - 172.31.x.x (private)
                (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31) ||
                // 192.168.x.x (private)
                (octets[0] == 192 && octets[1] == 168) ||
                // 169.254.x.x (link-local)
                (octets[0] == 169 && octets[1] == 254)
            },
            IpAddr::V6(ipv6) => {
                // ::1 (loopback) or fc00::/7 (unique local)
                ipv6.is_loopback() || ((ipv6.segments()[0] & 0xfe00) == 0xfc00)
            }
        }
    }

    fn validate_url(&self, url: &str) -> anyhow::Result<()> {
        let parsed = reqwest::Url::parse(url)
            .context("Invalid URL")?;

        // Check if localhost
        if let Some(host) = parsed.host_str() {
            if host == "localhost" || host == "0.0.0.0" {
                return Err(anyhow!("Requests to localhost are blocked"));
            }

            // Check if domain is allowed (if allowed_domains is not empty)
            if !self.allowed_domains.is_empty() {
                let allowed = self.allowed_domains.iter().any(|domain| {
                    host == domain || host.ends_with(&format!(".{}", domain))
                });
                if !allowed {
                    return Err(anyhow!("Domain {} is not in allowed list", host));
                }
            }

            // Try to resolve and check for private IPs
            if let Ok(ip) = host.parse::<IpAddr>() {
                if self.is_private_ip(ip) {
                    return Err(anyhow!("Requests to private IP addresses are blocked"));
                }
            }
        }

        Ok(())
    }

    fn redact_sensitive_headers(&self, headers: &reqwest::header::HeaderMap) -> serde_json::Value {
        let mut redacted = serde_json::Map::new();
        let sensitive_keys = ["authorization", "api-key", "apikey", "token", "secret", "password"];

        for (key, value) in headers.iter() {
            let key_lower = key.as_str().to_lowercase();
            let is_sensitive = sensitive_keys.iter().any(|s| key_lower.contains(s));
            
            if is_sensitive {
                redacted.insert(key.as_str().to_string(), json!("[REDACTED]"));
            } else {
                redacted.insert(
                    key.as_str().to_string(),
                    json!(value.to_str().unwrap_or("[BINARY]")),
                );
            }
        }

        json!(redacted)
    }
}

#[async_trait]
impl Tool for HttpRequestTool {
    fn name(&self) -> &str {
        "http_request"
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "http_request".to_string(),
            description: "Make HTTP requests. Supports GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS. Blocks private/local hosts. Max response size 1MB, timeout 30s.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "method": {
                        "type": "string",
                        "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"],
                        "description": "HTTP method"
                    },
                    "url": {
                        "type": "string",
                        "description": "URL to request"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Request headers",
                        "additionalProperties": {"type": "string"},
                        "default": {}
                    },
                    "body": {
                        "type": "string",
                        "description": "Request body (for POST, PUT, PATCH)",
                        "default": ""
                    }
                },
                "required": ["method", "url"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .context("Failed to parse arguments")?;

        let method_str = args["method"].as_str()
            .ok_or_else(|| anyhow!("Missing method"))?;
        let url = args["url"].as_str()
            .ok_or_else(|| anyhow!("Missing url"))?;

        // Validate URL
        if let Err(e) = self.validate_url(url) {
            return Ok(ToolResult::error(format!("URL validation failed: {}", e)));
        }

        let method = Method::from_bytes(method_str.as_bytes())
            .map_err(|_| anyhow!("Invalid HTTP method"))?;

        let mut request = self.client.request(method, url);

        // Add headers
        if let Some(headers) = args["headers"].as_object() {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    request = request.header(key, val_str);
                }
            }
        }

        // Add body
        if let Some(body) = args["body"].as_str() {
            if !body.is_empty() {
                request = request.body(body.to_string());
            }
        }

        // Execute request
        let response = match request.send().await {
            Ok(resp) => resp,
            Err(e) => return Ok(ToolResult::error(format!("Request failed: {}", e))),
        };

        let status = response.status().as_u16();
        let headers = self.redact_sensitive_headers(response.headers());

        // Read body with size limit (1MB)
        const MAX_SIZE: usize = 1024 * 1024;
        let body_bytes = match response.bytes().await {
            Ok(bytes) => {
                if bytes.len() > MAX_SIZE {
                    return Ok(ToolResult::error(format!(
                        "Response too large: {} bytes (max {})",
                        bytes.len(),
                        MAX_SIZE
                    )));
                }
                bytes
            },
            Err(e) => return Ok(ToolResult::error(format!("Failed to read response: {}", e))),
        };

        let body = String::from_utf8_lossy(&body_bytes).to_string();

        let result = json!({
            "status": status,
            "headers": headers,
            "body": body,
        });

        Ok(ToolResult::success(result.to_string()))
    }
}