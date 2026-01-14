import React from 'react';
import styles from './hud.module.scss';

interface ModelInfoProps {
  isLoadingModelInfo: boolean;
  modelInfo: any;
  currentSelectedModel: string | null;
  availableModels: any[];
  defaultModelLoaded: boolean;
  showModelSelection: boolean;
  setShowModelSelection: (show: boolean) => void;
  setActiveQuickAction: (action: string | null) => void;
  setModelsCached: (cached: boolean) => void;
  loadAvailableModels: () => Promise<void>;
  showFullDescription: boolean;
  setShowFullDescription: (show: boolean) => void;
  modelSearchQuery: string;
  setModelSearchQuery: (query: string) => void;
  isSearchingModels: boolean;
  searchResults: any[];
  searchSubmitted: boolean;
  setSearchSubmitted: (submitted: boolean) => void;
  showSearchDropdown: boolean;
  setShowSearchDropdown: (show: boolean) => void;
  searchDropdownRef: React.RefObject<HTMLDivElement>;
  activeQuickAction: string | null;
  categoryModels: any[];
  isLoadingCategory: boolean;
  setCategoryModels: (models: any[]) => void;
  loadCategoryModels: (category: string) => void;
  handleModelSelection: (modelName: string) => void;
  getFreeModels: (models: any[], limit?: number) => any[];
  getFastModels: (models: any[], limit?: number) => any[];
  getThinkerModels: (models: any[], limit?: number) => any[];
  getInstructModels: (models: any[], limit?: number) => any[];
  getImageModels: (models: any[], limit?: number) => any[];
  getLatestModels: (models: any[], limit?: number) => any[];
}

const ModelInfo: React.FC<ModelInfoProps> = ({
  isLoadingModelInfo,
  modelInfo,
  currentSelectedModel,
  availableModels,
  defaultModelLoaded,
  showModelSelection,
  setShowModelSelection,
  setActiveQuickAction,
  setModelsCached,
  loadAvailableModels,
  showFullDescription,
  setShowFullDescription,
  modelSearchQuery,
  setModelSearchQuery,
  isSearchingModels,
  searchResults,
  searchSubmitted,
  setSearchSubmitted,
  showSearchDropdown,
  setShowSearchDropdown,
  searchDropdownRef,
  activeQuickAction,
  categoryModels,
  isLoadingCategory,
  setCategoryModels,
  loadCategoryModels,
  handleModelSelection,
  getFreeModels,
  getFastModels,
  getThinkerModels,
  getInstructModels,
  getImageModels,
  getLatestModels,
}) => {
  const renderModelSelectButton = (model: any, keyPrefix: string, index: number) => (
    <button
      key={`${keyPrefix}-${model.name}-${index}`}
      onClick={() => {
        handleModelSelection(model.name);
        setShowModelSelection(false);
        setSearchSubmitted(false);
        setModelSearchQuery('');
        setShowSearchDropdown(false);
      }}
      className={`${styles.modelSelectButton} ${model.name === currentSelectedModel ? styles.currentModel : ''}`}
    >
      <div className={styles.modelSelectInfo}>
        <span className={styles.modelSelectIcon}>{model.icon || '🤖'}</span>
        <div>
          <div className={styles.modelSelectName}>{model.display_name}</div>
          <div className={styles.modelSelectProvider}>{model.provider}</div>
        </div>
      </div>
      <div className={styles.modelSelectMeta}>
        <span className={styles.modelSelectTier}>
          {model.tier === 'economical'
            ? '🆓'
            : model.tier === 'fast'
              ? '⚡'
              : model.tier === 'standard'
                ? '⭐'
                : model.tier === 'premium'
                  ? '💎'
                  : '🤖'}
        </span>
        {model.name === currentSelectedModel && <span className={styles.currentBadge}>✓</span>}
      </div>
    </button>
  );

  const renderQuickAction = (
    action: string,
    icon: string,
    label: string,
    getModels: (models: any[]) => any[],
  ) => (
    <button
      onClick={() => {
        const newAction = activeQuickAction === action ? null : action;

        setActiveQuickAction(newAction);

        if (newAction === action) {
          setModelSearchQuery('');
          setCategoryModels([]);
          loadCategoryModels(action);
        }
      }}
      className={`${styles.quickActionTab} ${activeQuickAction === action ? styles.active : ''}`}
    >
      {label}
    </button>
  );

  const renderCategoryButton = (
    action: string,
    icon: string,
    label: string,
    getModels: (models: any[]) => any[],
  ) => (
    <button className={styles.categoryButton} onClick={() => setActiveQuickAction(action)}>
      <span className={styles.categoryIcon}>{icon}</span>
      <div className={styles.categoryInfo}>
        <div className={styles.categoryName}>{label}</div>
        <div className={styles.categoryCount}>{getModels(availableModels).length} models</div>
      </div>
    </button>
  );

  if (isLoadingModelInfo) {
    return (
      <div className={styles.modelInfoScope}>
        <div className={styles.modelInfoLoading}>
          <p>🔄 Loading model information...</p>
        </div>
      </div>
    );
  }

  if (modelInfo?.error) {
    return (
      <div className={styles.modelInfoScope}>
        <div className={styles.modelInfoError}>
          <h6>❌ Error Loading Model Info</h6>
          <p>{modelInfo.error}</p>
          <div style={{ marginTop: '10px', fontSize: '12px', color: '#666' }}>
            <p>Debug info:</p>
            <p>• Current selected model: {currentSelectedModel || 'None'}</p>
            <p>• Available models: {availableModels.length}</p>
            <p>• Default loaded: {defaultModelLoaded ? 'Yes' : 'No'}</p>
          </div>
          <button
            onClick={() => {
              console.log('Manual reload triggered from error - clearing cache');
              setModelsCached(false);
              loadAvailableModels();
            }}
            style={{
              marginTop: '10px',
              padding: '5px 10px',
              border: '1px solid #ccc',
              borderRadius: '4px',
            }}
          >
            🔄 Reload Models
          </button>
        </div>
      </div>
    );
  }

  if (showModelSelection) {
    return (
      <div className={styles.modelInfoScope}>
        <div className={styles.modelSelectionContent}>
          <div className={styles.modelSelectionHeader}>
            <div style={{ position: 'relative', flex: 1 }} ref={searchDropdownRef}>
              <input
                type="text"
                placeholder="Search LLM database..."
                value={modelSearchQuery}
                onChange={(e) => setModelSearchQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && modelSearchQuery.trim()) {
                    setSearchSubmitted(true);
                    setShowSearchDropdown(false);
                    setActiveQuickAction(null);
                  } else if (e.key === 'Escape') {
                    setShowSearchDropdown(false);
                  }
                }}
                className={styles.modelSearchInput}
              />
              {isSearchingModels && <div className={styles.searchingIndicator}>⟳ Searching...</div>}
              {showSearchDropdown && !searchSubmitted && searchResults.length > 0 && (
                <div className={styles.searchDropdown}>
                  {searchResults.slice(0, 8).map((model, index) => (
                    <button
                      key={`dropdown-${model.name}-${index}`}
                      onClick={() => {
                        handleModelSelection(model.name);
                        setShowModelSelection(false);
                        setShowSearchDropdown(false);
                        setSearchSubmitted(false);
                        setModelSearchQuery('');
                      }}
                      className={styles.searchDropdownItem}
                    >
                      <span className={styles.modelIcon}>{model.icon || '🤖'}</span>
                      <div className={styles.searchDropdownItemInfo}>
                        <div className={styles.searchDropdownItemName}>
                          {model.display_name || model.name}
                        </div>
                        <div className={styles.searchDropdownItemProvider}>{model.provider}</div>
                      </div>
                      <span className={styles.modelTierIcon}>
                        {model.tier === 'economical'
                          ? '🆓'
                          : model.tier === 'fast'
                            ? '⚡'
                            : model.tier === 'premium'
                              ? '💎'
                              : '⭐'}
                      </span>
                    </button>
                  ))}
                </div>
              )}
            </div>
            <button
              className={styles.backButton}
              onClick={() => {
                setShowModelSelection(false);
                setActiveQuickAction(null);
                setSearchSubmitted(false);
                setModelSearchQuery('');
              }}
              title="Back to model info"
            >
              ← Back
            </button>
          </div>

          <div style={{ overflowX: 'auto', overflowY: 'hidden', paddingBottom: '8px' }}>
            <div
              className={styles.quickActionsMenu}
              style={{ display: 'flex', flexWrap: 'nowrap', gap: '8px', minWidth: 'fit-content' }}
            >
              <button
                onClick={() => {
                  setModelSearchQuery('');
                  setSearchSubmitted(false);
                  setShowSearchDropdown(false);
                  setActiveQuickAction('clear');
                  setTimeout(() => setActiveQuickAction(null), 500);
                }}
                className={`${styles.quickActionTab} ${activeQuickAction === 'clear' ? styles.active : ''}`}
              >
                Clear Search
              </button>
              {renderQuickAction('latest', '🆕', 'Latest', getLatestModels)}
              {renderQuickAction('free', '🆓', 'Free', getFreeModels)}
              {renderQuickAction('fast', '⚡', 'Fast', getFastModels)}
              {renderQuickAction('thinkers', '🧠', 'Thinkers', getThinkerModels)}
              {renderQuickAction('instruct', '💬', 'Instruct', getInstructModels)}
              {renderQuickAction('image', '🎨', 'Image', getImageModels)}
            </div>
          </div>

          {searchSubmitted && modelSearchQuery.trim() ? (
            <div className={styles.modelSection}>
              <h6>Search Results ({searchResults.length})</h6>
              {isSearchingModels ? (
                <div style={{ textAlign: 'center', padding: '20px', opacity: 0.7 }}>
                  🔄 Searching models...
                </div>
              ) : searchResults.length === 0 ? (
                <div style={{ textAlign: 'center', padding: '20px', opacity: 0.7 }}>
                  No models found matching "{modelSearchQuery}"
                </div>
              ) : (
                <div className={styles.modelsList}>
                  {searchResults.map((model, index) =>
                    renderModelSelectButton(model, 'search', index),
                  )}
                </div>
              )}
            </div>
          ) : activeQuickAction === 'latest' ? (
            <div className={styles.modelSection}>
              <h6>Latest Models ({isLoadingCategory ? '...' : categoryModels.length})</h6>
              {isLoadingCategory ? (
                <div style={{ textAlign: 'center', padding: '20px', opacity: 0.7 }}>
                  🔄 Loading latest models from OpenRouter...
                </div>
              ) : (
                <div className={styles.modelsList}>
                  {categoryModels.map((model, index) =>
                    renderModelSelectButton(model, 'category', index),
                  )}
                </div>
              )}
            </div>
          ) : activeQuickAction &&
            ['free', 'fast', 'thinkers', 'instruct', 'image'].includes(activeQuickAction) ? (
            (() => {
              const getModelsFn =
                activeQuickAction === 'free'
                  ? getFreeModels
                  : activeQuickAction === 'fast'
                    ? getFastModels
                    : activeQuickAction === 'thinkers'
                      ? getThinkerModels
                      : activeQuickAction === 'image'
                        ? getImageModels
                        : getInstructModels;
              const models = getModelsFn(availableModels, 999);

              return (
                <div className={styles.modelSection}>
                  <h6>
                    {activeQuickAction.charAt(0).toUpperCase() + activeQuickAction.slice(1)} Models
                    ({models.length})
                  </h6>
                  <div className={styles.modelsList}>
                    {models.map((model, index) =>
                      renderModelSelectButton(model, activeQuickAction, index),
                    )}
                  </div>
                </div>
              );
            })()
          ) : (
            <>
              <div className={styles.modelSection}>
                <h6>Model Categories</h6>
                <div className={styles.categoryOverview}>
                  {renderCategoryButton('latest', '🆕', 'Latest', getLatestModels)}
                  {renderCategoryButton('free', '🆓', 'Free', getFreeModels)}
                  {renderCategoryButton('fast', '⚡', 'Fast', getFastModels)}
                  {renderCategoryButton('thinkers', '🧠', 'Thinkers', getThinkerModels)}
                  {renderCategoryButton('instruct', '💬', 'Instruct', getInstructModels)}
                  {renderCategoryButton('image', '🎨', 'Image', getImageModels)}
                </div>
              </div>

              <div className={styles.modelSection}>
                <h6>Database Statistics</h6>
                <div className={styles.modelCounts}>
                  <p>📊 Total Available: {availableModels.filter((m) => m.available).length}</p>
                  <p>🆓 Free Models: {getFreeModels(availableModels, 999).length}</p>
                  <p>⚡ Fast Models: {getFastModels(availableModels, 999).length}</p>
                  <p>🧠 Thinking Models: {getThinkerModels(availableModels, 999).length}</p>
                  <p>💬 Instruct Models: {getInstructModels(availableModels, 999).length}</p>
                  <p>🎨 Image Gen: {getImageModels(availableModels, 999).length}</p>
                  <p>🆕 Latest Added: {getLatestModels(availableModels, 999).length}</p>
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    );
  }

  if (!modelInfo) {
    return (
      <div className={styles.modelInfoScope}>
        <div className={styles.modelInfoEmpty}>
          <p>No model information available</p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.modelInfoScope}>
      <div className={styles.modelInfoContent}>
        <div className={styles.modelInfoHeader}>
          <div className={styles.modelInfoTitle}>
            <span className={styles.modelIcon}>{modelInfo.icon || '🤖'}</span>
            <div>
              <h6>{modelInfo.display_name || modelInfo.name}</h6>
              <span className={styles.modelProvider}>{modelInfo.provider}</span>
            </div>
          </div>
          <div className={styles.modelStatus}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <button
                onClick={() => {
                  console.log('Force reloading models - clearing cache');
                  setModelsCached(false);
                  loadAvailableModels();
                }}
                title="Reload models (forces fresh API call)"
                style={{
                  background: 'none',
                  border: 'none',
                  padding: '4px',
                  cursor: 'pointer',
                  fontSize: '16px',
                  lineHeight: '1',
                  opacity: 0.7,
                  transition: 'opacity 0.2s',
                }}
                onMouseEnter={(e) => ((e.target as HTMLElement).style.opacity = '1')}
                onMouseLeave={(e) => ((e.target as HTMLElement).style.opacity = '0.7')}
              >
                🔄
              </button>
              <button
                className={styles.switchModelButton}
                onClick={() => {
                  setShowModelSelection(true);
                  setActiveQuickAction('latest');
                  setCategoryModels([]);
                  loadCategoryModels('latest');
                }}
                title="Switch to a different model"
              >
                Switch Model
              </button>
            </div>
          </div>
        </div>

        {modelInfo.description && (
          <div className={styles.modelInfoSection}>
            <h6>📝 Description</h6>
            <p>
              {modelInfo.description.length > 300 && !showFullDescription
                ? `${modelInfo.description.substring(0, 300)}...`
                : modelInfo.description}
            </p>
            {modelInfo.description.length > 300 && (
              <button
                onClick={() => setShowFullDescription(!showFullDescription)}
                className={styles.showMoreButton}
              >
                {showFullDescription ? 'Show Less' : 'Show More'}
              </button>
            )}
            {(modelInfo.supports_reasoning ||
              (modelInfo.capabilities && modelInfo.capabilities.includes('reasoning'))) &&
              !(modelInfo.capabilities && modelInfo.capabilities.includes('reasoning_tokens')) && (
                <div className={styles.reasoningNote}>
                  <p>
                    <strong>💡 Note:</strong> This model supports general reasoning but not
                    step-by-step reasoning tokens. For complex reasoning tasks, consider using a
                    model with reasoning tokens like DeepSeek-R1.
                  </p>
                </div>
              )}
          </div>
        )}

        <div className={styles.modelInfoSection}>
          <h6>⚙️ Technical Specifications</h6>
          <div className={styles.modelSpecs}>
            <div className={styles.specItem}>
              <span className={styles.specLabel}>Context Length:</span>
              <span className={styles.specValue}>
                {modelInfo.context_length?.toLocaleString() || 'N/A'} tokens
              </span>
            </div>
            {(modelInfo.max_completion_tokens || modelInfo.top_provider?.max_completion_tokens) && (
              <div className={styles.specItem}>
                <span className={styles.specLabel}>Max Output:</span>
                <span className={styles.specValue}>
                  {(
                    modelInfo.max_completion_tokens || modelInfo.top_provider.max_completion_tokens
                  ).toLocaleString()}{' '}
                  tokens
                </span>
              </div>
            )}
            <div className={styles.specItem}>
              <span className={styles.specLabel}>Reasoning:</span>
              <span className={styles.specValue}>
                {modelInfo.supports_reasoning ||
                (modelInfo.capabilities && modelInfo.capabilities.includes('reasoning'))
                  ? '✅ Yes (General reasoning)'
                  : '❌ No'}
              </span>
            </div>
            <div className={styles.specItem}>
              <span className={styles.specLabel}>Reasoning Tokens:</span>
              <span className={styles.specValue}>
                {modelInfo.supports_reasoning ||
                (modelInfo.capabilities && modelInfo.capabilities.includes('reasoning_tokens'))
                  ? '✅ Yes (Step-by-step thinking)'
                  : '❌ No (General reasoning only)'}
              </span>
            </div>
            <div className={styles.specItem}>
              <span className={styles.specLabel}>Vision:</span>
              <span className={styles.specValue}>
                {modelInfo.architecture?.input_modalities?.includes('image') ? '✅ Yes' : '❌ No'}
              </span>
            </div>
            <div className={styles.specItem}>
              <span className={styles.specLabel}>Image Generation:</span>
              <span className={styles.specValue}>
                {modelInfo.architecture?.output_modalities?.includes('image') ? '✅ Yes' : '❌ No'}
              </span>
            </div>
            <div className={styles.specItem}>
              <span className={styles.specLabel}>Audio:</span>
              <span className={styles.specValue}>
                {modelInfo.supports_audio ? '✅ Yes' : '❌ No'}
              </span>
            </div>
            <div className={styles.specItem}>
              <span className={styles.specLabel}>Function Calls:</span>
              <span className={styles.specValue}>
                {modelInfo.supports_function_calling ? '✅ Yes' : '❌ No'}
              </span>
            </div>
            <div className={styles.specItem}>
              <span className={styles.specLabel}>Moderated:</span>
              <span className={styles.specValue}>
                {modelInfo.is_moderated ? '✅ Yes' : '❌ No'}
              </span>
            </div>
          </div>
        </div>

        <div className={styles.modelInfoSection}>
          <h6>💰 Cost Information</h6>
          <div className={styles.costInfo}>
            <div className={styles.specItem}>
              <span className={styles.specLabel}>Tier:</span>
              <span className={styles.specValue}>
                {modelInfo.tier === 'economical'
                  ? '🆓 Free'
                  : modelInfo.tier === 'fast'
                    ? '⚡ Fast'
                    : modelInfo.tier === 'standard'
                      ? '⭐ Standard'
                      : modelInfo.tier === 'premium'
                        ? '💎 Premium'
                        : modelInfo.tier || 'Unknown'}
              </span>
            </div>
            {modelInfo.pricing ? (
              <>
                <div className={styles.specItem}>
                  <span className={styles.specLabel}>Input tokens (1M):</span>
                  <span className={styles.specValue}>
                    {!modelInfo.pricing.prompt ||
                    modelInfo.pricing.prompt === '0' ||
                    modelInfo.pricing.prompt === 0
                      ? '🆓 Free'
                      : `$${(Number.parseFloat(modelInfo.pricing.prompt) * 1000000).toFixed(3)}`}
                  </span>
                </div>
                <div className={styles.specItem}>
                  <span className={styles.specLabel}>Output tokens (1M):</span>
                  <span className={styles.specValue}>
                    {!modelInfo.pricing.completion ||
                    modelInfo.pricing.completion === '0' ||
                    modelInfo.pricing.completion === 0
                      ? '🆓 Free'
                      : `$${(Number.parseFloat(modelInfo.pricing.completion) * 1000000).toFixed(3)}`}
                  </span>
                </div>
                {modelInfo.pricing.image && Number.parseFloat(modelInfo.pricing.image) > 0 && (
                  <div className={styles.specItem}>
                    <span className={styles.specLabel}>Image generation:</span>
                    <span className={styles.specValue}>
                      ${Number.parseFloat(modelInfo.pricing.image).toFixed(4)} per image
                    </span>
                  </div>
                )}
                {modelInfo.pricing.request && Number.parseFloat(modelInfo.pricing.request) > 0 && (
                  <div className={styles.specItem}>
                    <span className={styles.specLabel}>Per request:</span>
                    <span className={styles.specValue}>
                      ${Number.parseFloat(modelInfo.pricing.request).toFixed(4)}
                    </span>
                  </div>
                )}
              </>
            ) : modelInfo.cost_per_1k_tokens !== undefined ? (
              <>
                <div className={styles.specItem}>
                  <span className={styles.specLabel}>Combined cost (1K tokens):</span>
                  <span className={styles.specValue}>
                    {modelInfo.cost_per_1k_tokens === 0 ||
                    modelInfo.cost_per_1k_tokens === null ||
                    modelInfo.cost_per_1k_tokens === undefined
                      ? modelInfo.tier === 'economical'
                        ? '🆓 Free'
                        : '❓ Variable pricing'
                      : `$${Number(modelInfo.cost_per_1k_tokens).toFixed(8)}`}
                  </span>
                </div>
                <div className={styles.specItem}>
                  <span className={styles.specLabel}>Combined cost (1M tokens):</span>
                  <span className={styles.specValue}>
                    {modelInfo.cost_per_1k_tokens === 0 ||
                    modelInfo.cost_per_1k_tokens === null ||
                    modelInfo.cost_per_1k_tokens === undefined
                      ? modelInfo.tier === 'economical'
                        ? '🆓 Free'
                        : '❓ Variable pricing'
                      : `$${(Number(modelInfo.cost_per_1k_tokens) * 1000).toFixed(5)}`}
                  </span>
                </div>
              </>
            ) : (
              <div className={styles.specItem}>
                <span className={styles.specLabel}>⚠️ Pricing data:</span>
                <span className={styles.specValue}>
                  <span style={{ color: 'red' }}>No pricing information available</span>
                </span>
              </div>
            )}
          </div>
        </div>

        {modelInfo.capabilities && modelInfo.capabilities.length > 0 && (
          <div className={styles.modelInfoSection}>
            <h6>🎯 Capabilities</h6>
            <div className={styles.capabilitiesList}>
              {modelInfo.capabilities.map((capability: string) => (
                <span
                  key={capability}
                  className={styles.capabilityTag}
                  title={capability.replace('_', ' ')}
                >
                  {capability.replace('_', ' ')}
                </span>
              ))}
            </div>
          </div>
        )}

        {modelInfo.supported_parameters && modelInfo.supported_parameters.length > 0 && (
          <div className={styles.modelInfoSection}>
            <h6>🔧 Supported Parameters</h6>
            <div className={styles.capabilitiesList}>
              {modelInfo.supported_parameters.map((param: string) => (
                <span key={param} className={styles.capabilityTag} title={param}>
                  {param}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default ModelInfo;
