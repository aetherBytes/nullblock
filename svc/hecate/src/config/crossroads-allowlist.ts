export const CROSSROADS_ALLOWED_TOOLS: string[] = [
  // Discovery (public, read-only) - marketplace browsing only
  'crossroads_list_tools',
  'crossroads_get_tool_info',
  'crossroads_list_agents',
  'crossroads_list_hot',
  'crossroads_get_stats',
];

export function isToolAllowedInCrossroads(toolName: string): boolean {
  return CROSSROADS_ALLOWED_TOOLS.includes(toolName);
}
