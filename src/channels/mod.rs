//! Channel abstraction — communication interfaces
//! Support for: CLI, Telegram, Discord, Slack, WhatsApp, Signal, Matrix, IRC,
//!              Google Chat, MS Teams, WebSocket

pub mod traits;
pub mod cli;
pub mod telegram;
pub mod discord;
pub mod slack;
pub mod whatsapp;
pub mod signal;
pub mod matrix;
pub mod irc;
pub mod googlechat;
pub mod msteams;

pub use traits::{Channel, IncomingMessage, OutgoingMessage};

