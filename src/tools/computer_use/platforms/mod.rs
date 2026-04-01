//! Platform abstraction for computer use operations.
//!
//! This module defines the ComputerPlatform trait that all platform-specific
//! implementations must fulfill. It provides a unified interface for:
//! - Accessibility tree interaction (ACI)
//! - Screen capture
//! - Input simulation (mouse, keyboard)
//! - State observation for self-correction

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

mod fallback;

use super::{MouseButton, ScrollDirection};

/// Screenshot data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    pub width: u32,
    pub height: u32,
    pub base64_data: String,
}

/// Platform abstraction trait for computer use operations
#[async_trait]
pub trait ComputerPlatform: Send + Sync {
    /// Click at the given coordinates
    async fn click(&self, x: i32, y: i32, button: MouseButton) -> anyhow::Result<()>;

    /// Type text at current cursor position
    async fn type_text(&self, text: &str) -> anyhow::Result<()>;

    /// Move mouse to coordinates
    async fn mouse_move(&self, x: i32, y: i32) -> anyhow::Result<()>;

    /// Send keyboard shortcuts (e.g., "Ctrl+C")
    async fn send_keys(&self, keys: &str) -> anyhow::Result<()>;

    /// Scroll in the given direction
    async fn scroll(&self, direction: ScrollDirection, amount: i32) -> anyhow::Result<()>;

    /// Capture full screen
    async fn screenshot_full(&self) -> anyhow::Result<Screenshot>;

    /// Capture specific screen area
    async fn screenshot_area(
        &self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Screenshot>;

    /// Resolve element by accessibility ID to coordinates
    async fn resolve_element_by_id(&self, id: &str) -> anyhow::Result<(i32, i32)>;

    /// Resolve element by path (e.g., "window[0]/button[3]") to coordinates
    async fn resolve_element_by_path(&self, path: &str) -> anyhow::Result<(i32, i32)>;

    /// Inspect element at coordinates
    async fn inspect_element(&self, x: i32, y: i32) -> anyhow::Result<String>;

    /// Inspect full accessibility tree
    async fn inspect_full(&self) -> anyhow::Result<String>;

    /// Capture current state for self-correction (summary)
    async fn capture_state(&self) -> anyhow::Result<String>;
}

/// Create platform-specific implementation
pub fn create_platform() -> anyhow::Result<Box<dyn ComputerPlatform>> {
    #[cfg(target_os = "linux")]
    {
        if cfg!(feature = "computer-use-linux") {
            Ok(Box::new(linux::LinuxPlatform::new()?))
        } else {
            Ok(Box::new(fallback::FallbackPlatform::new()?))
        }
    }

    #[cfg(target_os = "macos")]
    {
        if cfg!(feature = "computer-use-macos") {
            Ok(Box::new(macos::MacOSPlatform::new()?))
        } else {
            Ok(Box::new(fallback::FallbackPlatform::new()?))
        }
    }

    #[cfg(target_os = "windows")]
    {
        if cfg!(feature = "computer-use-windows") {
            Ok(Box::new(windows::WindowsPlatform::new()?))
        } else {
            Ok(Box::new(fallback::FallbackPlatform::new()?))
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("Computer use not supported on this platform")
    }
}
