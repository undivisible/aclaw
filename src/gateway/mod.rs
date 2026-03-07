//! Gateway — HTTP/WebSocket server for remote agent management
//! Allows external tools, editors, and clients to interact with agents

pub mod server;

pub use server::{Gateway, ChatRequest, ChatResponse, ContainerStatus, start_gateway};
