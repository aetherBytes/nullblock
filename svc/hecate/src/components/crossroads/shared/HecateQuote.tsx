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
            message:
              'Give me a single brief transmission as HECATE, the MK1 vessel AI, about navigating the void or discovering new crossroads. Speak as a ship AI with calm authority and dry wit. Address the user as "visitor". One sentence only, under 20 words.',
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
          setQuote('Sensors online, visitor. The crossroads await our arrival.');
        }
      } catch (error) {
        console.error('Failed to fetch HECATE transmission:', error);
        // Fallback quote
        setQuote('Sensors online, visitor. The crossroads await our arrival.');
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
            <span>.</span>
            <span>.</span>
            <span>.</span>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={quoteClass}>
      <div className={styles.quoteText}>
        <p>
          "{quote}" <span className={styles.attribution}>- HECATE</span>
        </p>
      </div>
    </div>
  );
};

export default HecateQuote;
