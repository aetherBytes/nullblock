import React from 'react';
import type { Trade } from '../../../../types/arbfarm';
import styles from '../arbfarm.module.scss';

interface TradeHistoryCardProps {
  trade: Trade;
  compact?: boolean;
}

const TradeHistoryCard: React.FC<TradeHistoryCardProps> = ({ trade, compact = false }) => {
  const isProfit = (trade.profit_lamports ?? 0) > 0;
  const profitSol = (trade.profit_lamports ?? 0) / 1_000_000_000;
  const gasSol = (trade.gas_cost_lamports ?? 0) / 1_000_000_000;
  const netProfit = profitSol - gasSol;

  const formatSol = (sol: number): string => `${sol >= 0 ? '+' : ''}${sol.toFixed(4)} SOL`;

  if (compact) {
    return (
      <div className={`${styles.tradeCardCompact} ${isProfit ? styles.win : styles.loss}`}>
        <span className={styles.tradeIndicator}>{isProfit ? '✓' : '✗'}</span>
        <span className={`${styles.tradeProfit} ${isProfit ? styles.positive : styles.negative}`}>
          {formatSol(netProfit)}
        </span>
        <span className={styles.tradeSlippage}>{trade.slippage_bps}bps slip</span>
        <span className={styles.tradeTime}>{new Date(trade.executed_at).toLocaleTimeString()}</span>
      </div>
    );
  }

  return (
    <div className={`${styles.tradeCard} ${isProfit ? styles.win : styles.loss}`}>
      <div className={styles.tradeHeader}>
        <span className={styles.tradeIndicator}>{isProfit ? '✓ Win' : '✗ Loss'}</span>
        <span className={styles.tradeId}>{trade.id.slice(0, 8)}...</span>
      </div>

      <div className={styles.tradeMetrics}>
        <div className={styles.metric}>
          <span className={styles.metricLabel}>Gross Profit</span>
          <span className={`${styles.metricValue} ${isProfit ? styles.positive : styles.negative}`}>
            {formatSol(profitSol)}
          </span>
        </div>
        <div className={styles.metric}>
          <span className={styles.metricLabel}>Gas Cost</span>
          <span className={styles.metricValue}>-{gasSol.toFixed(4)} SOL</span>
        </div>
        <div className={styles.metric}>
          <span className={styles.metricLabel}>Net P&L</span>
          <span
            className={`${styles.metricValue} ${netProfit >= 0 ? styles.positive : styles.negative}`}
          >
            {formatSol(netProfit)}
          </span>
        </div>
        <div className={styles.metric}>
          <span className={styles.metricLabel}>Slippage</span>
          <span className={styles.metricValue}>{trade.slippage_bps} bps</span>
        </div>
      </div>

      <div className={styles.tradePrices}>
        <span>Entry: {trade.entry_price?.toFixed(6)}</span>
        {trade.exit_price && <span>Exit: {trade.exit_price.toFixed(6)}</span>}
      </div>

      {trade.tx_signature && (
        <div className={styles.tradeTx}>
          <a
            href={`https://solscan.io/tx/${trade.tx_signature}`}
            target="_blank"
            rel="noopener noreferrer"
            className={styles.txLink}
          >
            View on Solscan ↗
          </a>
        </div>
      )}

      <div className={styles.tradeFooter}>
        <span className={styles.tradeTime}>{new Date(trade.executed_at).toLocaleString()}</span>
      </div>
    </div>
  );
};

export default TradeHistoryCard;
