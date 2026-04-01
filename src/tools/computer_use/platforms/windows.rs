//! Windows platform implementation using UI Automation and Win32.
//!
//! Provides accessibility tree access and screen capture on Windows.

use async_trait::async_trait;
use enigo::{Enigo, Key, KeyboardControllable, MouseButton as EnigoButton, MouseControllable};

use super::{ComputerPlatform, Screenshot};
use crate::tools::computer_use::{MouseButton, ScrollDirection};

pub struct WindowsPlatform {
    enigo: Enigo,
}

impl WindowsPlatform {
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
        let mut result = Vec::new();
        for part in keys.split('+') {
            let key = match part.trim().to_lowercase().as_str() {
                "ctrl" | "control" => Key::Control,
                "alt" => Key::Alt,
                "shift" => Key::Shift,
                "win" | "windows" | "meta" | "super" => Key::Meta,
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
impl ComputerPlatform for WindowsPlatform {
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
        
        for key in &parsed {
            enigo.key_down(*key);
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        for key in parsed.iter().rev() {
            enigo.key_up(*key);
        }
        
        Ok(())
    }

    async fn scroll(&self, direction: ScrollDirection, amount: i32) -> anyhow::Result<()> {
        let mut enigo = Enigo::new();
        let clicks = (amount / 10).max(1);
        
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
        // TODO: Implement using Win32 GDI
        self.placeholder_screenshot(1920, 1080).await
    }

    async fn screenshot_area(
        &self,
        _x: i32,
        _y: i32,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Screenshot> {
        // TODO: Implement area capture
        self.placeholder_screenshot(width, height).await
    }

    async fn resolve_element_by_id(&self, _id: &str) -> anyhow::Result<(i32, i32)> {
        // TODO: Implement UI Automation element lookup
        anyhow::bail!("Windows UI Automation element resolution not yet implemented")
    }

    async fn resolve_element_by_path(&self, _path: &str) -> anyhow::Result<(i32, i32)> {
        // TODO: Implement UI Automation path traversal
        anyhow::bail!("Windows UI Automation path resolution not yet implemented")
    }

    async fn inspect_element(&self, x: i32, y: i32) -> anyhow::Result<String> {
        // TODO: Query UI Automation at coordinates
        Ok(format!(
            "Windows: Element inspection at ({}, {}) - UI Automation implementation pending",
            x, y
        ))
    }

    async fn inspect_full(&self) -> anyhow::Result<String> {
        // TODO: Dump full UI Automation tree
        Ok("Windows: Full UI Automation tree dump - implementation pending".to_string())
    }

    async fn capture_state(&self) -> anyhow::Result<String> {
        Ok("Windows: Window state capture pending".to_string())
    }
}

impl WindowsPlatform {
    async fn placeholder_screenshot(&self, width: u32, height: u32) -> anyhow::Result<Screenshot> {
        use image::ImageEncoder;
        
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
