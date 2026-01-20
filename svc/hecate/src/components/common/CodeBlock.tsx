import React, { useState } from 'react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import styles from './CodeBlock.module.scss';

const customTheme = {
  'code[class*="language-"]': {
    color: '#f8f8f2',
    background: 'rgba(185, 103, 255, 0.1)',
    textShadow: '0 1px rgba(0, 0, 0, 0.3)',
    fontFamily: 'Consolas, Monaco, "Andale Mono", "Ubuntu Mono", monospace',
    fontSize: '0.85em',
    textAlign: 'left',
    whiteSpace: 'pre',
    wordSpacing: 'normal',
    wordBreak: 'normal',
    wordWrap: 'normal',
    lineHeight: '1.5',
    tabSize: '2',
  },
  'pre[class*="language-"]': {
    color: '#f8f8f2',
    background: 'rgba(185, 103, 255, 0.1)',
    textShadow: '0 1px rgba(0, 0, 0, 0.3)',
    fontFamily: 'Consolas, Monaco, "Andale Mono", "Ubuntu Mono", monospace',
    fontSize: '0.85em',
    textAlign: 'left',
    whiteSpace: 'pre',
    wordSpacing: 'normal',
    wordBreak: 'normal',
    wordWrap: 'normal',
    lineHeight: '1.5',
    tabSize: '2',
    padding: '1em',
    margin: '0',
    overflow: 'auto',
    borderRadius: '8px',
  },
  comment: { color: '#6a9955', fontStyle: 'italic' },
  punctuation: { color: '#d4d4d4' },
  property: { color: '#9cdcfe' },
  tag: { color: '#f92672' },
  constant: { color: '#f92672' },
  boolean: { color: '#569cd6' },
  number: { color: '#b5cea8' },
  string: { color: '#ce9178' },
  operator: { color: '#d4d4d4' },
  function: { color: '#4ec9b0' },
  'class-name': { color: '#4ec9b0' },
  keyword: { color: '#569cd6' },
};

interface CodeBlockProps {
  code: string;
  language?: string;
  title?: string;
  showCopy?: boolean;
  collapsible?: boolean;
  defaultCollapsed?: boolean;
  maxHeight?: string;
}

const CopyButton: React.FC<{ code: string }> = ({ code }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy code:', err);
    }
  };

  return (
    <button
      onClick={handleCopy}
      className={styles.copyButton}
      title={copied ? 'Copied!' : 'Copy code'}
    >
      {copied ? '✓' : '📋'}
    </button>
  );
};

const CodeBlock: React.FC<CodeBlockProps> = ({
  code,
  language = 'json',
  title,
  showCopy = true,
  collapsible = false,
  defaultCollapsed = false,
  maxHeight,
}) => {
  const [isCollapsed, setIsCollapsed] = useState(defaultCollapsed);

  const content = (
    <div className={styles.codeBlockWrapper} style={maxHeight ? { maxHeight } : undefined}>
      {showCopy && <CopyButton code={code} />}
      <SyntaxHighlighter
        style={customTheme}
        language={language}
        PreTag="div"
        className={styles.codeBlock}
        customStyle={{
          margin: 0,
          borderRadius: title || collapsible ? '0 0 8px 8px' : '8px',
          background: 'rgba(185, 103, 255, 0.08)',
          border: '1px solid rgba(185, 103, 255, 0.2)',
          borderTop: title || collapsible ? 'none' : '1px solid rgba(185, 103, 255, 0.2)',
        }}
        showLineNumbers={false}
        wrapLines
        wrapLongLines
      >
        {code}
      </SyntaxHighlighter>
    </div>
  );

  if (collapsible) {
    return (
      <div className={styles.collapsibleContainer}>
        <button
          className={styles.collapseHeader}
          onClick={() => setIsCollapsed(!isCollapsed)}
        >
          <span className={styles.collapseIcon}>{isCollapsed ? '▶' : '▼'}</span>
          <span className={styles.collapseTitle}>{title || 'Code'}</span>
        </button>
        {!isCollapsed && content}
      </div>
    );
  }

  if (title) {
    return (
      <div className={styles.titledContainer}>
        <div className={styles.titleBar}>
          <span className={styles.titleText}>{title}</span>
        </div>
        {content}
      </div>
    );
  }

  return content;
};

export const formatJson = (data: unknown): string => {
  try {
    return JSON.stringify(data, null, 2);
  } catch {
    return String(data);
  }
};

export default CodeBlock;
