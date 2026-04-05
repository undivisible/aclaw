//! Plugin host, manifest discovery, and optional integrations (Poke bridge, future native loaders).

mod host;
pub mod manifest;
#[cfg(feature = "plugin-poke")]
pub mod poke;

pub use host::PluginHost;
pub use manifest::{load_manifest, merge_manifest_into_config, PluginManifestFile};
