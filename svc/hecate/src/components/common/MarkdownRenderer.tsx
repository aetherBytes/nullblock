import React from 'react';
import ReactMarkdown from 'react-markdown';
import rehypeHighlight from 'rehype-highlight';
import 'prismjs/themes/prism-tomorrow.css';
import styles from './MarkdownRenderer.module.scss';

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

const MarkdownRenderer: React.FC<MarkdownRendererProps> = ({ content, className }) => {
  return (
    <div className={`${styles.markdownContainer} ${className || ''}`}>
      <ReactMarkdown
        rehypePlugins={[rehypeHighlight]}
        components={{
          // Custom styling for code blocks
          code: ({ node, inline, className, children, ...props }) => {
            const match = /language-(\w+)/.exec(className || '');
            const language = match ? match[1] : '';
            
            if (!inline && language) {
              return (
                <pre className={styles.codeBlock}>
                  <code className={className} {...props} style={{ textAlign: 'left', whiteSpace: 'pre' }}>
                    {children}
                  </code>
                </pre>
              );
            }
            
            return (
              <code className={styles.inlineCode} {...props}>
                {children}
              </code>
            );
          },
          // Custom styling for pre blocks
          pre: ({ children }) => (
            <div className={styles.preWrapper}>
              {children}
            </div>
          ),
          // Custom styling for blockquotes
          blockquote: ({ children }) => (
            <blockquote className={styles.blockquote}>
              {children}
            </blockquote>
          ),
          // Custom styling for lists
          ul: ({ children }) => (
            <ul className={styles.list}>
              {children}
            </ul>
          ),
          ol: ({ children }) => (
            <ol className={styles.orderedList}>
              {children}
            </ol>
          ),
          // Custom styling for links
          a: ({ href, children }) => (
            <a href={href} className={styles.link} target="_blank" rel="noopener noreferrer">
              {children}
            </a>
          ),
          // Custom styling for tables
          table: ({ children }) => (
            <div className={styles.tableWrapper}>
              <table className={styles.table}>
                {children}
              </table>
            </div>
          ),
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};

export default MarkdownRenderer;
