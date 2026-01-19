import React, { useState, useEffect } from 'react';
import type { Strategy, CurveStrategyParams, CurveStrategyMode } from '../../../../types/arbfarm';
import { CURVE_STRATEGY_MODE_LABELS } from '../../../../types/arbfarm';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import styles from '../arbfarm.module.scss';

interface CurveStrategyConfigModalProps {
  strategy: Strategy;
  curveParams: CurveStrategyParams;
  onClose: () => void;
  onSuccess: (message: string) => void;
  onError: (message: string) => void;
  onUpdate: () => void;
}

const STRATEGY_MODE_OPTIONS: { value: CurveStrategyMode; label: string; description: string }[] = [
  { value: 'graduation_sniper', label: 'Graduation Sniper', description: 'Target tokens approaching graduation' },
  { value: 'volume_rider', label: 'Volume Rider', description: 'Follow high volume momentum' },
  { value: 'early_entry', label: 'Early Entry', description: 'Enter tokens early in lifecycle' },
  { value: 'fast_snipe', label: 'Fast Snipe', description: 'Aggressive quick entries' },
];

const CurveStrategyConfigModal: React.FC<CurveStrategyConfigModalProps> = ({
  strategy,
  curveParams,
  onClose,
  onSuccess,
  onError,
  onUpdate,
}) => {
  const [params, setParams] = useState<CurveStrategyParams>(curveParams);
  const [saving, setSaving] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    const changed = JSON.stringify(params) !== JSON.stringify(curveParams);
    setHasChanges(changed);
  }, [params, curveParams]);

  const handleSave = async () => {
    setSaving(true);
    try {
      const response = await arbFarmService.updateCurveStrategyParams(strategy.id, params);
      if (response.success) {
        onSuccess('Strategy configuration updated successfully');
        onUpdate();
        onClose();
      } else {
        onError(response.error || 'Failed to update strategy configuration');
      }
    } catch (e) {
      onError(`Failed to save: ${e instanceof Error ? e.message : 'Unknown error'}`);
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    setParams(curveParams);
  };

  const updateParam = <K extends keyof CurveStrategyParams>(key: K, value: CurveStrategyParams[K]) => {
    setParams(prev => ({ ...prev, [key]: value }));
  };

  return (
    <div className={styles.opportunityModal} onClick={onClose}>
      <div className={styles.opportunityModalContent} onClick={(e) => e.stopPropagation()}>
        <button className={styles.closeButton} onClick={onClose}>×</button>

        <div className={styles.configModalHeader}>
          <h2>Configure Strategy</h2>
          <span className={styles.strategyNameLabel}>{strategy.name}</span>
        </div>

        <div className={styles.configSection}>
          <h3>Strategy Mode</h3>
          <select
            className={styles.configSelect}
            value={params.mode}
            onChange={(e) => updateParam('mode', e.target.value as CurveStrategyMode)}
          >
            {STRATEGY_MODE_OPTIONS.map(opt => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
          <span className={styles.configHint}>
            {STRATEGY_MODE_OPTIONS.find(o => o.value === params.mode)?.description}
          </span>
        </div>

        <div className={styles.configSection}>
          <h3>Graduation Progress Range</h3>
          <div className={styles.rangeInputGroup}>
            <div className={styles.rangeInput}>
              <label>Minimum %</label>
              <input
                type="range"
                min="30"
                max="95"
                step="1"
                value={params.min_graduation_progress}
                onChange={(e) => updateParam('min_graduation_progress', parseFloat(e.target.value))}
              />
              <span className={styles.rangeValue}>{params.min_graduation_progress}%</span>
            </div>
            <div className={styles.rangeInput}>
              <label>Maximum %</label>
              <input
                type="range"
                min="80"
                max="99.5"
                step="0.5"
                value={params.max_graduation_progress}
                onChange={(e) => updateParam('max_graduation_progress', parseFloat(e.target.value))}
              />
              <span className={styles.rangeValue}>{params.max_graduation_progress}%</span>
            </div>
          </div>
          <span className={styles.configHint}>
            Only target tokens within this graduation progress range
          </span>
        </div>

        <div className={styles.configSection}>
          <h3>Entry Configuration</h3>
          <div className={styles.configRow}>
            <label>Entry Size (SOL)</label>
            <input
              type="number"
              step="0.01"
              min="0.01"
              max="5.0"
              value={params.entry_sol_amount}
              onChange={(e) => updateParam('entry_sol_amount', parseFloat(e.target.value) || 0.1)}
              className={styles.configInput}
            />
          </div>
          <span className={styles.configHint}>
            Amount of SOL to invest per entry
          </span>
        </div>

        <div className={styles.configSection}>
          <h3>Exit Configuration</h3>
          <div className={styles.configRow}>
            <label>Exit on Graduation</label>
            <button
              className={`${styles.toggleSwitch} ${params.exit_on_graduation ? styles.toggleOn : ''}`}
              onClick={() => updateParam('exit_on_graduation', !params.exit_on_graduation)}
            >
              {params.exit_on_graduation ? 'ON' : 'OFF'}
            </button>
          </div>
          <div className={styles.configRow}>
            <label>Sell Delay (ms)</label>
            <input
              type="number"
              step="100"
              min="0"
              max="10000"
              value={params.graduation_sell_delay_ms}
              onChange={(e) => updateParam('graduation_sell_delay_ms', parseInt(e.target.value) || 0)}
              className={styles.configInput}
              disabled={!params.exit_on_graduation}
            />
          </div>
          <span className={styles.configHint}>
            {params.exit_on_graduation
              ? `Sell ${params.graduation_sell_delay_ms}ms after graduation detected`
              : 'Manual exit required'}
          </span>
        </div>

        <div className={styles.configSection}>
          <h3>Filtering Criteria</h3>
          <div className={styles.configRow}>
            <label>Min Volume (24h SOL)</label>
            <input
              type="number"
              step="1"
              min="0"
              value={params.min_volume_24h_sol}
              onChange={(e) => updateParam('min_volume_24h_sol', parseFloat(e.target.value) || 0)}
              className={styles.configInput}
            />
          </div>
          <div className={styles.configRow}>
            <label>Min Holders</label>
            <input
              type="number"
              step="10"
              min="0"
              value={params.min_holder_count}
              onChange={(e) => updateParam('min_holder_count', parseInt(e.target.value) || 0)}
              className={styles.configInput}
            />
          </div>
          <div className={styles.configRow}>
            <label>Max Top-10 Concentration %</label>
            <input
              type="number"
              step="1"
              min="0"
              max="100"
              value={params.max_holder_concentration}
              onChange={(e) => updateParam('max_holder_concentration', parseFloat(e.target.value) || 100)}
              className={styles.configInput}
            />
          </div>
          {params.min_score !== undefined && (
            <div className={styles.configRow}>
              <label>Min Opportunity Score</label>
              <input
                type="number"
                step="5"
                min="0"
                max="100"
                value={params.min_score ?? 0}
                onChange={(e) => updateParam('min_score', parseInt(e.target.value) || 0)}
                className={styles.configInput}
              />
            </div>
          )}
        </div>

        <div className={styles.configActions}>
          <button
            className={styles.configCancelButton}
            onClick={onClose}
            disabled={saving}
          >
            Cancel
          </button>
          <button
            className={styles.configResetButton}
            onClick={handleReset}
            disabled={saving || !hasChanges}
          >
            Reset
          </button>
          <button
            className={styles.configSaveButton}
            onClick={handleSave}
            disabled={saving || !hasChanges}
          >
            {saving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default CurveStrategyConfigModal;
