pub const CROSSROADS_PUBLIC_TOOLS: &[&str] = &[
    // Public discovery tools (no wallet required)
    "crossroads_list_tools",
    "crossroads_get_tool_info",
    "crossroads_list_agents",
    "crossroads_list_hot",
    "crossroads_get_stats",
];

pub const HECATE_MEMORY_TOOLS: &[&str] = &[
    // Engram CRUD
    "engram_create",
    "engram_get",
    "engram_search",
    "engram_update",
    "engram_delete",
    "engram_list_by_type",
    // Hecate memory helpers
    "hecate_remember",
    "hecate_pin_engram",
    "hecate_unpin_engram",
    "hecate_cleanup",
    // User profile (stored as engrams)
    "user_profile_get",
    "user_profile_update",
    // Session management (engram-backed)
    "hecate_list_sessions",
    "hecate_new_session",
    "hecate_resume_session",
    "hecate_delete_session",
];

pub const CROSSROADS_ALLOWED_TOOLS: &[&str] = &[
    // Public discovery tools (available to everyone)
    "crossroads_list_tools",
    "crossroads_get_tool_info",
    "crossroads_list_agents",
    "crossroads_list_hot",
    "crossroads_get_stats",
    // Read-only engram access for logged-in users
    "engram_get",
    "engram_search",
    "engram_list_by_type",
    "user_profile_get",
];

pub const AGENT_ALLOWED_TOOLS: &[&str] = &[
    // Basic engram CRUD only for now
    "engram_create",
    "engram_get",
    "engram_search",
    "engram_update",
    "engram_delete",
    "engram_list_by_type",
];

pub fn get_hecate_allowed_tools() -> Vec<&'static str> {
    let mut tools = HECATE_MEMORY_TOOLS.to_vec();
    tools.extend_from_slice(CROSSROADS_PUBLIC_TOOLS);
    tools
}

pub fn get_prelogin_allowed_tools() -> Vec<&'static str> {
    CROSSROADS_PUBLIC_TOOLS.to_vec()
}

pub fn is_tool_allowed_for_agent(tool_name: &str) -> bool {
    AGENT_ALLOWED_TOOLS.contains(&tool_name)
}

pub fn is_tool_allowed_for_hecate(tool_name: &str) -> bool {
    HECATE_MEMORY_TOOLS.contains(&tool_name) || CROSSROADS_PUBLIC_TOOLS.contains(&tool_name)
}

pub fn is_tool_allowed_for_crossroads(tool_name: &str) -> bool {
    CROSSROADS_ALLOWED_TOOLS.contains(&tool_name)
}

pub fn is_public_tool(tool_name: &str) -> bool {
    CROSSROADS_PUBLIC_TOOLS.contains(&tool_name)
}
