use crate::channels::IncomingMessage;

const GROUP_KEYWORDS: &[&str] = &[
    "unthinkclaw",
    "unthinkclaw-live",
    "plugin",
    "plugins",
    "plugin layer",
    "manifest",
    "settings",
    "command",
    "commands",
    "upgrade",
    "upgrades",
    "transport",
    "help",
    "bot",
    "module",
    "modules",
    "configure",
    "configuration",
    "how do i",
    "what is",
    "what does",
    "why is",
    "can you",
    "could you",
];

pub fn should_respond(msg: &IncomingMessage) -> bool {
    if !msg.is_group {
        return true;
    }

    let text = msg.text.trim().to_lowercase();
    if text.is_empty() {
        return false;
    }

    if text.starts_with('/') || text.starts_with('!') {
        return true;
    }

    let direct_mention = text.contains("@unthinkclaw") || text.contains("@unthinkclaw-live");
    let topic_hit = GROUP_KEYWORDS.iter().any(|keyword| text.contains(keyword));
    direct_mention || topic_hit
}

pub fn routing_guidance(is_group: bool, transport: &str) -> Option<String> {
    if !is_group {
        return None;
    }

    Some(format!(
        "## Group chat routing
Transport: {transport}

Respond even without a direct mention when the message is about unthinkclaw, unthinkclaw-live, plugins, plugin manifests, settings, commands, upgrades, transport, or asks for help with the bot. Stay silent for unrelated ambient chatter. When responding, be concise and reference the relevant command, plugin, or setting path when useful.",
        transport = transport
    ))
}

pub fn plugin_layer_policy_prompt() -> String {
    "## Plugin layer policy
Treat live functionality as a plugin layer on top of core. Prefer plugin manifests, hooks, and layered overrides rather than hardcoding live-only behavior into core runtime paths. Keep plugin-specific configuration isolated from core defaults so upgrades remain compatible.
".to_string()
}

pub fn transport_policy_prompt(transport: &str) -> String {
    format!(
        "## Transport policy
Default transport: {transport}

Treat the configured transport as a thin adapter over the core runtime, with settings overrides isolated from the core config so upgrades do not conflict. Use context-aware routing for group chats, but only answer ambient messages when they are clearly about the assistant, its plugins, or operational commands.",
        transport = transport
    )
}
