import React, { useState, useEffect, useRef } from 'react';
import styles from './digitizing-text.module.scss';

export type DigitizingTheme = 'null-dark' | 'null-light' | 'null-metal' | 'light' | 'blue';

interface DigitizingTextProps {
  text: string;
  duration?: number; // Duration in milliseconds before the text fades away (0 means no auto-fade)
  onComplete?: () => void;
  theme?: DigitizingTheme | 'light';
}

const DigitizingText: React.FC<DigitizingTextProps> = ({ 
  text, 
  duration = 10000, // Default 10 seconds
  onComplete,
  theme = 'light' // Default to light theme
}) => {
  const [displayText, setDisplayText] = useState<string>('');
  const [isVisible, setIsVisible] = useState<boolean>(true);
  const [isComplete, setIsComplete] = useState<boolean>(false);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const textRef = useRef<string>(text);
  const isAnimatingRef = useRef<boolean>(false);

  // Reset when text changes
  useEffect(() => {
    // Clear any existing timeouts
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    
    // Reset state
    textRef.current = text;
    setDisplayText('');
    setIsVisible(true);
    setIsComplete(false);
    isAnimatingRef.current = false;
    
    // Start animation
    animateText();
    
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [text]);

  const animateText = () => {
    if (isAnimatingRef.current) return;
    isAnimatingRef.current = true;
    
    let currentIndex = 0;
    const speed = 30; // Speed of digitizing effect (reduced from 50ms to 30ms)
    
    const addNextChar = () => {
      if (currentIndex <= text.length) {
        setDisplayText(text.substring(0, currentIndex));
        currentIndex++;
        timeoutRef.current = setTimeout(addNextChar, speed);
      } else {
        setIsComplete(true);
        isAnimatingRef.current = false;
        if (onComplete) {
          onComplete();
        }
        
        // Only set timeout to fade out if duration is not 0
        if (duration > 0) {
          timeoutRef.current = setTimeout(() => {
            setIsVisible(false);
          }, duration);
        }
      }
    };
    
    addNextChar();
  };

  if (!isVisible) return null;

  // Map app themes to digitizing themes
  const getDigitizingTheme = () => {
    switch (theme) {
      case 'null-dark':
        return 'null-dark';
      case 'light':
        return 'light';
      case 'blue':
        return 'blue';
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