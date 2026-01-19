import React, { useEffect, useRef, useCallback } from 'react';
import type { SlashCommand } from '../../../hooks/useCommands';
import styles from './commandDropdown.module.scss';

interface CommandDropdownProps {
  commands: SlashCommand[];
  selectedIndex: number;
  onSelect: (command: SlashCommand) => void;
  onClose: () => void;
  query: string;
}

const CommandDropdown: React.FC<CommandDropdownProps> = ({
  commands,
  selectedIndex,
  onSelect,
  onClose,
  query,
}) => {
  const dropdownRef = useRef<HTMLDivElement>(null);
  const selectedRef = useRef<HTMLDivElement>(null);

  // Scroll selected item into view
  useEffect(() => {
    if (selectedRef.current) {
      selectedRef.current.scrollIntoView({ block: 'nearest' });
    }
  }, [selectedIndex]);

  // Close on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [onClose]);

  // Get category icon
  const getCategoryIcon = (category: string): string => {
    switch (category) {
      case 'builtin':
        return '⚡';
      case 'mcp':
        return '🔧';
      case 'agent':
        return '🤖';
      default:
        return '•';
    }
  };

  // Highlight matching text
  const highlightMatch = useCallback(
    (text: string) => {
      if (!query || query === '/') return text;

      const searchTerm = query.replace(/^\//, '').toLowerCase();
      const index = text.toLowerCase().indexOf(searchTerm);

      if (index === -1) return text;

      return (
        <>
          {text.slice(0, index)}
          <span className={styles.highlight}>{text.slice(index, index + searchTerm.length)}</span>
          {text.slice(index + searchTerm.length)}
        </>
      );
    },
    [query],
  );

  if (commands.length === 0) {
    return (
      <div ref={dropdownRef} className={styles.commandDropdown}>
        <div className={styles.noResults}>No matching commands</div>
      </div>
    );
  }

  return (
    <div ref={dropdownRef} className={styles.commandDropdown}>
      <div className={styles.dropdownHeader}>
        <span className={styles.headerIcon}>⌘</span>
        <span className={styles.headerText}>Commands</span>
        <span className={styles.headerHint}>↑↓ navigate • ↵ select • esc close</span>
      </div>
      <div className={styles.commandList}>
        {commands.map((cmd, index) => (
          <div
            key={cmd.name}
            ref={index === selectedIndex ? selectedRef : null}
            className={`${styles.commandItem} ${index === selectedIndex ? styles.selected : ''}`}
            onClick={() => onSelect(cmd)}
            onMouseEnter={() => {
              /* Could update selectedIndex on hover */
            }}
          >
            <span className={styles.categoryIcon}>{getCategoryIcon(cmd.category)}</span>
            <div className={styles.commandContent}>
              <span className={styles.commandName}>{highlightMatch(cmd.name)}</span>
              <span className={styles.commandDescription}>{cmd.description}</span>
            </div>
            <span className={styles.commandCategory}>{cmd.category}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default CommandDropdown;
