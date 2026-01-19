import React from 'react';
import styles from '../crossroads.module.scss';
import type { MCPTool } from '../../../types/crossroads';
import { TOOL_CATEGORY_ICONS, TOOL_CATEGORY_LABELS } from '../../../types/crossroads';

interface ToolDetailPanelProps {
  tool: MCPTool;
  onClose: () => void;
}

const ToolDetailPanel: React.FC<ToolDetailPanelProps> = ({ tool, onClose }) => {
  const formatSchema = (schema: Record<string, unknown>): string => {
    try {
      return JSON.stringify(schema, null, 2);
    } catch {
      return '{}';
    }
  };

  const getSchemaProperties = (schema: Record<string, unknown>): Array<{ name: string; type: string; description?: string; required: boolean }> => {
    const properties = schema.properties as Record<string, { type?: string; description?: string }> | undefined;
    const required = (schema.required as string[]) || [];

    if (!properties) return [];

    return Object.entries(properties).map(([name, prop]) => ({
      name,
      type: prop.type || 'any',
      description: prop.description,
      required: required.includes(name),
    }));
  };

  const schemaProps = getSchemaProperties(tool.inputSchema);

  return (
    <div className={styles.toolDetailOverlay} onClick={onClose}>
      <div className={styles.toolDetailPanel} onClick={(e) => e.stopPropagation()}>
        <button className={styles.toolDetailClose} onClick={onClose}>
          ×
        </button>

        <div className={styles.toolDetailHeader}>
          <div className={styles.toolDetailHeaderLeft}>
            <span className={styles.toolDetailIcon}>
              {TOOL_CATEGORY_ICONS[tool.category]}
            </span>
            <div className={styles.toolDetailTitleGroup}>
              <h2 className={styles.toolDetailName}>{tool.name}</h2>
              <span className={styles.toolDetailCategory}>
                {TOOL_CATEGORY_LABELS[tool.category]}
              </span>
            </div>
          </div>
          {tool.isHot && (
            <span className={styles.toolDetailHotBadge}>🔥 Hot Tool</span>
          )}
        </div>

        <p className={styles.toolDetailDescription}>{tool.description}</p>

        <div className={styles.toolDetailMeta}>
          <div className={styles.toolDetailMetaItem}>
            <span className={styles.metaLabel}>Provider</span>
            <span className={styles.metaValue}>{tool.provider}</span>
          </div>
          <div className={styles.toolDetailMetaItem}>
            <span className={styles.metaLabel}>Endpoint</span>
            <span className={styles.metaValueCode}>{tool.endpoint}</span>
          </div>
          {tool.relatedCow && (
            <div className={styles.toolDetailMetaItem}>
              <span className={styles.metaLabel}>Related COW</span>
              <span className={styles.metaValueBadge}>{tool.relatedCow}</span>
            </div>
          )}
        </div>

        <div className={styles.toolDetailSection}>
          <h3 className={styles.toolDetailSectionTitle}>Input Schema</h3>
          {schemaProps.length > 0 ? (
            <div className={styles.schemaTable}>
              <div className={styles.schemaTableHeader}>
                <span>Parameter</span>
                <span>Type</span>
                <span>Required</span>
                <span>Description</span>
              </div>
              {schemaProps.map((prop) => (
                <div key={prop.name} className={styles.schemaTableRow}>
                  <span className={styles.schemaParamName}>{prop.name}</span>
                  <span className={styles.schemaParamType}>{prop.type}</span>
                  <span className={styles.schemaParamRequired}>
                    {prop.required ? '✓' : '−'}
                  </span>
                  <span className={styles.schemaParamDesc}>
                    {prop.description || '—'}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <p className={styles.schemaEmpty}>No input parameters required</p>
          )}
        </div>

        <div className={styles.toolDetailSection}>
          <h3 className={styles.toolDetailSectionTitle}>Raw Schema</h3>
          <pre className={styles.toolDetailCode}>
            {formatSchema(tool.inputSchema)}
          </pre>
        </div>

        <div className={styles.toolDetailSection}>
          <h3 className={styles.toolDetailSectionTitle}>Usage Example</h3>
          <pre className={styles.toolDetailCode}>
{`// Call this tool via MCP
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "${tool.name}",
    "arguments": {
      // Add required parameters here
    }
  }
}`}
          </pre>
        </div>

        <div className={styles.toolDetailActions}>
          <button className={styles.toolDetailActionPrimary}>
            Copy Endpoint
          </button>
          <button className={styles.toolDetailActionSecondary}>
            View Documentation
          </button>
        </div>
      </div>
    </div>
  );
};

export default ToolDetailPanel;
