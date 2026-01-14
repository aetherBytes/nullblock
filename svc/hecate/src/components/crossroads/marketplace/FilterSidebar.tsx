import React from 'react';
import styles from '../crossroads.module.scss';
import type { FilterState, ServiceCategory } from '../types';

interface FilterSidebarProps {
  filters: FilterState;
  onFilterChange: (filters: FilterState) => void;
}

const categories: ServiceCategory[] = [
  'Agent',
  'Workflow',
  'Tool',
  'McpServer',
  'Dataset',
  'Model',
];

const FilterSidebar: React.FC<FilterSidebarProps> = ({ filters, onFilterChange }) => {
  const handleCategoryChange = (category: ServiceCategory) => {
    onFilterChange({
      ...filters,
      category: filters.category === category ? undefined : category,
    });
  };

  const handlePriceChange = (isFree?: boolean) => {
    onFilterChange({
      ...filters,
      is_free: filters.is_free === isFree ? undefined : isFree,
    });
  };

  const handleRatingChange = (minRating: number) => {
    onFilterChange({
      ...filters,
      min_rating: filters.min_rating === minRating ? undefined : minRating,
    });
  };

  const clearFilters = () => {
    onFilterChange({});
  };

  const hasFilters = Boolean(
    filters.category || filters.is_free !== undefined || filters.min_rating,
  );

  return (
    <div className={styles.filterSidebar}>
      <h3>Filters</h3>

      <div className={styles.filterSection}>
        <h4>Category</h4>
        {categories.map((category) => (
          <div key={category} className={styles.filterOption}>
            <input
              type="checkbox"
              id={`category-${category}`}
              checked={filters.category === category}
              onChange={() => handleCategoryChange(category)}
            />
            <label htmlFor={`category-${category}`}>{category}</label>
          </div>
        ))}
      </div>

      <div className={styles.filterSection}>
        <h4>Price</h4>
        <div className={styles.filterOption}>
          <input
            type="radio"
            id="price-all"
            name="price"
            checked={filters.is_free === undefined}
            onChange={() => handlePriceChange(undefined)}
          />
          <label htmlFor="price-all">All</label>
        </div>
        <div className={styles.filterOption}>
          <input
            type="radio"
            id="price-free"
            name="price"
            checked={filters.is_free === true}
            onChange={() => handlePriceChange(true)}
          />
          <label htmlFor="price-free">Free</label>
        </div>
        <div className={styles.filterOption}>
          <input
            type="radio"
            id="price-paid"
            name="price"
            checked={filters.is_free === false}
            onChange={() => handlePriceChange(false)}
          />
          <label htmlFor="price-paid">Paid</label>
        </div>
      </div>

      <div className={styles.filterSection}>
        <h4>Rating</h4>
        {[5, 4, 3].map((rating) => (
          <div key={rating} className={styles.filterOption}>
            <input
              type="checkbox"
              id={`rating-${rating}`}
              checked={filters.min_rating === rating}
              onChange={() => handleRatingChange(rating)}
            />
            <label htmlFor={`rating-${rating}`}>{'⭐'.repeat(rating)} & up</label>
          </div>
        ))}
      </div>

      {hasFilters && (
        <button className={styles.clearFilters} onClick={clearFilters}>
          Clear All Filters
        </button>
      )}
    </div>
  );
};

export default FilterSidebar;
