import React, { useEffect, useState, useCallback } from 'react';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import type { BehavioralStrategy, ScannerStatus } from '../../../../types/arbfarm';
import styles from '../arbfarm.module.scss';

interface IBehavioralStrategiesPanelProps {
  compact?: boolean;
}

const BehavioralStrategiesPanel: React.FC<IBehavioralStrategiesPanelProps> = ({
  compact = false,
}) => {
  const [strategies, setStrategies] = useState<BehavioralStrategy[]>([]);
  const [scannerStatus, setScannerStatus] = useState<ScannerStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [toggling, setToggling] = useState<string | null>(null);
  const [isScannerToggling, setIsScannerToggling] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    try {
      const [strategiesRes, scannerRes] = await Promise.all([
        arbFarmService.listBehavioralStrategies(),
        arbFarmService.getScannerStatus(),
      ]);

      if (strategiesRes.success && strategiesRes.data) {
        setStrategies(strategiesRes.data.strategies || []);
      }

      if (scannerRes.success && scannerRes.data) {
        setScannerStatus(scannerRes.data);
      }

      setError(null);
    } catch (err) {
      console.error('Failed to fetch behavioral strategies:', err);
      setError('Failed to load strategies');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void fetchData();
    const interval = setInterval(() => void fetchData(), 10000);

    return () => clearInterval(interval);
  }, [fetchData]);

  const handleToggleStrategy = async (name: string, currentActive: boolean) => {
    setToggling(name);
    try {
      const res = await arbFarmService.toggleBehavioralStrategy(name, !currentActive);

      if (res.success) {
        setStrategies((prev) =>
          prev.map((s) => (s.name === name ? { ...s, is_active: !currentActive } : s)),
        );
      } else {
        setError(res.error || 'Failed to toggle strategy');
      }
    } catch (err) {
      setError('Failed to toggle strategy');
    } finally {
      setToggling(null);
    }
  };

  const handleToggleScanner = async () => {
    if (!scannerStatus) {
      return;
    }

    setIsScannerToggling(true);
    try {
      const res = scannerStatus.is_running
        ? await arbFarmService.stopScanner()
        : await arbFarmService.startScanner();

      if (res.success) {
        setScannerStatus((prev) => (prev ? { ...prev, is_running: !prev.is_running } : prev));
      } else {
        setError(res.error || 'Failed to toggle scanner');
      }
    } catch (err) {
      setError('Failed to toggle scanner');
    } finally {
      setIsScannerToggling(false);
    }
  };

  const handleToggleAll = async (active: boolean) => {
    setToggling('all');
    try {
      const res = await arbFarmService.toggleAllBehavioralStrategies(active);

      if (res.success) {
        setStrategies((prev) => prev.map((s) => ({ ...s, is_active: active })));
      } else {
        setError(res.error || 'Failed to toggle all strategies');
      }
    } catch (err) {
      setError('Failed to toggle all strategies');
    } finally {
      setToggling(null);
    }
  };

  const getStrategyIcon = (strategyType: string): string => {
    switch (strategyType) {
      case 'copy_trade':
        return '👥';
      case 'volume_hunter':
        return '📈';
      case 'graduation_sniper':
        return '🎯';
      default:
        return '⚡';
    }
  };

  const getVenueLabel = (venue: string): string => {
    if (venue.includes('BondingCurve')) {
      return 'Curves';
    }

    if (venue.includes('DexAmm')) {
      return 'DEX';
    }

    return venue;
  };

  const activeCount = strategies.filter((s) => s.is_active).length;
  const totalCount = strategies.length;

  if (isLoading) {
    return (
      <div className={styles.activityCard}>
        <div className={styles.activityCardHeader}>
          <h3 className={styles.activityCardTitle}>📊 Behavioral Strategies</h3>
        </div>
        <div className={styles.activityCardContent}>
          <div className={styles.emptyState}>Loading strategies...</div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.activityCard}>
      <div className={styles.activityCardHeader}>
        <h3 className={styles.activityCardTitle}>📊 Behavioral Strategies</h3>
        <span
          className={`${styles.activityBadge} ${activeCount > 0 ? styles.badgeActive : styles.badgeInactive}`}
        >
          {activeCount}/{totalCount} active
        </span>
      </div>
      <div className={styles.activityCardContent}>
        {error && (
          <div className={styles.errorBanner}>
            {error}
            <button type="button" onClick={() => setError(null)} className={styles.dismissBtn}>
              ×
            </button>
          </div>
        )}

        {/* Scanner Master Control */}
        <div className={styles.scannerControl}>
          <div className={styles.scannerInfo}>
            <span
              className={scannerStatus?.is_running ? styles.statusActive : styles.statusInactive}
            >
              {scannerStatus?.is_running ? '🟢 Scanner Running' : '🔴 Scanner Stopped'}
            </span>
            {scannerStatus?.is_running && (
              <span className={styles.scannerStats}>
                {scannerStatus.stats?.total_signals || 0} signals detected
              </span>
            )}
          </div>
          <button
            type="button"
            onClick={() => void handleToggleScanner()}
            disabled={isScannerToggling}
            className={scannerStatus?.is_running ? styles.btnDanger : styles.btnSuccess}
          >
            {isScannerToggling
              ? 'Processing...'
              : scannerStatus?.is_running
                ? 'Stop Scanner'
                : 'Start Scanner'}
          </button>
        </div>

        {!scannerStatus?.is_running && (
          <div className={styles.warningBanner}>
            ⚠️ Scanner is stopped. No strategies will execute until scanner is started.
          </div>
        )}

        {/* Bulk Actions */}
        <div className={styles.bulkActions}>
          <button
            type="button"
            onClick={() => void handleToggleAll(true)}
            disabled={toggling === 'all' || activeCount === totalCount}
            className={styles.btnSecondary}
          >
            Enable All
          </button>
          <button
            type="button"
            onClick={() => void handleToggleAll(false)}
            disabled={toggling === 'all' || activeCount === 0}
            className={styles.btnSecondary}
          >
            Disable All
          </button>
        </div>

        {/* Strategy List */}
        <div className={styles.strategyList}>
          {strategies.length === 0 ? (
            <div className={styles.emptyState}>No behavioral strategies registered</div>
          ) : (
            strategies.map((strategy) => (
              <div
                key={strategy.name}
                className={`${styles.strategyItem} ${strategy.is_active ? styles.active : styles.inactive}`}
              >
                <div className={styles.strategyInfo}>
                  <span className={styles.strategyIcon}>
                    {getStrategyIcon(strategy.strategy_type)}
                  </span>
                  <div className={styles.strategyDetails}>
                    <span className={styles.strategyName}>{strategy.name}</span>
                    <span className={styles.strategyMeta}>
                      {strategy.strategy_type} •{' '}
                      {strategy.supported_venues.map(getVenueLabel).join(', ')}
                    </span>
                  </div>
                </div>
                <div className={styles.strategyActions}>
                  <span
                    className={
                      strategy.is_active ? styles.statusBadgeActive : styles.statusBadgeInactive
                    }
                  >
                    {strategy.is_active ? 'Active' : 'Inactive'}
                  </span>
                  <button
                    type="button"
                    onClick={() => void handleToggleStrategy(strategy.name, strategy.is_active)}
                    disabled={toggling === strategy.name}
                    className={strategy.is_active ? styles.btnDanger : styles.btnSuccess}
                  >
                    {toggling === strategy.name ? '...' : strategy.is_active ? 'Disable' : 'Enable'}
                  </button>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Info Box */}
        {!compact && (
          <div className={styles.infoBox}>
            <strong>How it works:</strong>
            <ul>
              <li>
                <strong>KOL Copy Trading</strong>: Copies trades from tracked KOL wallets
              </li>
              <li>
                <strong>Volume Hunter</strong>: Detects volume spikes on graduation curves
              </li>
              <li>
                <strong>Graduation Sniper</strong>: Snipes tokens near 85%+ graduation
              </li>
            </ul>
            <p>
              All strategies share capital equally. Stopping the scanner stops all strategy
              execution.
            </p>
          </div>
        )}
      </div>
    </div>
  );
};

export default BehavioralStrategiesPanel;
