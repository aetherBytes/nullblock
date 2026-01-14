import React from 'react';
import styles from '../arbfarm.module.scss';

interface MetricCardProps {
  label: string;
  value: string;
  subValue?: string;
  trend?: 'up' | 'down';
  trendValue?: string;
  color?: string;
  onClick?: () => void;
}

const MetricCard: React.FC<MetricCardProps> = ({
  label,
  value,
  subValue,
  trend,
  trendValue,
  color,
  onClick,
}) => (
  <div
    className={`${styles.metricCard} ${onClick ? styles.clickable : ''}`}
    onClick={onClick}
    style={color ? { borderColor: color } : undefined}
  >
    <span className={styles.metricLabel}>{label}</span>
    <span className={styles.metricValue} style={color ? { color } : undefined}>
      {value}
    </span>
    {subValue && <span className={styles.metricSubValue}>{subValue}</span>}
    {trend && trendValue && (
      <span className={`${styles.metricTrend} ${styles[trend]}`}>
        {trend === 'up' ? '↑' : '↓'} {trendValue}
      </span>
    )}
  </div>
);

export default MetricCard;
