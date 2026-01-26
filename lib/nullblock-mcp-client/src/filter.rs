use crate::types::McpTool;

pub trait ToolFilter {
    fn filter(&self, tool: &McpTool) -> bool;
}

pub struct ReadOnlyFilter;

impl ToolFilter for ReadOnlyFilter {
    fn filter(&self, tool: &McpTool) -> bool {
        tool.annotations
            .as_ref()
            .and_then(|a| a.read_only_hint)
            .unwrap_or(false)
    }
}

pub struct TagFilter {
    tag: String,
}

impl TagFilter {
    pub fn new(tag: impl Into<String>) -> Self {
        Self { tag: tag.into() }
    }
}

impl ToolFilter for TagFilter {
    fn filter(&self, tool: &McpTool) -> bool {
        tool.tags
            .as_ref()
            .map(|tags| tags.iter().any(|t| t == &self.tag))
            .unwrap_or(false)
    }
}

pub struct NotDestructiveFilter;

impl ToolFilter for NotDestructiveFilter {
    fn filter(&self, tool: &McpTool) -> bool {
        tool.annotations
            .as_ref()
            .map(|a| !a.is_destructive())
            .unwrap_or(false)
    }
}

pub struct IdempotentFilter;

impl ToolFilter for IdempotentFilter {
    fn filter(&self, tool: &McpTool) -> bool {
        tool.annotations
            .as_ref()
            .and_then(|a| a.idempotent_hint)
            .unwrap_or(false)
    }
}

pub struct NamePrefixFilter {
    prefix: String,
}

impl NamePrefixFilter {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self { prefix: prefix.into() }
    }
}

impl ToolFilter for NamePrefixFilter {
    fn filter(&self, tool: &McpTool) -> bool {
        tool.name.starts_with(&self.prefix)
    }
}

pub struct CompositeFilter {
    filters: Vec<Box<dyn ToolFilter + Send + Sync>>,
    mode: CompositeMode,
}

pub enum CompositeMode {
    All,
    Any,
}

impl CompositeFilter {
    pub fn all(filters: Vec<Box<dyn ToolFilter + Send + Sync>>) -> Self {
        Self {
            filters,
            mode: CompositeMode::All,
        }
    }

    pub fn any(filters: Vec<Box<dyn ToolFilter + Send + Sync>>) -> Self {
        Self {
            filters,
            mode: CompositeMode::Any,
        }
    }
}

impl ToolFilter for CompositeFilter {
    fn filter(&self, tool: &McpTool) -> bool {
        match self.mode {
            CompositeMode::All => self.filters.iter().all(|f| f.filter(tool)),
            CompositeMode::Any => self.filters.iter().any(|f| f.filter(tool)),
        }
    }
}

pub fn filter_read_only(tools: Vec<McpTool>) -> Vec<McpTool> {
    let filter = ReadOnlyFilter;
    tools.into_iter().filter(|t| filter.filter(t)).collect()
}

pub fn filter_by_tag(tools: Vec<McpTool>, tag: &str) -> Vec<McpTool> {
    let filter = TagFilter::new(tag);
    tools.into_iter().filter(|t| filter.filter(t)).collect()
}

pub fn filter_not_destructive(tools: Vec<McpTool>) -> Vec<McpTool> {
    let filter = NotDestructiveFilter;
    tools.into_iter().filter(|t| filter.filter(t)).collect()
}

pub fn filter_idempotent(tools: Vec<McpTool>) -> Vec<McpTool> {
    let filter = IdempotentFilter;
    tools.into_iter().filter(|t| filter.filter(t)).collect()
}

pub fn filter_by_name_prefix(tools: Vec<McpTool>, prefix: &str) -> Vec<McpTool> {
    let filter = NamePrefixFilter::new(prefix);
    tools.into_iter().filter(|t| filter.filter(t)).collect()
}

pub fn apply_filter<F: ToolFilter>(tools: Vec<McpTool>, filter: &F) -> Vec<McpTool> {
    tools.into_iter().filter(|t| filter.filter(t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ToolAnnotations;
    use serde_json::json;

    fn make_tool(name: &str, read_only: Option<bool>, destructive: Option<bool>) -> McpTool {
        McpTool {
            name: name.to_string(),
            title: None,
            description: Some(format!("{} description", name)),
            input_schema: json!({"type": "object"}),
            output_schema: None,
            annotations: Some(ToolAnnotations {
                read_only_hint: read_only,
                destructive_hint: destructive,
                idempotent_hint: None,
                open_world_hint: None,
            }),
            tags: None,
        }
    }

    #[test]
    fn test_filter_read_only() {
        let tools = vec![
            make_tool("read_tool", Some(true), Some(false)),
            make_tool("write_tool", Some(false), Some(true)),
            make_tool("unknown_tool", None, None),
        ];

        let filtered = filter_read_only(tools);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "read_tool");
    }

    #[test]
    fn test_filter_not_destructive() {
        let tools = vec![
            make_tool("safe_tool", Some(true), Some(false)),
            make_tool("destructive_tool", Some(false), Some(true)),
        ];

        let filtered = filter_not_destructive(tools);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "safe_tool");
    }
}
