import React from 'react';
import styles from '../crossroads.module.scss';
import type { FilterState } from '../types';

interface CommandBarProps {
  filters: FilterState;
  onFilterChange: (filters: FilterState) => void;
}

const CommandBar: React.FC<CommandBarProps> = ({ filters, onFilterChange }) => {
  const handleSortChange = (sortBy: 'trending' | 'rating' | 'recent' | 'price') => {
    onFilterChange({
      ...filters,
      sort_by: filters.sort_by === sortBy ? undefined : sortBy,
    });
  };

  const handlePriceFilter = (isFree: boolean | undefined) => {
    onFilterChange({
      ...filters,
      is_free: filters.is_free === isFree ? undefined : isFree,
    });
  };

  return (
    <div className={styles.commandBar}>
      <span className={styles.commandLabel}>Sort:</span>
      <button
        className={`${styles.commandChip} ${filters.sort_by === 'trending' ? styles.active : ''}`}
        onClick={() => handleSortChange('trending')}
      >
        🔥 Trending
      </button>
      <button
        className={`${styles.commandChip} ${filters.sort_by === 'rating' ? styles.active : ''}`}
        onClick={() => handleSortChange('rating')}
      >
        ⭐ Top Rated
      </button>
      <button
        className={`${styles.commandChip} ${filters.sort_by === 'recent' ? styles.active : ''}`}
        onClick={() => handleSortChange('recent')}
      >
        🆕 Recent
      </button>

      <span className={styles.commandDivider}></span>

      <span className={styles.commandLabel}>Price:</span>
      <button
        className={`${styles.commandChip} ${filters.is_free === true ? styles.active : ''}`}
        onClick={() => handlePriceFilter(true)}
      >
        Free
      </button>
      <button
        className={`${styles.commandChip} ${filters.is_free === false ? styles.active : ''}`}
        onClick={() => handlePriceFilter(false)}
      >
        Paid
      </button>
    </div>
  );
};

export default CommandBar;

