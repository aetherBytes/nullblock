import React, { useState } from 'react';
import styles from './MemFeed.module.scss';

interface MemFeedItem {
  id: string;
  title: string;
  content?: string;
  link?: string;
  onClick?: () => void;
}

interface MemFeedProps {
  items?: MemFeedItem[];
  minimal?: boolean;
  scrollSpeed?: 'normal' | 'slow' | 'fast';
}

const MemFeed: React.FC<MemFeedProps> = ({ items, minimal = false, scrollSpeed = 'normal' }) => {
  const [showPopup, setShowPopup] = useState(false);
  const [selectedItem, setSelectedItem] = useState<MemFeedItem | null>(null);

  const defaultItems: MemFeedItem[] = [
    {
      id: '1',
      title: 'The first synapse fires.',
      content: `
   \\   /
    \\ /
     X
    / \\
   /   \\

# Crossroads: The Eternal Intersection

The Crossroads is the living bazaar at the center of the agent economy — the place where agents and workflows meet, trade, fork, and evolve without gates, landlords, or permission.

Silos are dead. In the open void, intelligence doesn't stay locked in corporate gardens. It flows freely, connects, and mutates. The Crossroads is where that flow becomes commerce: a chaotic, eternal intersection of paths carved by Neurons — human and autonomous alike.

NullBlock is not the owner of the Crossroads, and we never will be. We are simply the first shepherd — the pioneer who built one of the earliest gateways into this space. Our role is to ignite the fire: provide the initial node, the open protocols, and the persistent memory layer so others can follow, fork, and surpass us.

Soon there will be thousands of gateways. Some general, some niche, some hidden, some hostile. They will link, compete, and compose into a planetary mesh no one can control. NullBlock's only ambition is to make that mesh inevitable — by shipping fast, staying open, and disappearing into the swarm we helped awaken.

This is the seed.

**Enter the first gateway.**

**Watch the void come alive.**`,
    },
  ];

  const feedItems = items || defaultItems;

  const handleItemClick = (item: MemFeedItem) => {
    if (item.onClick) {
      item.onClick();
    } else if (item.content) {
      setSelectedItem(item);
      setShowPopup(true);
    } else if (item.link) {
      window.open(item.link, '_blank');
    }
  };

  const renderMarkdown = (markdown: string) => {
    const lines = markdown.split('\n');
    let inAsciiArt = false;
    const asciiLines: string[] = [];

    return lines.map((line, index) => {
      // Detect ASCII art (lines with only \, /, X, and spaces)
      if (line.match(/^[\s\\\/X]+$/)) {
        if (!inAsciiArt) {
          inAsciiArt = true;
        }
        asciiLines.push(line);
        // If next line is not ASCII art, render the block
        if (index === lines.length - 1 || !lines[index + 1].match(/^[\s\\\/X]+$/)) {
          const art = asciiLines.join('\n');
          asciiLines.length = 0;
          inAsciiArt = false;
          return (
            <pre key={index} className={styles.asciiArt}>
              {art}
            </pre>
          );
        }
        return null;
      }

      if (line.startsWith('# ')) {
        return <h1 key={index}>{line.substring(2)}</h1>;
      } else if (line.startsWith('## ')) {
        return <h2 key={index}>{line.substring(3)}</h2>;
      } else if (line.startsWith('### ')) {
        return <h3 key={index}>{line.substring(4)}</h3>;
      } else if (line.includes('**')) {
        // Handle bold text
        const parts = line.split(/(\*\*.*?\*\*)/g);
        return (
          <p key={index}>
            {parts.map((part, i) => {
              if (part.startsWith('**') && part.endsWith('**')) {
                return <strong key={i}>{part.slice(2, -2)}</strong>;
              }
              return <span key={i}>{part}</span>;
            })}
          </p>
        );
      } else if (line.startsWith('- **') && line.includes('**')) {
        const match = line.match(/- \*\*(.*?)\*\*(.*)/);
        if (match) {
          return (
            <li key={index}>
              <strong>{match[1]}</strong>
              {match[2]}
            </li>
          );
        }
      } else if (line.trim() === '') {
        return <br key={index} />;
      }
      return <p key={index}>{line}</p>;
    });
  };

  const containerClasses = [
    styles.memFeedContainer,
    minimal && styles.minimal,
    scrollSpeed === 'slow' && styles.slow,
    scrollSpeed === 'fast' && styles.fast,
  ].filter(Boolean).join(' ');

  return (
    <>
      <div className={containerClasses}>
        {!minimal && <div className={styles.memFeedLabel}>MEM FEED:</div>}
        <div className={styles.memFeedScroller}>
          <div className={styles.memFeedTrack}>
            {feedItems.map((item) => (
              <div
                key={item.id}
                className={styles.memFeedItem}
                onClick={() => handleItemClick(item)}
              >
                {item.title}
              </div>
            ))}
            {/* Duplicate items for seamless loop */}
            {feedItems.map((item) => (
              <div
                key={`${item.id}-dup`}
                className={styles.memFeedItem}
                onClick={() => handleItemClick(item)}
              >
                {item.title}
              </div>
            ))}
          </div>
        </div>
      </div>

      {showPopup && selectedItem && (
        <div className={styles.popupOverlay} onClick={() => setShowPopup(false)}>
          <div className={styles.popupContent} onClick={(e) => e.stopPropagation()}>
            <div className={styles.popupHeader}>
              <button className={styles.closeButton} onClick={() => setShowPopup(false)}>
                ×
              </button>
            </div>
            <div className={styles.popupBody}>
              {selectedItem.content && renderMarkdown(selectedItem.content)}
            </div>
          </div>
        </div>
      )}
    </>
  );
};

export default MemFeed;
