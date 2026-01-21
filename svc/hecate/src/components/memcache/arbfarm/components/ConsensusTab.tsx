import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import type {
  ConsensusConfig,
  ConsensusModelConfig,
  ConsensusConfigResponse,
  ConsensusStatsResponse,
} from '../../../../types/consensus';

const ConsensusTab: React.FC = () => {
  const [config, setConfig] = useState<ConsensusConfig | null>(null);
  const [availableModels, setAvailableModels] = useState<ConsensusModelConfig[]>([]);
  const [isDevWallet, setIsDevWallet] = useState(false);
  const [stats, setStats] = useState<ConsensusStatsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [showAddModel, setShowAddModel] = useState(false);
  const [selectedModelToAdd, setSelectedModelToAdd] = useState<string>('');
  const [newModelWeight, setNewModelWeight] = useState<number>(1.0);

  const fetchConfig = useCallback(async () => {
    try {
      const [configRes, statsRes] = await Promise.all([
        arbFarmService.getConsensusConfig(),
        arbFarmService.getConsensusStats(),
      ]);

      if (configRes.success && configRes.data) {
        setConfig(configRes.data.config);
        setAvailableModels(configRes.data.available_models);
        setIsDevWallet(configRes.data.is_dev_wallet);
      }

      if (statsRes.success && statsRes.data) {
        setStats(statsRes.data);
      }
    } catch (error) {
      console.error('Failed to fetch consensus config:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  const handleToggleEnabled = async () => {
    if (!config) return;
    setSaving(true);
    try {
      const res = await arbFarmService.updateConsensusConfig({
        enabled: !config.enabled,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to toggle consensus:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleToggleModel = async (modelId: string, enabled: boolean) => {
    if (!config) return;
    setSaving(true);
    try {
      const updatedModels = config.models.map(m =>
        m.model_id === modelId ? { ...m, enabled } : m
      );
      const res = await arbFarmService.updateConsensusConfig({
        models: updatedModels,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to toggle model:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleUpdateWeight = async (modelId: string, weight: number) => {
    if (!config) return;
    setSaving(true);
    try {
      const updatedModels = config.models.map(m =>
        m.model_id === modelId ? { ...m, weight } : m
      );
      const res = await arbFarmService.updateConsensusConfig({
        models: updatedModels,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to update weight:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleRemoveModel = async (modelId: string) => {
    if (!config) return;
    setSaving(true);
    try {
      const updatedModels = config.models.filter(m => m.model_id !== modelId);
      const res = await arbFarmService.updateConsensusConfig({
        models: updatedModels,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to remove model:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleAddModel = async () => {
    if (!config || !selectedModelToAdd) return;
    const modelToAdd = availableModels.find(m => m.model_id === selectedModelToAdd);
    if (!modelToAdd) return;

    setSaving(true);
    try {
      const updatedModels = [
        ...config.models,
        { ...modelToAdd, weight: newModelWeight, enabled: true },
      ];
      const res = await arbFarmService.updateConsensusConfig({
        models: updatedModels,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
        setShowAddModel(false);
        setSelectedModelToAdd('');
        setNewModelWeight(1.0);
      }
    } catch (error) {
      console.error('Failed to add model:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleUpdateThreshold = async (threshold: number) => {
    if (!config) return;
    setSaving(true);
    try {
      const res = await arbFarmService.updateConsensusConfig({
        min_consensus_threshold: threshold,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to update threshold:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleUpdateReviewInterval = async (hours: number) => {
    if (!config) return;
    setSaving(true);
    try {
      const res = await arbFarmService.updateConsensusConfig({
        review_interval_hours: hours,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to update review interval:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleToggleAutoApply = async () => {
    if (!config) return;
    setSaving(true);
    try {
      const res = await arbFarmService.updateConsensusConfig({
        auto_apply_recommendations: !config.auto_apply_recommendations,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to toggle auto apply:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleResetConfig = async () => {
    setSaving(true);
    try {
      const res = await arbFarmService.resetConsensusConfig();
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to reset config:', error);
    } finally {
      setSaving(false);
    }
  };

  const availableModelsToAdd = availableModels.filter(
    am => !config?.models.some(m => m.model_id === am.model_id)
  );

  if (loading) {
    return (
      <div className={styles.loadingContainer}>
        <div className={styles.spinner} />
        <span>Loading consensus configuration...</span>
      </div>
    );
  }

  return (
    <div className={styles.consensusTab}>
      <div className={styles.sectionHeader}>
        <h3>LLM Consensus Configuration</h3>
        {isDevWallet && <span className={styles.devBadge}>Dev Wallet</span>}
      </div>

      {stats && (
        <div className={styles.statsRow}>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Total Decisions</span>
            <span className={styles.statValue}>{stats.total_decisions}</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Approved</span>
            <span className={`${styles.statValue} ${styles.positive}`}>{stats.approved_count}</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Rejected</span>
            <span className={`${styles.statValue} ${styles.negative}`}>{stats.rejected_count}</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Avg Agreement</span>
            <span className={styles.statValue}>{(stats.average_agreement * 100).toFixed(1)}%</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Last 24h</span>
            <span className={styles.statValue}>{stats.decisions_last_24h}</span>
          </div>
        </div>
      )}

      <div className={styles.configSection}>
        <div className={styles.configRow}>
          <label className={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config?.enabled ?? false}
              onChange={handleToggleEnabled}
              disabled={saving}
            />
            Enable Consensus Engine
          </label>

          <div className={styles.configField}>
            <label>Review Interval</label>
            <select
              value={config?.review_interval_hours ?? 1}
              onChange={e => handleUpdateReviewInterval(parseInt(e.target.value))}
              disabled={saving}
            >
              <option value={1}>1 hour</option>
              <option value={2}>2 hours</option>
              <option value={4}>4 hours</option>
              <option value={6}>6 hours</option>
              <option value={12}>12 hours</option>
              <option value={24}>24 hours</option>
            </select>
          </div>
        </div>

        <div className={styles.configRow}>
          <label className={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={config?.auto_apply_recommendations ?? false}
              onChange={handleToggleAutoApply}
              disabled={saving}
            />
            Auto-apply Recommendations
          </label>

          <div className={styles.configField}>
            <label>Min Threshold</label>
            <input
              type="number"
              min={0}
              max={1}
              step={0.1}
              value={config?.min_consensus_threshold ?? 0.6}
              onChange={e => handleUpdateThreshold(parseFloat(e.target.value))}
              disabled={saving}
              className={styles.smallInput}
            />
          </div>
        </div>
      </div>

      <div className={styles.modelsSection}>
        <div className={styles.sectionHeader}>
          <h4>Active Models</h4>
          <button
            className={styles.addButton}
            onClick={() => setShowAddModel(true)}
            disabled={availableModelsToAdd.length === 0}
          >
            + Add Model
          </button>
        </div>

        <div className={styles.modelsList}>
          {config?.models.map(model => (
            <div key={model.model_id} className={styles.modelItem}>
              <div className={styles.modelInfo}>
                <span className={styles.modelName}>{model.display_name}</span>
                <span className={styles.modelProvider}>{model.provider}</span>
              </div>
              <div className={styles.modelControls}>
                <label className={styles.weightLabel}>
                  Weight:
                  <input
                    type="number"
                    min={0.1}
                    max={3}
                    step={0.1}
                    value={model.weight}
                    onChange={e => handleUpdateWeight(model.model_id, parseFloat(e.target.value))}
                    disabled={saving}
                    className={styles.weightInput}
                  />
                </label>
                <label className={styles.checkboxLabel}>
                  <input
                    type="checkbox"
                    checked={model.enabled}
                    onChange={e => handleToggleModel(model.model_id, e.target.checked)}
                    disabled={saving}
                  />
                  Enabled
                </label>
                <button
                  className={styles.removeButton}
                  onClick={() => handleRemoveModel(model.model_id)}
                  disabled={saving || config.models.length <= 1}
                  title="Remove model"
                >
                  &times;
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {showAddModel && (
        <div className={styles.addModelModal}>
          <div className={styles.modalContent}>
            <h4>Add Model</h4>
            <div className={styles.modalField}>
              <label>Select Model</label>
              <select
                value={selectedModelToAdd}
                onChange={e => setSelectedModelToAdd(e.target.value)}
              >
                <option value="">-- Select --</option>
                {availableModelsToAdd.map(m => (
                  <option key={m.model_id} value={m.model_id}>
                    {m.display_name} ({m.provider})
                  </option>
                ))}
              </select>
            </div>
            <div className={styles.modalField}>
              <label>Weight</label>
              <input
                type="number"
                min={0.1}
                max={3}
                step={0.1}
                value={newModelWeight}
                onChange={e => setNewModelWeight(parseFloat(e.target.value))}
              />
            </div>
            <div className={styles.modalActions}>
              <button onClick={() => setShowAddModel(false)}>Cancel</button>
              <button
                onClick={handleAddModel}
                disabled={!selectedModelToAdd || saving}
                className={styles.primaryButton}
              >
                Add
              </button>
            </div>
          </div>
        </div>
      )}

      <div className={styles.actionsSection}>
        <button
          onClick={handleResetConfig}
          disabled={saving}
          className={styles.resetButton}
        >
          Reset to Defaults
        </button>
      </div>
    </div>
  );
};

export default ConsensusTab;
