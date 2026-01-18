import React, { useState } from 'react';
import type { CurvePosition, CurveExitConfig } from '../../../../types/arbfarm';
import GraduationProgressBar from './GraduationProgressBar';
import styles from '../arbfarm.module.scss';

interface CurvePositionCardProps {
  position: CurvePosition;
  onClose: (positionId: string, percent?: number) => void;
  onUpdateExit: (positionId: string, config: Partial<CurveExitConfig>) => void;
}

const CurvePositionCard: React.FC<CurvePositionCardProps> = ({
  position,
  onClose,
  onUpdateExit,
}) => {
  const [showEditExit, setShowEditExit] = useState(false);
  const [exitConfig, setExitConfig] = useState<Partial<CurveExitConfig>>({
    sell_on_graduation: position.exit_config.sell_on_graduation,
    graduation_sell_delay_ms: position.exit_config.graduation_sell_delay_ms,
    stop_loss_percent: position.exit_config.stop_loss_percent,
    take_profit_percent: position.exit_config.take_profit_percent,
  });

  const getPnlClass = (pnl: number): string => {
    if (pnl > 0) return styles.pnlPositive;
    if (pnl < 0) return styles.pnlNegative;
    return styles.pnlNeutral;
  };

  const getStatusBadge = (status: string): { text: string; className: string } => {
    switch (status) {
      case 'open':
        return { text: 'Open', className: styles.statusOpen };
      case 'pending_exit':
        return { text: 'Exiting', className: styles.statusPending };
      case 'closed':
        return { text: 'Closed', className: styles.statusClosed };
      default:
        return { text: status, className: '' };
    }
  };

  const statusBadge = getStatusBadge(position.status);

  const handleSaveExit = () => {
    onUpdateExit(position.id, exitConfig);
    setShowEditExit(false);
  };

  return (
    <div className={styles.curvePositionCard}>
      <div className={styles.positionHeader}>
        <div className={styles.positionToken}>
          <span className={styles.tokenSymbol}>${position.token_symbol}</span>
          <span className={`${styles.statusBadge} ${statusBadge.className}`}>
            {statusBadge.text}
          </span>
        </div>
        <div className={`${styles.positionPnl} ${getPnlClass(position.unrealized_pnl_percent)}`}>
          {position.unrealized_pnl_percent >= 0 ? '+' : ''}
          {position.unrealized_pnl_percent.toFixed(2)}%
        </div>
      </div>

      <div className={styles.positionProgress}>
        <div className={styles.progressLabels}>
          <span>Entry: {position.entry_progress.toFixed(1)}%</span>
          <span>Now: {position.current_progress.toFixed(1)}%</span>
        </div>
        <GraduationProgressBar progress={position.current_progress} showLabel={false} size="small" />
      </div>

      <div className={styles.positionMetrics}>
        <div className={styles.positionMetric}>
          <span className={styles.metricLabel}>Entry</span>
          <span className={styles.metricValue}>{position.entry_sol.toFixed(4)} SOL</span>
        </div>
        <div className={styles.positionMetric}>
          <span className={styles.metricLabel}>Tokens</span>
          <span className={styles.metricValue}>{position.entry_tokens.toLocaleString()}</span>
        </div>
        <div className={styles.positionMetric}>
          <span className={styles.metricLabel}>Venue</span>
          <span className={styles.metricValue}>{position.venue.replace('_', '.')}</span>
        </div>
      </div>

      <div className={styles.exitConfigDisplay}>
        <span className={styles.exitLabel}>Exit:</span>
        {position.exit_config.sell_on_graduation && (
          <span className={styles.exitBadge}>On Graduation</span>
        )}
        {position.exit_config.stop_loss_percent && (
          <span className={styles.exitBadge}>SL {position.exit_config.stop_loss_percent}%</span>
        )}
        {position.exit_config.take_profit_percent && (
          <span className={styles.exitBadge}>TP {position.exit_config.take_profit_percent}%</span>
        )}
        <button
          className={styles.editExitButton}
          onClick={() => setShowEditExit(!showEditExit)}
        >
          Edit
        </button>
      </div>

      {showEditExit && (
        <div className={styles.exitConfigEditor}>
          <div className={styles.configRow}>
            <label>
              <input
                type="checkbox"
                checked={exitConfig.sell_on_graduation}
                onChange={(e) =>
                  setExitConfig({ ...exitConfig, sell_on_graduation: e.target.checked })
                }
              />
              Sell on Graduation
            </label>
          </div>
          <div className={styles.configRow}>
            <label>Sell Delay (ms)</label>
            <input
              type="number"
              value={exitConfig.graduation_sell_delay_ms}
              onChange={(e) =>
                setExitConfig({ ...exitConfig, graduation_sell_delay_ms: parseInt(e.target.value) })
              }
            />
          </div>
          <div className={styles.configRow}>
            <label>Stop Loss %</label>
            <input
              type="number"
              value={exitConfig.stop_loss_percent || ''}
              placeholder="None"
              onChange={(e) =>
                setExitConfig({
                  ...exitConfig,
                  stop_loss_percent: e.target.value ? parseFloat(e.target.value) : undefined,
                })
              }
            />
          </div>
          <div className={styles.configRow}>
            <label>Take Profit %</label>
            <input
              type="number"
              value={exitConfig.take_profit_percent || ''}
              placeholder="None"
              onChange={(e) =>
                setExitConfig({
                  ...exitConfig,
                  take_profit_percent: e.target.value ? parseFloat(e.target.value) : undefined,
                })
              }
            />
          </div>
          <div className={styles.configActions}>
            <button className={styles.saveButton} onClick={handleSaveExit}>
              Save
            </button>
            <button className={styles.cancelButton} onClick={() => setShowEditExit(false)}>
              Cancel
            </button>
          </div>
        </div>
      )}

      {position.status === 'open' && (
        <div className={styles.positionActions}>
          <button className={styles.sellPartialButton} onClick={() => onClose(position.id, 50)}>
            Sell 50%
          </button>
          <button className={styles.sellAllButton} onClick={() => onClose(position.id, 100)}>
            Sell All
          </button>
        </div>
      )}

      <div className={styles.positionFooter}>
        <span className={styles.mintAddress} title={position.token_mint}>
          {position.token_mint.slice(0, 8)}...
        </span>
        <span className={styles.positionTime}>
          {new Date(position.created_at).toLocaleString()}
        </span>
      </div>
    </div>
  );
};

export default CurvePositionCard;
