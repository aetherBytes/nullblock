pub const CROSSROADS_ALLOWED_TOOLS: &[&str] = &[
    // Empty for now - will be populated as tools are verified for marketplace exposure
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

pub fn is_tool_allowed_for_agent(tool_name: &str) -> bool {
    AGENT_ALLOWED_TOOLS.contains(&tool_name)
}

pub fn is_tool_allowed_for_crossroads(tool_name: &str) -> bool {
    CROSSROADS_ALLOWED_TOOLS.contains(&tool_name)
}
