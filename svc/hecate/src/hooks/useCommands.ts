import { useState, useEffect, useCallback, useMemo } from 'react';

export interface SlashCommand {
  name: string;
  description: string;
  category: 'builtin' | 'mcp' | 'agent';
  action?: 'execute' | 'insert' | 'tools';
  handler?: () => Promise<string> | string;
}

export interface McpTool {
  name: string;
  description: string;
  inputSchema?: Record<string, unknown>;
}

const BUILTIN_COMMANDS: SlashCommand[] = [
  {
    name: '/help',
    description: 'Show available commands and how to use them',
    category: 'builtin',
    action: 'execute',
  },
  {
    name: '/list-tools',
    description: 'List all available MCP tools from connected services',
    category: 'builtin',
    action: 'tools',
  },
  {
    name: '/mcp',
    description: 'Show MCP service status and available tool categories',
    category: 'builtin',
    action: 'execute',
  },
  {
    name: '/clear',
    description: 'Clear the chat history',
    category: 'builtin',
    action: 'execute',
  },
  {
    name: '/status',
    description: 'Show agent and service status',
    category: 'builtin',
    action: 'execute',
  },
  {
    name: '/consensus',
    description: 'Query the LLM consensus service for multi-model decisions',
    category: 'mcp',
    action: 'insert',
  },
  {
    name: '/engrams',
    description: 'Browse and search stored engrams (learning data)',
    category: 'mcp',
    action: 'insert',
  },
  {
    name: '/wallet',
    description: 'Show wallet status and balance',
    category: 'mcp',
    action: 'execute',
  },
  {
    name: '/strategies',
    description: 'List active trading strategies',
    category: 'mcp',
    action: 'execute',
  },
  {
    name: '/positions',
    description: 'Show open positions',
    category: 'mcp',
    action: 'execute',
  },
];

// Natural language patterns that should trigger help/tool listing
const TOOL_QUERY_PATTERNS = [
  /what (tools|commands|capabilities) (do you have|are available|can you use|are live|are online|are working)/i,
  /show me (your |the )?(tools|commands|capabilities)/i,
  /list (your |the )?(tools|commands|capabilities)/i,
  /what can you do/i,
  /help me with tools/i,
  /do you have any tools/i,
  /available (tools|commands|mcp)/i,
  /mcp tools/i,
  /what('s| is) (live|online|available|working)/i,
  /what (tools|mcp|commands) (are|is) (live|online|available|working)/i,
  /tools (live|online|available)/i,
  /internal (mcp|tools|tooling)/i,
  /your (mcp|tooling)/i,
  /help$/i,
  /what commands/i,
  /show commands/i,
];

export function useCommands(erebusUrl: string = 'http://localhost:3000') {
  const [mcpTools, setMcpTools] = useState<McpTool[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch MCP tools from backend
  const fetchMcpTools = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      // Try arb-farm directly first (has most tools)
      const arbResponse = await fetch('http://localhost:9007/mcp/tools');
      if (arbResponse.ok) {
        const tools = await arbResponse.json();
        // Handle both array and object with tools property
        const toolList = Array.isArray(tools) ? tools : tools.tools || [];
        setMcpTools(toolList);
        return toolList;
      }

      // Fallback to Erebus proxy
      const response = await fetch(`${erebusUrl}/api/mcp/tools`);
      if (response.ok) {
        const data = await response.json();
        const toolList = Array.isArray(data) ? data : data.tools || [];
        setMcpTools(toolList);
        return toolList;
      }

      throw new Error('Failed to fetch MCP tools');
    } catch (err) {
      console.error('Failed to fetch MCP tools:', err);
      setError((err as Error).message);
      return [];
    } finally {
      setIsLoading(false);
    }
  }, [erebusUrl]);

  // Fetch tools on mount
  useEffect(() => {
    fetchMcpTools();
  }, [fetchMcpTools]);

  // All available commands (builtin + MCP tool commands)
  const allCommands = useMemo((): SlashCommand[] => {
    const mcpCommands: SlashCommand[] = mcpTools.slice(0, 50).map((tool) => ({
      name: `/${tool.name}`,
      description: tool.description || `MCP tool: ${tool.name}`,
      category: 'mcp' as const,
      action: 'insert' as const,
    }));

    return [...BUILTIN_COMMANDS, ...mcpCommands];
  }, [mcpTools]);

  // Fuzzy filter commands based on input
  const filterCommands = useCallback(
    (query: string): SlashCommand[] => {
      if (!query || query === '/') {
        // Show builtin commands first when just "/" is typed
        return BUILTIN_COMMANDS;
      }

      const searchTerm = query.toLowerCase().replace(/^\//, '');

      return allCommands
        .filter((cmd) => {
          const name = cmd.name.toLowerCase().replace(/^\//, '');
          const desc = cmd.description.toLowerCase();

          // Exact prefix match gets priority
          if (name.startsWith(searchTerm)) return true;

          // Fuzzy match on name
          if (name.includes(searchTerm)) return true;

          // Match on description words
          if (desc.includes(searchTerm)) return true;

          return false;
        })
        .sort((a, b) => {
          const aName = a.name.toLowerCase().replace(/^\//, '');
          const bName = b.name.toLowerCase().replace(/^\//, '');

          // Exact prefix matches first
          const aExact = aName.startsWith(searchTerm);
          const bExact = bName.startsWith(searchTerm);
          if (aExact && !bExact) return -1;
          if (!aExact && bExact) return 1;

          // Then builtin commands
          if (a.category === 'builtin' && b.category !== 'builtin') return -1;
          if (a.category !== 'builtin' && b.category === 'builtin') return 1;

          // Then alphabetically
          return aName.localeCompare(bName);
        })
        .slice(0, 10); // Limit to 10 results
    },
    [allCommands],
  );

  // Check if a message is asking about tools (natural language)
  const isToolQuery = useCallback((message: string): boolean => {
    return TOOL_QUERY_PATTERNS.some((pattern) => pattern.test(message));
  }, []);

  // Generate help text for a command
  const getHelpText = useCallback((): string => {
    const builtinHelp = BUILTIN_COMMANDS.map(
      (cmd) => `**${cmd.name}** - ${cmd.description}`,
    ).join('\n');

    return `## Available Commands

### Built-in Commands
${builtinHelp}

### MCP Tools
Type \`/\` followed by a tool name to see available MCP tools.
Currently **${mcpTools.length}** MCP tools available.

### Natural Language
You can also ask me about tools in plain English:
- "What tools do you have?"
- "Show me available commands"
- "List MCP capabilities"`;
  }, [mcpTools.length]);

  // Generate tool list text
  const getToolListText = useCallback((): string => {
    if (mcpTools.length === 0) {
      return 'No MCP tools available. Services may be offline.';
    }

    // Group tools by prefix
    const grouped: Record<string, McpTool[]> = {};
    mcpTools.forEach((tool) => {
      const prefix = tool.name.split('_')[0];
      if (!grouped[prefix]) grouped[prefix] = [];
      grouped[prefix].push(tool);
    });

    let text = `## Available MCP Tools (${mcpTools.length} total)\n\n`;

    Object.entries(grouped)
      .sort(([a], [b]) => a.localeCompare(b))
      .forEach(([prefix, tools]) => {
        text += `### ${prefix} (${tools.length})\n`;
        tools.slice(0, 5).forEach((tool) => {
          text += `- **${tool.name}**: ${tool.description?.slice(0, 80) || 'No description'}${tool.description && tool.description.length > 80 ? '...' : ''}\n`;
        });
        if (tools.length > 5) {
          text += `- _...and ${tools.length - 5} more_\n`;
        }
        text += '\n';
      });

    return text;
  }, [mcpTools]);

  // Get MCP status text
  const getMcpStatusText = useCallback((): string => {
    const toolCount = mcpTools.length;
    const categories = new Set(mcpTools.map((t) => t.name.split('_')[0]));

    return `## MCP Service Status

**Connected**: ${toolCount > 0 ? 'Yes' : 'No'}
**Total Tools**: ${toolCount}
**Categories**: ${categories.size}

### Tool Categories
${Array.from(categories)
  .sort()
  .map((cat) => `- ${cat}`)
  .join('\n')}

Use \`/list-tools\` to see all available tools.`;
  }, [mcpTools]);

  return {
    commands: allCommands,
    mcpTools,
    isLoading,
    error,
    filterCommands,
    fetchMcpTools,
    isToolQuery,
    getHelpText,
    getToolListText,
    getMcpStatusText,
    builtinCommands: BUILTIN_COMMANDS,
  };
}

export default useCommands;
