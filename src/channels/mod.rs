//! Channel abstraction — communication interfaces.

pub mod traits;
pub mod cli;

pub use traits::{Channel, IncomingMessage, OutgoingMessage};
