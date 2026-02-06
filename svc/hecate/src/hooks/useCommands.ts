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

const PRE_LOGIN_COMMANDS: SlashCommand[] = [
  {
    name: '/help',
    description: 'Show available commands',
    category: 'builtin',
    action: 'execute',
  },
  {
    name: '/list-tools',
    description: 'List available MCP tools',
    category: 'builtin',
    action: 'tools',
  },
  {
    name: '/status',
    description: 'Show service status',
    category: 'builtin',
    action: 'execute',
  },
];

const POST_LOGIN_COMMANDS: SlashCommand[] = [
  {
    name: '/help',
    description: 'Show available commands',
    category: 'builtin',
    action: 'execute',
  },
  {
    name: '/list-tools',
    description: 'List available MCP tools',
    category: 'builtin',
    action: 'tools',
  },
  {
    name: '/status',
    description: 'Show agent and service status',
    category: 'builtin',
    action: 'execute',
  },
  {
    name: '/mcp',
    description: 'Show MCP service status and tool categories',
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
    name: '/new',
    description: 'Start a new chat session',
    category: 'builtin',
    action: 'execute',
  },
  {
    name: '/sessions',
    description: 'View and manage chat sessions',
    category: 'builtin',
    action: 'execute',
  },
  {
    name: '/consensus',
    description: 'Query the LLM consensus service',
    category: 'mcp',
    action: 'insert',
  },
  {
    name: '/engrams',
    description: 'Browse stored engrams',
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

export function useCommands(_erebusUrl: string = 'http://localhost:3000', isAuthenticated: boolean = false) {
  const [mcpTools, setMcpTools] = useState<McpTool[]>([]);
  const [agentTools, setAgentTools] = useState<McpTool[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const activeBuiltinCommands = isAuthenticated ? POST_LOGIN_COMMANDS : PRE_LOGIN_COMMANDS;

  // Fetch agent tools from agents service MCP endpoint
  const fetchMcpTools = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const agentMcpResponse = await fetch('http://localhost:9003/mcp/jsonrpc', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          id: 1,
          method: 'tools/list',
          params: {}
        })
      });

      if (agentMcpResponse.ok) {
        const data = await agentMcpResponse.json();
        const tools = data.result?.tools || [];
        setAgentTools(tools);
        setMcpTools(tools);
        return tools;
      }

      return [];
    } catch (e) {
      console.warn('Failed to fetch agent tools:', e);
      setError('Agent service unavailable');
      return [];
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Fetch tools on mount
  useEffect(() => {
    fetchMcpTools();
  }, [fetchMcpTools]);

  // All available commands (builtin + agent tool commands)
  const allCommands = useMemo((): SlashCommand[] => {
    // Only include tool commands if authenticated
    if (!isAuthenticated) {
      return [...activeBuiltinCommands];
    }

    const toolCommands: SlashCommand[] = agentTools.map((tool) => ({
      name: `/${tool.name}`,
      description: tool.description || `Tool: ${tool.name}`,
      category: 'mcp' as const,
      action: 'insert' as const,
    }));

    return [...activeBuiltinCommands, ...toolCommands];
  }, [agentTools, isAuthenticated, activeBuiltinCommands]);

  // Fuzzy filter commands based on input
  const filterCommands = useCallback(
    (query: string): SlashCommand[] => {
      if (!query || query === '/') {
        // Show builtin commands first when just "/" is typed
        return activeBuiltinCommands;
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
    if (!isAuthenticated) {
      return `## Commands

\`/help\` — Show this help

\`/list-tools\` — List available tools

\`/status\` — Show service status

---

**Limited mode.** Connect wallet to unlock memories, sessions, and tools.`;
    }

    const builtinHelp = activeBuiltinCommands.map(
      (cmd) => `\`${cmd.name}\` — ${cmd.description}`,
    ).join('\n\n');

    return `## Commands

${builtinHelp}

---

**${agentTools.length}** tools enabled. Type \`/\` to browse.`;
  }, [agentTools.length, isAuthenticated, activeBuiltinCommands]);

  // Generate tool list text (shows only agent's allowed tools)
  const getToolListText = useCallback((): string => {
    if (!isAuthenticated) {
      return `## Tools

Connect wallet to view and use agent tools.

Available after login:
- Engram management (create, read, update, delete)
- Memory search and filtering
- More tools as they are enabled`;
    }

    if (agentTools.length === 0) {
      return 'No tools available. Agent service may be offline.';
    }

    let text = `## Agent Tools (${agentTools.length})\n\n`;

    agentTools.forEach((tool) => {
      text += `**${tool.name}**\n${tool.description || 'No description'}\n\n`;
    });

    return text;
  }, [agentTools, isAuthenticated]);

  // Get MCP status text
  const getMcpStatusText = useCallback((): string => {
    if (!isAuthenticated) {
      return `## Agent Tools

**Status**: Not logged in

Connect wallet to access tools.`;
    }

    const toolCount = agentTools.length;
    const categories = new Set(agentTools.map((t) => t.name.split('_')[0]));

    return `## Agent Tools

**Connected**: ${toolCount > 0 ? 'Yes' : 'No'}
**Tools**: ${toolCount}
**Categories**: ${categories.size}

### Categories
${Array.from(categories)
  .sort()
  .map((cat) => `- ${cat}`)
  .join('\n')}

Use \`/list-tools\` to see details.`;
  }, [agentTools, isAuthenticated]);

  return {
    commands: allCommands,
    mcpTools,
    agentTools,
    isLoading,
    error,
    filterCommands,
    fetchMcpTools,
    isToolQuery,
    getHelpText,
    getToolListText,
    getMcpStatusText,
    builtinCommands: activeBuiltinCommands,
  };
}

export default useCommands;
