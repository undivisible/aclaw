use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn name(&self) -> &str;
    fn dimensions(&self) -> usize;
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    
    async fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let mut results = self.embed(&[text]).await?;
        results.pop().context("No embedding returned")
    }
}

// Noop provider for keyword-only fallback
pub struct NoopEmbedding;

#[async_trait]
impl EmbeddingProvider for NoopEmbedding {
    fn name(&self) -> &str {
        "noop"
    }

    fn dimensions(&self) -> usize {
        0
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Ok(vec![vec![]; texts.len()])
    }
}

// OpenAI provider
pub struct OpenAiEmbedding {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    dimensions: usize,
}

impl OpenAiEmbedding {
    pub fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        let model = model.unwrap_or_else(|| "text-embedding-3-small".to_string());
        let dimensions = if model.contains("text-embedding-3-small") {
            1536
        } else if model.contains("text-embedding-3-large") {
            3072
        } else if model.contains("ada-002") {
            1536
        } else {
            1536 // default
        };

        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com".to_string()),
            model,
            dimensions,
        }
    }
}

#[derive(Serialize)]
struct OpenAiEmbeddingRequest {
    input: Vec<String>,
    model: String,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for OpenAiEmbedding {
    fn name(&self) -> &str {
        &self.model
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let url = format!("{}/v1/embeddings", self.base_url);
        
        let request = OpenAiEmbeddingRequest {
            input: texts.iter().map(|s| s.to_string()).collect(),
            model: self.model.clone(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send embedding request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error {}: {}", status, body);
        }

        let response: OpenAiEmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse embedding response")?;

        Ok(response.data.into_iter().map(|d| d.embedding).collect())
    }
}

// Factory function
pub fn create_embedding_provider(
    provider_type: &str,
    api_key: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
) -> Result<Arc<dyn EmbeddingProvider>> {
    match provider_type.to_lowercase().as_str() {
        "noop" | "none" | "keyword" => Ok(Arc::new(NoopEmbedding)),
        "openai" => {
            let api_key = api_key.context("OpenAI API key required")?;
            Ok(Arc::new(OpenAiEmbedding::new(api_key, model, base_url)))
        }
        _ => anyhow::bail!("Unknown embedding provider: {}", provider_type),
    }
}
