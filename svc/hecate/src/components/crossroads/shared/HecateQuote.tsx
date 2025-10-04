import React, { useState, useEffect } from 'react';
import styles from './HecateQuote.module.scss';

interface HecateQuoteProps {
  refreshTrigger?: number;
  compact?: boolean;
}

const HecateQuote: React.FC<HecateQuoteProps> = ({ refreshTrigger = 0, compact = false }) => {
  const [quote, setQuote] = useState<string>('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchQuote = async () => {
      setLoading(true);
      try {
        const response = await fetch('/api/agents/hecate/chat', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            message: 'Give me a single profound, mystical quote about the Crossroads marketplace - the meeting place of agents, tools, and workflows. Keep it under 25 words, poetic and fitting for Hecate, goddess of crossroads.',
            session_id: 'crossroads-quote',
          }),
        });

        if (response.ok) {
          const data = await response.json();
          const content = data.response || data.message || '';
          // Clean up the response - remove quotes if present
          const cleanedQuote = content.replace(/^["']|["']$/g, '').trim();
          setQuote(cleanedQuote);
        } else {
          // Fallback quote if API fails
          setQuote('At the crossroads, all paths converge. Here, agents and workflows intertwine, forging new destinies.');
        }
      } catch (error) {
        console.error('Failed to fetch Hecate quote:', error);
        // Fallback quote
        setQuote('At the crossroads, all paths converge. Here, agents and workflows intertwine, forging new destinies.');
      } finally {
        setLoading(false);
      }
    };

    fetchQuote();
  }, [refreshTrigger]);

  const quoteClass = compact ? `${styles.hecateQuote} ${styles.compact}` : styles.hecateQuote;

  if (loading) {
    return (
      <div className={quoteClass}>
        <div className={styles.quoteText}>
          <div className={styles.loadingDots}>
            <span>.</span><span>.</span><span>.</span>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={quoteClass}>
      <div className={styles.quoteText}>
        <p>"{quote}" <span className={styles.attribution}>- Hecate</span></p>
      </div>
    </div>
  );
};

export default HecateQuote;

