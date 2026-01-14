import React from 'react';
import styles from '../crossroads.module.scss';

export type ServiceCategory =
  | 'Agent'
  | 'Workflow'
  | 'Tool'
  | 'McpServer'
  | 'Dataset'
  | 'Model'
  | 'ArbFarm';

interface CategoryBadgeProps {
  category: ServiceCategory;
  showIcon?: boolean;
}

const categoryIcons: Record<ServiceCategory, string> = {
  Agent: '🤖',
  Workflow: '🔄',
  Tool: '🔧',
  McpServer: '🌐',
  Dataset: '📊',
  Model: '🧠',
  ArbFarm: '⚡',
};

const CategoryBadge: React.FC<CategoryBadgeProps> = ({ category, showIcon = true }) => {
  const categoryClass = category.toLowerCase().replace(/\s/g, '');

  return (
    <span className={`${styles.categoryBadge} ${styles[categoryClass]}`}>
      {showIcon && <span>{categoryIcons[category]}</span>}
      <span>{category}</span>
    </span>
  );
};

export default CategoryBadge;
