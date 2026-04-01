//! Computer Use — native OS automation tool.
//!
//! Provides cross-platform computer interaction capabilities:
//! - Screen capture and visual inspection
//! - Mouse and keyboard control
//! - Accessibility tree navigation (when available)
//! - Self-correcting action execution
//!
//! Architecture:
//! - Primary: Agent-Computer Interface (ACI) via OS accessibility APIs
//! - Fallback: Pixel-based interaction using screenshots and coordinates
//!
//! Platform support:
//! - Linux: X11/Wayland via XCB and portals
//! - macOS: Accessibility API and CoreGraphics
//! - Windows: UI Automation and Win32

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::traits::{Tool, ToolResult, ToolSpec};

mod platforms;
use platforms::ComputerPlatform;

/// Computer Use tool implementation.
pub struct ComputerUseTool {
    platform: Box<dyn ComputerPlatform>,
    #[allow(dead_code)]
    workspace: Option<String>,
}

impl ComputerUseTool {
    pub fn new(workspace: Option<String>) -> anyhow::Result<Self> {
        let platform = platforms::create_platform()?;
        Ok(Self {
            platform,
            workspace,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum ComputerAction {
    Click {
        target: ClickTarget,
        #[serde(default)]
        button: MouseButton,
    },
    Type {
        text: String,
        #[serde(default)]
        target: Option<ClickTarget>,
    },
    Screenshot {
        #[serde(default)]
        area: Option<ScreenArea>,
    },
    Scroll {
        direction: ScrollDirection,
        #[serde(default = "default_scroll_amount")]
        amount: i32,
    },
    Key {
        keys: String,
    },
    MouseMove {
        x: i32,
        y: i32,
    },
    Inspect {
        #[serde(default)]
        target: Option<ClickTarget>,
    },
}

fn default_scroll_amount() -> i32 {
    100
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ClickTarget {
    Coordinates { x: i32, y: i32 },
    Element { id: String },
    Path { path: String },
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum MouseButton {
    #[default]
    Left,
    Right,
    Middle,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScreenArea {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[async_trait]
impl Tool for ComputerUseTool {
    fn name(&self) -> &str {
        "computer"
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.name().to_string(),
            description: "Control the computer through mouse, keyboard, and screen capture. \
                         Supports clicking elements, typing text, taking screenshots, scrolling, \
                         sending keyboard shortcuts, moving the mouse, and inspecting UI state."
                .to_string(),
            parameters: json!({
                "type": "object",
                "required": ["action"],
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["click", "type", "screenshot", "scroll", "key", "mouse_move", "inspect"],
                        "description": "The action to perform"
                    },
                    "target": {
                        "description": "Target element or coordinates (for click, type, inspect)",
                        "oneOf": [
                            {
                                "type": "object",
                                "required": ["x", "y"],
                                "properties": {
                                    "x": {"type": "integer", "description": "X coordinate"},
                                    "y": {"type": "integer", "description": "Y coordinate"}
                                }
                            },
                            {
                                "type": "object",
                                "required": ["id"],
                                "properties": {
                                    "id": {"type": "string", "description": "Element ID from accessibility tree"}
                                }
                            },
                            {
                                "type": "object",
                                "required": ["path"],
                                "properties": {
                                    "path": {"type": "string", "description": "Element path (e.g., 'window[0]/button[3]')"}
                                }
                            }
                        ]
                    },
                    "button": {
                        "type": "string",
                        "enum": ["left", "right", "middle"],
                        "description": "Mouse button for click action (default: left)"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to type (for type action)"
                    },
                    "area": {
                        "type": "object",
                        "properties": {
                            "x": {"type": "integer"},
                            "y": {"type": "integer"},
                            "width": {"type": "integer"},
                            "height": {"type": "integer"}
                        },
                        "description": "Screen area to capture (for screenshot action)"
                    },
                    "direction": {
                        "type": "string",
                        "enum": ["up", "down", "left", "right"],
                        "description": "Scroll direction"
                    },
                    "amount": {
                        "type": "integer",
                        "description": "Scroll amount in pixels (default: 100)"
                    },
                    "keys": {
                        "type": "string",
                        "description": "Keyboard shortcuts (e.g., 'Ctrl+C', 'Alt+Tab')"
                    },
                    "x": {
                        "type": "integer",
                        "description": "X coordinate for mouse_move"
                    },
                    "y": {
                        "type": "integer",
                        "description": "Y coordinate for mouse_move"
                    }
                }
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let action: ComputerAction = serde_json::from_str(arguments)
            .map_err(|e| anyhow::anyhow!("Invalid computer action: {}", e))?;

        match action {
            ComputerAction::Click { target, button } => {
                self.execute_click(target, button).await
            }
            ComputerAction::Type { text, target } => {
                self.execute_type(text, target).await
            }
            ComputerAction::Screenshot { area } => {
                self.execute_screenshot(area).await
            }
            ComputerAction::Scroll { direction, amount } => {
                self.execute_scroll(direction, amount).await
            }
            ComputerAction::Key { keys } => {
                self.execute_key(keys).await
            }
            ComputerAction::MouseMove { x, y } => {
                self.execute_mouse_move(x, y).await
            }
            ComputerAction::Inspect { target } => {
                self.execute_inspect(target).await
            }
        }
    }
}

impl ComputerUseTool {
    async fn execute_click(
        &self,
        target: ClickTarget,
        button: MouseButton,
    ) -> anyhow::Result<ToolResult> {
        let coords = self.resolve_target(target).await?;
        self.platform.click(coords.0, coords.1, button).await?;
        
        // Capture state after action for self-correction
        let state = self.platform.capture_state().await?;
        
        Ok(ToolResult::success(format!(
            "Clicked at ({}, {})\nState: {}",
            coords.0, coords.1, state
        )))
    }

    async fn execute_type(
        &self,
        text: String,
        target: Option<ClickTarget>,
    ) -> anyhow::Result<ToolResult> {
        // Click target first if specified
        if let Some(t) = target {
            let coords = self.resolve_target(t).await?;
            self.platform.click(coords.0, coords.1, MouseButton::Left).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        self.platform.type_text(&text).await?;
        
        let state = self.platform.capture_state().await?;
        
        Ok(ToolResult::success(format!(
            "Typed: {}\nState: {}",
            text, state
        )))
    }

    async fn execute_screenshot(
        &self,
        area: Option<ScreenArea>,
    ) -> anyhow::Result<ToolResult> {
        let screenshot = if let Some(a) = area {
            self.platform
                .screenshot_area(a.x, a.y, a.width, a.height)
                .await?
        } else {
            self.platform.screenshot_full().await?
        };

        Ok(ToolResult::success(format!(
            "Screenshot captured ({}x{})\nBase64: {}",
            screenshot.width, screenshot.height, screenshot.base64_data
        )))
    }

    async fn execute_scroll(
        &self,
        direction: ScrollDirection,
        amount: i32,
    ) -> anyhow::Result<ToolResult> {
        self.platform.scroll(direction, amount).await?;
        
        let state = self.platform.capture_state().await?;
        
        Ok(ToolResult::success(format!(
            "Scrolled {:?} by {}\nState: {}",
            direction, amount, state
        )))
    }

    async fn execute_key(
        &self,
        keys: String,
    ) -> anyhow::Result<ToolResult> {
        self.platform.send_keys(&keys).await?;
        
        let state = self.platform.capture_state().await?;
        
        Ok(ToolResult::success(format!(
            "Sent keys: {}\nState: {}",
            keys, state
        )))
    }

    async fn execute_mouse_move(
        &self,
        x: i32,
        y: i32,
    ) -> anyhow::Result<ToolResult> {
        self.platform.mouse_move(x, y).await?;
        
        Ok(ToolResult::success(format!(
            "Moved mouse to ({}, {})",
            x, y
        )))
    }

    async fn execute_inspect(
        &self,
        target: Option<ClickTarget>,
    ) -> anyhow::Result<ToolResult> {
        let info = if let Some(t) = target {
            let coords = self.resolve_target(t).await?;
            self.platform.inspect_element(coords.0, coords.1).await?
        } else {
            self.platform.inspect_full().await?
        };

        Ok(ToolResult::success(format!(
            "Inspection result:\n{}",
            info
        )))
    }

    async fn resolve_target(&self, target: ClickTarget) -> anyhow::Result<(i32, i32)> {
        match target {
            ClickTarget::Coordinates { x, y } => Ok((x, y)),
            ClickTarget::Element { id } => {
                self.platform.resolve_element_by_id(&id).await
            }
            ClickTarget::Path { path } => {
                self.platform.resolve_element_by_path(&path).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spec_generation() {
        let tool = ComputerUseTool::new(None).unwrap();
        let spec = tool.spec();
        assert_eq!(spec.name, "computer");
        assert!(spec.description.contains("Control the computer"));
    }
}
