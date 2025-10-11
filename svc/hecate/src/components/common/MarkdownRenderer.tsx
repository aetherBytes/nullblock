import React, { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';

// Create a custom theme to avoid dependency issues
const customTheme = {
  'code[class*="language-"]': {
    color: '#f8f8f2',
    background: 'rgba(185, 103, 255, 0.1)',
    textShadow: '0 1px rgba(0, 0, 0, 0.3)',
    fontFamily: 'Consolas, Monaco, "Andale Mono", "Ubuntu Mono", monospace',
    fontSize: '1em',
    textAlign: 'left',
    whiteSpace: 'pre',
    wordSpacing: 'normal',
    wordBreak: 'normal',
    wordWrap: 'normal',
    lineHeight: '1.5',
    MozTabSize: '4',
    OTabSize: '4',
    tabSize: '4',
    WebkitHyphens: 'none',
    MozHyphens: 'none',
    msHyphens: 'none',
    hyphens: 'none',
  },
  'pre[class*="language-"]': {
    color: '#f8f8f2',
    background: 'rgba(185, 103, 255, 0.1)',
    textShadow: '0 1px rgba(0, 0, 0, 0.3)',
    fontFamily: 'Consolas, Monaco, "Andale Mono", "Ubuntu Mono", monospace',
    fontSize: '1em',
    textAlign: 'left',
    whiteSpace: 'pre',
    wordSpacing: 'normal',
    wordBreak: 'normal',
    wordWrap: 'normal',
    lineHeight: '1.5',
    MozTabSize: '4',
    OTabSize: '4',
    tabSize: '4',
    WebkitHyphens: 'none',
    MozHyphens: 'none',
    msHyphens: 'none',
    hyphens: 'none',
    padding: '1em',
    margin: '.5em 0',
    overflow: 'auto',
    borderRadius: '0.3em',
    backgroundAttachment: 'local',
  },
  'comment': {
    color: '#6a9955',
    fontStyle: 'italic',
  },
  'prolog': {
    color: '#6a9955',
  },
  'doctype': {
    color: '#6a9955',
  },
  'cdata': {
    color: '#6a9955',
  },
  'punctuation': {
    color: '#d4d4d4',
  },
  'property': {
    color: '#9cdcfe',
  },
  'tag': {
    color: '#f92672',
  },
  'constant': {
    color: '#f92672',
  },
  'symbol': {
    color: '#f92672',
  },
  'deleted': {
    color: '#f92672',
  },
  'boolean': {
    color: '#569cd6',
  },
  'number': {
    color: '#b5cea8',
  },
  'selector': {
    color: '#ce9178',
  },
  'attr-name': {
    color: '#ce9178',
  },
  'string': {
    color: '#ce9178',
  },
  'char': {
    color: '#ce9178',
  },
  'builtin': {
    color: '#ce9178',
  },
  'inserted': {
    color: '#ce9178',
  },
  'operator': {
    color: '#d4d4d4',
  },
  'entity': {
    color: '#d4d4d4',
  },
  'url': {
    color: '#d4d4d4',
  },
  'variable': {
    color: '#d4d4d4',
  },
  'atrule': {
    color: '#dcdcaa',
  },
  'attr-value': {
    color: '#dcdcaa',
  },
  'function': {
    color: '#4ec9b0',
  },
  'class-name': {
    color: '#4ec9b0',
  },
  'keyword': {
    color: '#569cd6',
  },
  'regex': {
    color: '#d16969',
  },
  'important': {
    color: '#d16969',
    fontWeight: 'bold',
  },
  'bold': {
    fontWeight: 'bold',
  },
  'italic': {
    fontStyle: 'italic',
  },
};

import styles from './MarkdownRenderer.module.scss';

interface MarkdownRendererProps {
  content: string;
  className?: string;
  images?: Array<{
    url: string;
    alt?: string;
    caption?: string;
  }>;
}

// Copy button component
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

// Image display component
const ImageDisplay: React.FC<{ 
  url: string; 
  alt?: string; 
  caption?: string; 
}> = ({ url, alt, caption }) => {
  const [imageError, setImageError] = useState(false);
  const [imageLoading, setImageLoading] = useState(true);
  const [downloading, setDownloading] = useState(false);

  const handleDownload = async () => {
    try {
      setDownloading(true);
      
      // Generate filename with timestamp
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
      const filename = `hecate-image-${timestamp}.png`;
      
      if (url.startsWith('data:')) {
        // Handle base64 data URLs
        const link = document.createElement('a');
        link.href = url;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
      } else {
        // Handle regular URLs
        const response = await fetch(url);
        const blob = await response.blob();
        const blobUrl = window.URL.createObjectURL(blob);
        
        const link = document.createElement('a');
        link.href = blobUrl;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        
        // Clean up blob URL
        window.URL.revokeObjectURL(blobUrl);
      }
    } catch (error) {
      console.error('Failed to download image:', error);
      alert('Failed to download image. Please try right-clicking and saving the image manually.');
    } finally {
      setDownloading(false);
    }
  };

  return (
    <div className={styles.imageContainer}>
      {imageLoading && (
        <div className={styles.imageLoading}>
          <div className={styles.loadingSpinner}></div>
          <span>Loading image...</span>
        </div>
      )}
      {imageError ? (
        <div className={styles.imageError}>
          <span>❌ Failed to load image</span>
          <a href={url} target="_blank" rel="noopener noreferrer" className={styles.imageLink}>
            View original
          </a>
        </div>
      ) : (
        <>
          <div className={styles.imageWrapper}>
            <img
              src={url}
              alt={alt || 'Generated image'}
              className={styles.chatImage}
              onLoad={() => setImageLoading(false)}
              onError={() => {
                setImageError(true);
                setImageLoading(false);
              }}
              loading="lazy"
            />
            <div className={styles.imageOverlay}>
              <button
                onClick={handleDownload}
                className={styles.downloadButton}
                disabled={downloading}
                title="Download image"
              >
                {downloading ? '⏳' : '💾'} {downloading ? 'Downloading...' : 'Download'}
              </button>
            </div>
          </div>
          {caption && (
            <div className={styles.imageCaption}>
              {caption}
            </div>
          )}
        </>
      )}
    </div>
  );
};

const MarkdownRenderer: React.FC<MarkdownRendererProps> = React.memo(({ content, className, images = [] }) => {
  return (
    <div className={`${styles.markdownContainer} ${className || ''}`}>
      {/* Render images first if they exist */}
      {images.length > 0 && (
        <div className={styles.imagesContainer}>
          {images.map((image, index) => (
            <ImageDisplay
              key={index}
              url={image.url}
              alt={image.alt}
              caption={image.caption}
            />
          ))}
        </div>
      )}

      <ReactMarkdown
        components={{
          // Custom styling for code blocks with syntax highlighting
          code: ({ node, inline, className, children, ...props }) => {
            const match = /language-(\w+)/.exec(className || '');
            const language = match ? match[1] : '';
            
            // Language mapping for common aliases
            const languageMap: { [key: string]: string } = {
              'js': 'javascript',
              'ts': 'typescript',
              'golang': 'go',
              'c': 'cpp',
              'cs': 'csharp',
              'rb': 'ruby',
              'shell': 'bash',
              'sh': 'bash',
              'yml': 'yaml',
              'md': 'markdown'
            };
            
            const normalizedLanguage = languageMap[language] || language;
            
            if (!inline && normalizedLanguage) {
              const codeString = String(children).replace(/\n$/, '');
              return (
                <div className={styles.codeBlockWrapper}>
                  <CopyButton code={codeString} />
                  <SyntaxHighlighter
                    style={customTheme}
                    language={normalizedLanguage}
                    PreTag="div"
                    className={styles.codeBlock}
                    customStyle={{
                      margin: '1rem 0',
                      borderRadius: '8px',
                      fontSize: '0.9rem',
                      lineHeight: '1.4',
                      textAlign: 'left',
                      background: 'rgba(185, 103, 255, 0.1)',
                      border: '1px solid rgba(185, 103, 255, 0.2)',
                    }}
                    showLineNumbers={false}
                    wrapLines={true}
                    wrapLongLines={true}
                    {...props}
                  >
                    {codeString}
                  </SyntaxHighlighter>
                </div>
              );
            }
            
            return (
              <code className={styles.inlineCode} {...props}>
                {children}
              </code>
            );
          },
          // Custom styling for pre blocks
          pre: ({ children }) => {
            // If it's already a SyntaxHighlighter, don't wrap it
            if (React.isValidElement(children) && children.type === SyntaxHighlighter) {
              return children;
            }
            return (
              <div className={styles.preWrapper}>
                {children}
              </div>
            );
          },
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
}, (prevProps, nextProps) => {
  return (
    prevProps.content === nextProps.content &&
    prevProps.className === nextProps.className &&
    prevProps.images?.length === nextProps.images?.length &&
    prevProps.images?.every((img, idx) => img.url === nextProps.images?.[idx]?.url)
  );
});

MarkdownRenderer.displayName = 'MarkdownRenderer';

export default MarkdownRenderer;
