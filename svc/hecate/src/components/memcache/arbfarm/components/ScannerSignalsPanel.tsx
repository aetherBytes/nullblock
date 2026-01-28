import React, { useState, useEffect, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import type { Signal, ScannerStatus } from '../../../../types/arbfarm';

interface ScannerSignalsPanelProps {
  onSignalClick?: (signal: Signal) => void;
  maxSignals?: number;
}

const ScannerSignalsPanel: React.FC<ScannerSignalsPanelProps> = ({
  onSignalClick,
  maxSignals = 8,
}) => {
  const [signals, setSignals] = useState<Signal[]>([]);
  const [status, setStatus] = useState<ScannerStatus | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchData = useCallback(async () => {
    try {
      const [signalsRes, statusRes] = await Promise.all([
        arbFarmService.getSignals({ limit: maxSignals }),
        arbFarmService.getScannerStatus(),
      ]);

      if (signalsRes.success && signalsRes.data) {
        const signalData = signalsRes.data as { signals?: Signal[]; count?: number };
        setSignals(signalData.signals || []);
      }
      if (statusRes.success && statusRes.data) {
        setStatus(statusRes.data);
      }
    } catch (e) {
      console.error('Scanner data fetch error:', e);
    } finally {
      setLoading(false);
    }
  }, [maxSignals]);

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 15000);
    return () => clearInterval(interval);
  }, [fetchData]);

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) {
      const mins = Math.floor(diff / 60000);
      return `${mins}m ago`;
    }
    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
  };

  const getSignalIcon = (signal: Signal): string => {
    const source = (signal.metadata as Record<string, unknown>)?.signal_source as string | undefined;
    if (source) {
      if (source.includes('graduation_sniper')) return '🎓';
      if (source.includes('volume_hunter')) return '📊';
    }
    const type = signal.signal_type.toLowerCase();
    if (type.includes('graduation')) return '🎓';
    if (type.includes('volume')) return '📊';
    if (type.includes('momentum')) return '🚀';
    if (type.includes('price')) return '💰';
    if (type.includes('liquidity')) return '💧';
    if (type.includes('new_token')) return '✨';
    return '📡';
  };

  const getSignalSource = (signal: Signal): string | null => {
    const source = (signal.metadata as Record<string, unknown>)?.signal_source as string | undefined;
    if (!source) return null;
    return source.replace(/_/g, ' ');
  };

  const getSignificanceColor = (confidence: number): string => {
    if (confidence >= 80) return styles.sigCritical;
    if (confidence >= 60) return styles.sigHigh;
    if (confidence >= 40) return styles.sigMedium;
    return styles.sigLow;
  };

  const isActive = status?.is_running;

  if (loading) {
    return (
      <div className={styles.activityCard}>
        <div className={styles.activityCardHeader}>
          <h3 className={styles.activityCardTitle}>Scanner Signals</h3>
        </div>
        <div className={styles.activityCardContent}>
          <div className={styles.activityEmptyState}>
            <span className={styles.logSpinner}>◠</span>
            <span>Loading signals...</span>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.activityCard}>
      <div className={styles.activityCardHeader}>
        <h3 className={styles.activityCardTitle}>
          {isActive && <span className={styles.liveIndicator} />}
          Scanner Signals
        </h3>
        <span className={`${styles.activityBadge} ${isActive ? styles.badgeActive : styles.badgeInactive}`}>
          {isActive ? 'Active' : 'Paused'}
        </span>
      </div>

      <div className={styles.activityCardContent}>
        {!isActive && signals.length === 0 ? (
          <div className={styles.activityEmptyState}>
            <span className={styles.activityEmptyIcon}>📡</span>
            <span>Scanner is paused</span>
            <span className={styles.activityEmptyHint}>Start scanner to detect opportunities</span>
          </div>
        ) : signals.length === 0 ? (
          <div className={styles.activityEmptyState}>
            <span className={styles.activityEmptyIcon}>🔍</span>
            <span>No signals detected</span>
            <span className={styles.activityEmptyHint}>Scanner is active and monitoring</span>
          </div>
        ) : (
          <div className={styles.signalsStream}>
            {signals.map((signal) => (
              <div
                key={signal.id}
                className={styles.signalItem}
                onClick={() => onSignalClick?.(signal)}
              >
                <div className={styles.signalIcon}>
                  <span>{getSignalIcon(signal)}</span>
                </div>
                <div className={styles.signalContent}>
                  <div className={styles.signalHeader}>
                    <span className={styles.signalType}>
                      {signal.signal_type.replace(/_/g, ' ')}
                    </span>
                    {getSignalSource(signal) && (
                      <span className={styles.signalSourceBadge}>
                        {getSignalSource(signal)}
                      </span>
                    )}
                    <span className={styles.signalTime}>{formatTime(signal.detected_at)}</span>
                  </div>
                  <div className={styles.signalDetails}>
                    {signal.token_mint && (
                      <span className={styles.signalMint}>
                        {signal.token_mint.slice(0, 6)}...
                      </span>
                    )}
                    <span className={`${styles.signalConfidence} ${getSignificanceColor(signal.confidence)}`}>
                      {signal.confidence}% conf
                    </span>
                    {signal.estimated_profit_bps > 0 && (
                      <span className={styles.signalProfit}>
                        +{(signal.estimated_profit_bps / 100).toFixed(1)}%
                      </span>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default ScannerSignalsPanel;
