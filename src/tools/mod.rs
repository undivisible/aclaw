//! Tool abstraction — agent capabilities (shell, file I/O, web, etc.)

pub mod traits;
pub mod shell;
pub mod file_ops;
pub mod web_search;
pub mod vibemania;

pub use traits::{Tool, ToolSpec, ToolResult};
