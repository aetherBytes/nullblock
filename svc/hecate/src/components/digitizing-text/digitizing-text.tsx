import React, { useState, useEffect } from 'react';
import styles from './digitizing-text.module.scss';

export type DigitizingTheme = 'null-dark' | 'null-light' | 'null-metal' | 'light';

interface DigitizingTextProps {
  text: string;
  duration?: number; // Duration in milliseconds before the text fades away (0 means no auto-fade)
  onComplete?: () => void;
  theme?: DigitizingTheme | 'null' | 'light';
}

const DigitizingText: React.FC<DigitizingTextProps> = ({ 
  text, 
  duration = 10000, // Default 10 seconds
  onComplete,
  theme = 'cyberpunk' // Default to cyberpunk theme
}) => {
  const [displayText, setDisplayText] = useState<string>('');
  const [isVisible, setIsVisible] = useState<boolean>(true);
  const [isComplete, setIsComplete] = useState<boolean>(false);

  useEffect(() => {
    let currentIndex = 0;
    const interval = setInterval(() => {
      if (currentIndex <= text.length) {
        setDisplayText(text.substring(0, currentIndex));
        currentIndex++;
      } else {
        clearInterval(interval);
        setIsComplete(true);
      }
    }, 50); // Speed of digitizing effect

    // Only set timeout to fade out if duration is not 0$cyberpunk-accent: #ffd700;

    let fadeTimeout: NodeJS.Timeout | null = null;
    if (duration > 0) {
      fadeTimeout = setTimeout(() => {
        setIsVisible(false);
        if (onComplete) {
          onComplete();
        }
      }, duration);
    }

    return () => {
      clearInterval(interval);
      if (fadeTimeout) {
        clearTimeout(fadeTimeout);
      }
    };
  }, [text, duration, onComplete]);

  if (!isVisible) return null;

  // Map app themes to digitizing themes
  const getDigitizingTheme = () => {
    switch (theme) {
      case 'null':
        return 'blue';
      case 'light':
        return 'light';
      default:
        return theme;
    }
  };

  return (
    <div className={`${styles.digitizingText} ${styles[getDigitizingTheme()]} ${isComplete ? styles.complete : ''}`}>
      {displayText}
      {!isComplete && <span className={styles.cursor}>_</span>}
    </div>
  );
};

export default DigitizingText; 