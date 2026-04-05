//! Extension point for registering extra tools at runtime (native / dynamic plugins).

use std::sync::Arc;

use crate::tools::Tool;

#[derive(Default)]
pub struct PluginHost {
    pub extra_tools: Vec<Arc<dyn Tool>>,
}

impl PluginHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_tool(&mut self, tool: Arc<dyn Tool>) {
        self.extra_tools.push(tool);
    }
}
