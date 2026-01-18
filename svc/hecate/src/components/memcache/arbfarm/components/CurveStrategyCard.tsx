import React from 'react';
import type { Strategy, CurveStrategyStats, CurveStrategyParams } from '../../../../types/arbfarm';
import { CURVE_STRATEGY_MODE_LABELS } from '../../../../types/arbfarm';
import styles from '../arbfarm.module.scss';

type RiskProfile = 'conservative' | 'moderate' | 'aggressive';

interface CurveStrategyCardProps {
  strategy: Strategy;
  curveParams?: CurveStrategyParams;
  stats?: CurveStrategyStats;
  onToggle: (id: string, enabled: boolean) => void;
  onConfigure: (strategy: Strategy) => void;
  onViewStats: (id: string) => void;
  onChangeExecutionMode?: (id: string, mode: 'autonomous' | 'agent_directed' | 'hybrid') => void;
  onChangeRiskProfile?: (id: string, profile: RiskProfile) => void;
}

const EXECUTION_MODE_LABELS: Record<string, string> = {
  autonomous: '🤖 Auto',
  agent_directed: '👤 Manual',
  hybrid: '⚡ Hybrid',
  manual: '👤 Manual',
};

const EXECUTION_MODE_COLORS: Record<string, string> = {
  autonomous: '#22c55e',
  agent_directed: '#f59e0b',
  hybrid: '#3b82f6',
  manual: '#f59e0b',
};

const RISK_PROFILE_LABELS: Record<RiskProfile, string> = {
  conservative: '🛡️ Conservative',
  moderate: '⚖️ Moderate',
  aggressive: '🔥 Aggressive',
};

const RISK_PROFILE_COLORS: Record<RiskProfile, string> = {
  conservative: '#3b82f6',
  moderate: '#f59e0b',
  aggressive: '#ef4444',
};

const getRiskProfileFromParams = (params: Strategy['risk_params']): RiskProfile => {
  if (params.max_position_sol <= 0.2 && params.max_risk_score <= 40) return 'conservative';
  if (params.max_position_sol >= 1.5 || params.max_risk_score >= 70) return 'aggressive';
  return 'moderate';
};

const CurveStrategyCard: React.FC<CurveStrategyCardProps> = ({
  strategy,
  curveParams,
  stats,
  onToggle,
  onConfigure,
  onViewStats,
  onChangeExecutionMode,
  onChangeRiskProfile,
}) => {
  const currentRiskProfile = getRiskProfileFromParams(strategy.risk_params);
  const getStatusIndicator = (isActive: boolean): { color: string; label: string } => {
    return isActive
      ? { color: '#22c55e', label: 'ON' }
      : { color: '#6b7280', label: 'OFF' };
  };

  const status = getStatusIndicator(strategy.is_active);

  const getModeLabel = (mode?: string): string => {
    if (!mode) return 'Unknown';
    return CURVE_STRATEGY_MODE_LABELS[mode as keyof typeof CURVE_STRATEGY_MODE_LABELS] || mode;
  };

  const formatPnl = (pnl: number): string => {
    const sign = pnl >= 0 ? '+' : '';
    return `${sign}${pnl.toFixed(4)} SOL`;
  };

  const formatWinRate = (rate: number): string => {
    return `${(rate * 100).toFixed(1)}%`;
  };

  return (
    <div className={styles.curveStrategyCard}>
      <div className={styles.strategyHeader}>
        <div className={styles.strategyInfo}>
          <span className={styles.strategyName}>{strategy.name}</span>
          <span className={styles.strategyMode}>
            {getModeLabel(curveParams?.mode)}
          </span>
        </div>
        <div className={styles.strategyStatus}>
          <span
            className={styles.executionModeBadge}
            style={{
              backgroundColor: `${EXECUTION_MODE_COLORS[strategy.execution_mode] || '#6b7280'}20`,
              color: EXECUTION_MODE_COLORS[strategy.execution_mode] || '#6b7280',
              borderColor: EXECUTION_MODE_COLORS[strategy.execution_mode] || '#6b7280',
            }}
          >
            {EXECUTION_MODE_LABELS[strategy.execution_mode] || strategy.execution_mode}
          </span>
          <span className={styles.statusIndicator} style={{ backgroundColor: status.color }}>
            {status.label}
          </span>
        </div>
      </div>

      {curveParams && (
        <div className={styles.strategyParams}>
          <div className={styles.paramRow}>
            <span className={styles.paramLabel}>Progress Range</span>
            <span className={styles.paramValue}>
              {curveParams.min_graduation_progress}% - {curveParams.max_graduation_progress}%
            </span>
          </div>
          <div className={styles.paramRow}>
            <span className={styles.paramLabel}>Entry Size</span>
            <span className={styles.paramValue}>{curveParams.entry_sol_amount} SOL</span>
          </div>
          <div className={styles.paramRow}>
            <span className={styles.paramLabel}>Exit</span>
            <span className={styles.paramValue}>
              {curveParams.exit_on_graduation ? 'On Graduation' : 'Manual'}
              {curveParams.graduation_sell_delay_ms > 0 &&
                ` (+${curveParams.graduation_sell_delay_ms}ms)`}
            </span>
          </div>
        </div>
      )}

      <div className={styles.riskProfileSection}>
        <span className={styles.riskProfileLabel}>Risk Appetite:</span>
        {onChangeRiskProfile ? (
          <select
            className={styles.riskProfileSelect}
            value={currentRiskProfile}
            onChange={(e) => onChangeRiskProfile(strategy.id, e.target.value as RiskProfile)}
            style={{
              borderColor: RISK_PROFILE_COLORS[currentRiskProfile],
              color: RISK_PROFILE_COLORS[currentRiskProfile],
            }}
          >
            <option value="conservative">🛡️ Conservative</option>
            <option value="moderate">⚖️ Moderate</option>
            <option value="aggressive">🔥 Aggressive</option>
          </select>
        ) : (
          <span
            className={styles.riskProfileBadge}
            style={{
              backgroundColor: `${RISK_PROFILE_COLORS[currentRiskProfile]}20`,
              color: RISK_PROFILE_COLORS[currentRiskProfile],
              borderColor: RISK_PROFILE_COLORS[currentRiskProfile],
            }}
          >
            {RISK_PROFILE_LABELS[currentRiskProfile]}
          </span>
        )}
        <span className={styles.riskDetails}>
          Max: {strategy.risk_params.max_position_sol} SOL | SL: {strategy.risk_params.stop_loss_percent || 0}%
        </span>
      </div>

      {stats && (
        <div className={styles.strategyStats}>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Win Rate</span>
            <span className={styles.statValue}>{formatWinRate(stats.win_rate)}</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>P&L</span>
            <span
              className={`${styles.statValue} ${
                stats.total_pnl_sol >= 0 ? styles.positive : styles.negative
              }`}
            >
              {formatPnl(stats.total_pnl_sol)}
            </span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Entries</span>
            <span className={styles.statValue}>{stats.total_entries}</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Graduations</span>
            <span className={styles.statValue}>{stats.graduations_caught}</span>
          </div>
        </div>
      )}

      <div className={styles.strategyActions}>
        <button
          className={`${styles.toggleButton} ${strategy.is_active ? styles.active : ''}`}
          onClick={() => onToggle(strategy.id, !strategy.is_active)}
        >
          {strategy.is_active ? 'Disable' : 'Enable'}
        </button>
        {onChangeExecutionMode && (
          <button
            className={`${styles.autoModeButton} ${strategy.execution_mode === 'autonomous' ? styles.autoActive : ''}`}
            onClick={() =>
              onChangeExecutionMode(
                strategy.id,
                strategy.execution_mode === 'autonomous' ? 'agent_directed' : 'autonomous'
              )
            }
            title={
              strategy.execution_mode === 'autonomous'
                ? 'Switch to manual approval mode'
                : 'Switch to fully automatic execution'
            }
          >
            {strategy.execution_mode === 'autonomous' ? '👤 Manual' : '🤖 Auto'}
          </button>
        )}
        <button
          className={styles.configureButton}
          onClick={() => onConfigure(strategy)}
        >
          Configure
        </button>
        <button
          className={styles.statsButton}
          onClick={() => onViewStats(strategy.id)}
        >
          Stats
        </button>
      </div>

      <div className={styles.strategyFooter}>
        <span className={styles.strategyId}>{strategy.id.slice(0, 8)}...</span>
        <span className={styles.strategyUpdated}>
          Updated: {new Date(strategy.updated_at).toLocaleDateString()}
        </span>
      </div>
    </div>
  );
};

export default CurveStrategyCard;
