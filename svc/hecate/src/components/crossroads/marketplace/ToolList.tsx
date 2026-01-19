import React, { useState } from 'react';
import styles from '../crossroads.module.scss';
import type { MCPTool, ToolCategory } from '../../../types/crossroads';
import { TOOL_CATEGORY_ICONS, TOOL_CATEGORY_LABELS } from '../../../types/crossroads';

interface ToolListProps {
  tools: MCPTool[];
  onToolClick?: (tool: MCPTool) => void;
  loading?: boolean;
}

interface CategoryGroup {
  category: ToolCategory;
  tools: MCPTool[];
}

const ToolList: React.FC<ToolListProps> = ({ tools, onToolClick, loading }) => {
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set(['consensus', 'scanner', 'curve']));
  const [expandedTool, setExpandedTool] = useState<string | null>(null);

  const groupedTools: CategoryGroup[] = React.useMemo(() => {
    const groups: Record<string, MCPTool[]> = {};

    for (const tool of tools) {
      const category = tool.category;
      if (!groups[category]) {
        groups[category] = [];
      }
      groups[category].push(tool);
    }

    const categoryOrder: ToolCategory[] = [
      'consensus', 'scanner', 'edge', 'strategy', 'curve', 'position',
      'research', 'kol', 'threat', 'event', 'engram', 'learning',
      'swarm', 'approval', 'utility', 'integration', 'analysis', 'unknown'
    ];

    return categoryOrder
      .filter(cat => groups[cat] && groups[cat].length > 0)
      .map(cat => ({
        category: cat,
        tools: groups[cat],
      }));
  }, [tools]);

  const toggleCategory = (category: string) => {
    setExpandedCategories(prev => {
      const next = new Set(prev);
      if (next.has(category)) {
        next.delete(category);
      } else {
        next.add(category);
      }
      return next;
    });
  };

  const toggleToolExpand = (toolName: string) => {
    setExpandedTool(prev => prev === toolName ? null : toolName);
  };

  if (loading) {
    return (
      <div className={styles.toolListLoading}>
        {[1, 2, 3].map(i => (
          <div key={i} className={styles.toolListSkeleton}>
            <div className={styles.skeletonHeader} />
            <div className={styles.skeletonItems}>
              {[1, 2, 3].map(j => (
                <div key={j} className={styles.skeletonItem} />
              ))}
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (tools.length === 0) {
    return (
      <div className={styles.toolListEmpty}>
        <span className={styles.emptyIcon}>🔍</span>
        <p>No tools found matching your criteria</p>
      </div>
    );
  }

  return (
    <div className={styles.toolList}>
      {groupedTools.map(({ category, tools: categoryTools }) => (
        <div key={category} className={styles.toolCategorySection}>
          <button
            className={`${styles.categoryHeader} ${expandedCategories.has(category) ? styles.expanded : ''}`}
            onClick={() => toggleCategory(category)}
          >
            <span className={styles.categoryHeaderIcon}>
              {TOOL_CATEGORY_ICONS[category]}
            </span>
            <span className={styles.categoryHeaderTitle}>
              {TOOL_CATEGORY_LABELS[category]}
            </span>
            <span className={styles.categoryHeaderCount}>
              {categoryTools.length} tool{categoryTools.length !== 1 ? 's' : ''}
            </span>
            <span className={styles.categoryHeaderChevron}>
              {expandedCategories.has(category) ? '▼' : '▶'}
            </span>
          </button>

          {expandedCategories.has(category) && (
            <div className={styles.toolItems}>
              {categoryTools.map((tool) => (
                <div
                  key={tool.name}
                  className={`${styles.toolItem} ${tool.isHot ? styles.hotTool : ''}`}
                >
                  <div
                    className={styles.toolItemMain}
                    onClick={() => toggleToolExpand(tool.name)}
                  >
                    <div className={styles.toolItemLeft}>
                      <span className={styles.toolName}>{tool.name}</span>
                      {tool.isHot && <span className={styles.hotIndicator}>🔥</span>}
                    </div>
                    <div className={styles.toolItemRight}>
                      <span className={styles.toolProvider}>{tool.provider}</span>
                      <span className={styles.toolExpandIcon}>
                        {expandedTool === tool.name ? '−' : '+'}
                      </span>
                    </div>
                  </div>

                  {expandedTool === tool.name && (
                    <div className={styles.toolItemDetails}>
                      <p className={styles.toolDescription}>{tool.description}</p>
                      <div className={styles.toolMeta}>
                        {tool.relatedCow && (
                          <span className={styles.toolCowBadge}>
                            Part of {tool.relatedCow}
                          </span>
                        )}
                        <span className={styles.toolEndpoint}>
                          {tool.endpoint}
                        </span>
                      </div>
                      <button
                        className={styles.viewDetailsButton}
                        onClick={(e) => {
                          e.stopPropagation();
                          onToolClick?.(tool);
                        }}
                      >
                        View Full Details →
                      </button>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

export default ToolList;
