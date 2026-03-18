//! Managed skills tool — create, inspect, and list persisted workspace skills.

use std::path::PathBuf;

use async_trait::async_trait;
use serde::Deserialize;

use crate::skills::{
    discover_skills_for_workspace, load_skill_content, save_managed_skill, ManagedSkillInput,
};

use super::{Tool, ToolResult, ToolSpec};

pub struct SkillManagerTool {
    workspace: PathBuf,
}

impl SkillManagerTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[derive(Deserialize)]
struct SkillManagerArgs {
    action: String,
    name: Option<String>,
    description: Option<String>,
    body: Option<String>,
}

#[async_trait]
impl Tool for SkillManagerTool {
    fn name(&self) -> &str {
        "skill_manager"
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "skill_manager".to_string(),
            description: "List, inspect, or save managed skills in the workspace.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["list", "get", "save"] },
                    "name": { "type": "string", "description": "Skill name for get/save" },
                    "description": { "type": "string", "description": "Skill description for save" },
                    "body": { "type": "string", "description": "Skill body markdown for save" }
                },
                "required": ["action"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> anyhow::Result<ToolResult> {
        let args: SkillManagerArgs = serde_json::from_str(arguments)?;
        match args.action.as_str() {
            "list" => {
                let skills = discover_skills_for_workspace(Some(&self.workspace));
                if skills.is_empty() {
                    return Ok(ToolResult::success("No skills found."));
                }
                let output = skills
                    .iter()
                    .map(|skill| format!("{} — {}", skill.name, skill.description))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(ToolResult::success(output))
            }
            "get" => {
                let Some(name) = args.name else {
                    return Ok(ToolResult::error("name is required for get"));
                };
                let skills = discover_skills_for_workspace(Some(&self.workspace));
                let Some(skill) = skills.into_iter().find(|skill| skill.name == name) else {
                    return Ok(ToolResult::error("Skill not found"));
                };
                Ok(ToolResult::success(
                    load_skill_content(&skill).unwrap_or_default(),
                ))
            }
            "save" => {
                let Some(name) = args.name else {
                    return Ok(ToolResult::error("name is required for save"));
                };
                let Some(description) = args.description else {
                    return Ok(ToolResult::error("description is required for save"));
                };
                let Some(body) = args.body else {
                    return Ok(ToolResult::error("body is required for save"));
                };
                let path = save_managed_skill(
                    &self.workspace,
                    &ManagedSkillInput {
                        name,
                        description,
                        body,
                    },
                )?;
                Ok(ToolResult::success(format!(
                    "Managed skill saved at {}",
                    path.display()
                )))
            }
            _ => Ok(ToolResult::error("Unknown action")),
        }
    }
}
