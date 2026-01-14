import React, { useState, useEffect } from 'react';
import styles from '../crossroads.module.scss';

interface HecateWelcomeProps {
  compact?: boolean;
  maxChars?: number;
}

const HecateWelcome: React.FC<HecateWelcomeProps> = ({ compact = false, maxChars = 80 }) => {
  const [fullContent, setFullContent] = useState<string>('');
  const [originalContent, setOriginalContent] = useState<string>('');
  const [displayedContent, setDisplayedContent] = useState<string>('');
  const [isLoading, setIsLoading] = useState(true);
  const [isTyping, setIsTyping] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isTruncated, setIsTruncated] = useState(false);
  const [showTooltip, setShowTooltip] = useState(false);

  // Use Erebus as the main routing point
  const erebusUrl = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000';

  // Fetch new message on mount (only once)
  useEffect(() => {
    let mounted = true;

    const loadWelcome = async () => {
      if (mounted) {
        await fetchHecateWelcome();
      }
    };

    loadWelcome();

    return () => {
      mounted = false;
    };
  }, []);

  // Typing animation effect
  useEffect(() => {
    if (!fullContent || isLoading) {
      return;
    }

    setIsTyping(true);
    setDisplayedContent('');

    let currentIndex = 0;
    const typingSpeed = 30; // milliseconds per character

    const typingInterval = setInterval(() => {
      if (currentIndex < fullContent.length) {
        setDisplayedContent(fullContent.slice(0, currentIndex + 1));
        currentIndex++;
      } else {
        setIsTyping(false);
        clearInterval(typingInterval);
      }
    }, typingSpeed);

    return () => clearInterval(typingInterval);
  }, [fullContent, isLoading]);

  const fetchHecateWelcome = async () => {
    setIsLoading(true);
    setError(null);
    setFullContent('');
    setDisplayedContent('');

    try {
      const response = await fetch(`${erebusUrl}/api/agents/hecate/chat`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          message: `You are Hecate, the guide of NullBlock's Crossroads marketplace. 

Write a brief welcome message with these requirements:
1. FIRST SENTENCE is CRITICAL - it must be captivating and mysterious (this is what users see before clicking "more info")
2. Make the opening hook compelling and intriguing about autonomous agents/Web3 automation
3. Keep total message to 1-2 sentences (under 80 characters)
4. Use cyberpunk, enigmatic tone - make them want to learn more
5. NO emojis, NO quotes, NO attribution

Example first sentence style: "Where autonomous agents converge"
Example second sentence: "Your gateway to Web3 automation awaits"

The first sentence should make users curious to click the info icon.

Return ONLY the welcome text.`,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to fetch welcome message');
      }

      const data = await response.json();
      let content = data.content || data.message || 'Welcome to Crossroads';

      // Store original content
      setOriginalContent(content);

      // Truncate to max characters if in compact mode
      if (compact && content.length > maxChars) {
        setIsTruncated(true);
        content = `${content.substring(0, maxChars - 3)}...`;
      } else {
        setIsTruncated(false);
      }

      setFullContent(content);
    } catch (err) {
      console.error('Failed to fetch Hecate welcome:', err);
      setError('Failed to load welcome message');
      // Fallback content
      let fallback = 'Where autonomous agents converge. Your gateway awaits.';

      // Store original
      setOriginalContent(fallback);

      // Truncate fallback if needed
      if (compact && fallback.length > maxChars) {
        setIsTruncated(true);
        fallback = `${fallback.substring(0, maxChars - 3)}...`;
      } else {
        setIsTruncated(false);
      }

      setFullContent(fallback);
    } finally {
      setIsLoading(false);
    }
  };

  if (isLoading) {
    return (
      <div className={`${styles.hecateWelcome} ${compact ? styles.compact : ''}`}>
        <div className={styles.loadingDots}>
          <span>.</span>
          <span>.</span>
          <span>.</span>
        </div>
      </div>
    );
  }

  if (error && !fullContent) {
    return null;
  }

  return (
    <div className={`${styles.hecateWelcome} ${compact ? styles.compact : ''}`}>
      <div className={styles.welcomeContainer}>
        <p className={styles.welcomeText}>
          "{displayedContent}"{isTyping && <span className={styles.cursor}>|</span>}
        </p>
        {!isTyping && <span className={styles.attribution}>- Hecate</span>}
        {isTruncated && !isTyping && (
          <>
            <div className={styles.tooltipWrapper}>
              <button
                className={styles.expandButton}
                onMouseEnter={() => setShowTooltip(true)}
                onMouseLeave={() => setShowTooltip(false)}
                onClick={() => setShowTooltip(!showTooltip)}
                aria-label="View full quote"
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                >
                  <circle cx="12" cy="12" r="10" />
                  <line x1="12" y1="16" x2="12" y2="12" />
                  <line x1="12" y1="8" x2="12.01" y2="8" />
                </svg>
              </button>
            </div>
            {showTooltip && (
              <div className={styles.tooltipPortal}>
                <div className={styles.tooltip}>
                  <div className={styles.tooltipContent}>{originalContent}</div>
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};

export default HecateWelcome;
