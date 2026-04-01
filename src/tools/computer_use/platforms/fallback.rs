//! Fallback platform implementation using pure input simulation.
//!
//! This implementation provides basic computer use capabilities without
//! accessibility APIs. It's used when platform-specific features are disabled.

use async_trait::async_trait;
use enigo::{Enigo, Key, KeyboardControllable, MouseButton as EnigoButton, MouseControllable};

use super::{ComputerPlatform, Screenshot};
use crate::tools::computer_use::{MouseButton, ScrollDirection};

pub struct FallbackPlatform {
    enigo: Enigo,
}

impl FallbackPlatform {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            enigo: Enigo::new(),
        })
    }

    fn convert_button(&self, button: MouseButton) -> EnigoButton {
        match button {
            MouseButton::Left => EnigoButton::Left,
            MouseButton::Right => EnigoButton::Right,
            MouseButton::Middle => EnigoButton::Middle,
        }
    }

    fn parse_keys(&self, keys: &str) -> Vec<Key> {
        // Simple key parser - extend as needed
        let mut result = Vec::new();
        for part in keys.split('+') {
            let key = match part.trim().to_lowercase().as_str() {
                "ctrl" | "control" => Key::Control,
                "alt" => Key::Alt,
                "shift" => Key::Shift,
                "super" | "meta" | "cmd" => Key::Meta,
                "return" | "enter" => Key::Return,
                "tab" => Key::Tab,
                "space" => Key::Space,
                "backspace" => Key::Backspace,
                "delete" => Key::Delete,
                "escape" | "esc" => Key::Escape,
                "up" => Key::UpArrow,
                "down" => Key::DownArrow,
                "left" => Key::LeftArrow,
                "right" => Key::RightArrow,
                "home" => Key::Home,
                "end" => Key::End,
                "pageup" => Key::PageUp,
                "pagedown" => Key::PageDown,
                other => Key::Layout(other.chars().next().unwrap_or(' ')),
            };
            result.push(key);
        }
        result
    }
}

#[async_trait]
impl ComputerPlatform for FallbackPlatform {
    async fn click(&self, x: i32, y: i32, button: MouseButton) -> anyhow::Result<()> {
        let mut enigo = Enigo::new();
        enigo.mouse_move_to(x, y);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        enigo.mouse_click(self.convert_button(button));
        Ok(())
    }

    async fn type_text(&self, text: &str) -> anyhow::Result<()> {
        let mut enigo = Enigo::new();
        enigo.key_sequence(text);
        Ok(())
    }

    async fn mouse_move(&self, x: i32, y: i32) -> anyhow::Result<()> {
        let mut enigo = Enigo::new();
        enigo.mouse_move_to(x, y);
        Ok(())
    }

    async fn send_keys(&self, keys: &str) -> anyhow::Result<()> {
        let mut enigo = Enigo::new();
        let parsed = self.parse_keys(keys);
        
        // Press all keys
        for key in &parsed {
            enigo.key_down(*key);
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Release in reverse order
        for key in parsed.iter().rev() {
            enigo.key_up(*key);
        }
        
        Ok(())
    }

    async fn scroll(&self, direction: ScrollDirection, amount: i32) -> anyhow::Result<()> {
        let mut enigo = Enigo::new();
        let clicks = (amount / 10).max(1); // Rough conversion
        
        match direction {
            ScrollDirection::Up => {
                for _ in 0..clicks {
                    enigo.mouse_scroll_y(1);
                }
            }
            ScrollDirection::Down => {
                for _ in 0..clicks {
                    enigo.mouse_scroll_y(-1);
                }
            }
            ScrollDirection::Left => {
                for _ in 0..clicks {
                    enigo.mouse_scroll_x(-1);
                }
            }
            ScrollDirection::Right => {
                for _ in 0..clicks {
                    enigo.mouse_scroll_x(1);
                }
            }
        }
        
        Ok(())
    }

    async fn screenshot_full(&self) -> anyhow::Result<Screenshot> {
        // Basic screenshot using platform-agnostic method
        self.take_screenshot(None).await
    }

    async fn screenshot_area(
        &self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Screenshot> {
        self.take_screenshot(Some((x, y, width, height))).await
    }

    async fn resolve_element_by_id(&self, _id: &str) -> anyhow::Result<(i32, i32)> {
        anyhow::bail!("Element resolution by ID not supported without accessibility features")
    }

    async fn resolve_element_by_path(&self, _path: &str) -> anyhow::Result<(i32, i32)> {
        anyhow::bail!("Element resolution by path not supported without accessibility features")
    }

    async fn inspect_element(&self, x: i32, y: i32) -> anyhow::Result<String> {
        Ok(format!(
            "Fallback mode: element inspection at ({}, {}) requires accessibility features",
            x, y
        ))
    }

    async fn inspect_full(&self) -> anyhow::Result<String> {
        Ok("Fallback mode: full inspection requires accessibility features".to_string())
    }

    async fn capture_state(&self) -> anyhow::Result<String> {
        Ok("State capture requires accessibility features (fallback mode)".to_string())
    }
}

impl FallbackPlatform {
    async fn take_screenshot(
        &self,
        area: Option<(i32, i32, u32, u32)>,
    ) -> anyhow::Result<Screenshot> {
        use image::ImageEncoder;
        
        // For now, return a placeholder
        // In production, would use platform-specific APIs or scrap crate
        let (width, height) = if let Some((_, _, w, h)) = area {
            (w, h)
        } else {
            (1920, 1080) // Default resolution
        };

        // Create a minimal placeholder image
        let img = image::RgbaImage::new(width, height);
        
        let mut png_data = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
        encoder
            .write_image(
                img.as_raw(),
                width,
                height,
                image::ColorType::Rgba8.into(),
            )
            .map_err(|e| anyhow::anyhow!("Failed to encode PNG: {}", e))?;

        let base64_data = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &png_data,
        );

        Ok(Screenshot {
            width,
            height,
            base64_data,
        })
    }
}
