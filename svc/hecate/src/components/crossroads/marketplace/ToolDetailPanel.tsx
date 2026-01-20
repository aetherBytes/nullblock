import React from 'react';
import styles from '../crossroads.module.scss';
import type { MCPTool } from '../../../types/crossroads';
import { TOOL_CATEGORY_ICONS, TOOL_CATEGORY_LABELS } from '../../../types/crossroads';
import CodeBlock, { formatJson } from '../../common/CodeBlock';

interface ToolDetailPanelProps {
  tool: MCPTool;
  onClose: () => void;
}

const ToolDetailPanel: React.FC<ToolDetailPanelProps> = ({ tool, onClose }) => {
  const generateUsageExample = (toolName: string, schema: Record<string, unknown>): string => {
    const properties = schema.properties as Record<string, { type?: string }> | undefined;
    const required = (schema.required as string[]) || [];

    const args: Record<string, unknown> = {};
    if (properties) {
      for (const [name, prop] of Object.entries(properties)) {
        if (required.includes(name)) {
          switch (prop.type) {
            case 'string':
              args[name] = `<${name}>`;
              break;
            case 'number':
            case 'integer':
              args[name] = 0;
              break;
            case 'boolean':
              args[name] = true;
              break;
            case 'array':
              args[name] = [];
              break;
            case 'object':
              args[name] = {};
              break;
            default:
              args[name] = `<${name}>`;
          }
        }
      }
    }

    return JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "tools/call",
      params: {
        name: toolName,
        arguments: args
      }
    }, null, 2);
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
          <h3 className={styles.toolDetailSectionTitle}>Input Schema</h3>
          <CodeBlock
            code={formatJson(tool.inputSchema)}
            language="json"
            maxHeight="200px"
          />
        </div>

        <div className={styles.toolDetailSection}>
          <h3 className={styles.toolDetailSectionTitle}>Usage Example</h3>
          <CodeBlock
            code={generateUsageExample(tool.name, tool.inputSchema)}
            language="json"
            title="MCP JSON-RPC Request"
            showCopy
          />
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
