import React from 'react';
import styles from '../crossroads.module.scss';
import type { MCPTool } from '../../../types/crossroads';
import { TOOL_CATEGORY_ICONS, TOOL_CATEGORY_LABELS } from '../../../types/crossroads';

interface HotToolsSectionProps {
  tools: MCPTool[];
  onToolClick?: (tool: MCPTool) => void;
}

const HotToolsSection: React.FC<HotToolsSectionProps> = ({ tools, onToolClick }) => {
  if (tools.length === 0) {
    return null;
  }

  return (
    <div className={styles.hotToolsSection}>
      <div className={styles.hotToolsHeader}>
        <span className={styles.hotToolsIcon}>🔥</span>
        <h2 className={styles.hotToolsTitle}>Hot Tools</h2>
        <span className={styles.hotToolsSubtitle}>LLM Consensus & Featured</span>
      </div>
      <div className={styles.hotToolsGrid}>
        {tools.map((tool) => (
          <div
            key={tool.name}
            className={styles.hotToolCard}
            onClick={() => onToolClick?.(tool)}
          >
            <div className={styles.hotToolHeader}>
              <span className={styles.hotToolCategoryIcon}>
                {TOOL_CATEGORY_ICONS[tool.category]}
              </span>
              <span className={styles.hotToolName}>{tool.name}</span>
              <span className={styles.hotBadge}>🔥</span>
            </div>
            <p className={styles.hotToolDescription}>{tool.description}</p>
            <div className={styles.hotToolMeta}>
              <span className={styles.hotToolCategory}>
                {TOOL_CATEGORY_LABELS[tool.category]}
              </span>
              {tool.relatedCow && (
                <span className={styles.hotToolCow}>
                  Part of {tool.relatedCow}
                </span>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default HotToolsSection;
