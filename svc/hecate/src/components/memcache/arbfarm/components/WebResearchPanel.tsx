import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService, WebSearchResult, WebResearchItem } from '../../../../common/services/arbfarm-service';
import CodeBlock, { formatJson } from '../../../common/CodeBlock';

type AnalysisFocus = 'strategy' | 'alpha' | 'risk' | 'token_analysis' | 'general';

const FOCUS_LABELS: Record<AnalysisFocus, string> = {
  strategy: 'Strategy',
  alpha: 'Alpha',
  risk: 'Risk',
  token_analysis: 'Token Analysis',
  general: 'General',
};

const FOCUS_COLORS: Record<AnalysisFocus, string> = {
  strategy: '#4CAF50',
  alpha: '#FF9800',
  risk: '#F44336',
  token_analysis: '#2196F3',
  general: '#9C27B0',
};

const WebResearchPanel: React.FC = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [urlInput, setUrlInput] = useState('');
  const [searchResults, setSearchResults] = useState<WebSearchResult[]>([]);
  const [savedResearch, setSavedResearch] = useState<WebResearchItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [analyzing, setAnalyzing] = useState(false);
  const [selectedFocus, setSelectedFocus] = useState<AnalysisFocus>('general');
  const [searchType, setSearchType] = useState<'search' | 'news'>('search');
  const [timeRange, setTimeRange] = useState<'day' | 'week' | 'month' | 'year' | ''>('');
  const [saveToEngrams, setSaveToEngrams] = useState(true);
  const [selectedResult, setSelectedResult] = useState<WebSearchResult | null>(null);
  const [fetchedContent, setFetchedContent] = useState<string | null>(null);
  const [analysisResult, setAnalysisResult] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchSavedResearch = useCallback(async () => {
    try {
      const res = await arbFarmService.getWebResearch(20);
      if (res.success && res.data) {
        const content = typeof res.data === 'string' ? JSON.parse(res.data) : res.data;
        if (content.research) {
          setSavedResearch(content.research);
        }
      }
    } catch (err) {
      console.error('Failed to fetch saved research:', err);
    }
  }, []);

  useEffect(() => {
    fetchSavedResearch();
  }, [fetchSavedResearch]);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchQuery.trim()) return;

    setLoading(true);
    setError(null);
    setSearchResults([]);

    try {
      const res = await arbFarmService.webSearch(searchQuery, {
        numResults: 5,
        searchType,
        ...(timeRange && { timeRange }),
      });

      if (res.success && res.data) {
        const content = typeof res.data === 'string' ? JSON.parse(res.data) : res.data;
        setSearchResults(content.results || []);
      } else {
        setError(res.error || 'Search failed');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed');
    } finally {
      setLoading(false);
    }
  };

  const handleFetchUrl = async (url: string) => {
    setAnalyzing(true);
    setError(null);
    setFetchedContent(null);
    setAnalysisResult(null);

    try {
      const fetchRes = await arbFarmService.webFetch(url, {
        extractMode: 'article',
        maxLength: 10000,
      });

      if (fetchRes.success && fetchRes.data) {
        const fetchContent = typeof fetchRes.data === 'string' ? JSON.parse(fetchRes.data) : fetchRes.data;
        setFetchedContent(fetchContent.content);

        const summarizeRes = await arbFarmService.webSummarize(
          fetchContent.content,
          url,
          {
            focus: selectedFocus,
            saveAsEngram: saveToEngrams,
          }
        );

        if (summarizeRes.success && summarizeRes.data) {
          const summaryContent = typeof summarizeRes.data === 'string' ? JSON.parse(summarizeRes.data) : summarizeRes.data;
          setAnalysisResult(summaryContent);
          if (saveToEngrams) {
            fetchSavedResearch();
          }
        } else {
          setError(summarizeRes.error || 'Analysis failed');
        }
      } else {
        setError(fetchRes.error || 'Failed to fetch URL');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Analysis failed');
    } finally {
      setAnalyzing(false);
    }
  };

  const handleDirectUrlAnalysis = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!urlInput.trim()) return;
    await handleFetchUrl(urlInput);
  };

  const formatTimestamp = (ts: string) => {
    const date = new Date(ts);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins} min ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  const truncateUrl = (url: string, maxLen: number = 50) => {
    if (url.length <= maxLen) return url;
    return url.substring(0, maxLen) + '...';
  };

  return (
    <div className={styles.webResearchPanel}>
      <div className={styles.sectionHeader}>
        <h3>Web Research</h3>
        <span className={styles.count}>{savedResearch.length} saved</span>
      </div>

      {/* Search Section */}
      <div className={styles.researchSection}>
        <h4>Web Search</h4>
        <form onSubmit={handleSearch} className={styles.searchForm}>
          <div className={styles.searchRow}>
            <input
              type="text"
              placeholder="Search for trading strategies, token info, market research..."
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              className={styles.searchInput}
            />
            <select
              value={searchType}
              onChange={e => setSearchType(e.target.value as 'search' | 'news')}
              className={styles.typeSelect}
            >
              <option value="search">Web</option>
              <option value="news">News</option>
            </select>
            <select
              value={timeRange}
              onChange={e => setTimeRange(e.target.value as any)}
              className={styles.typeSelect}
            >
              <option value="">Any time</option>
              <option value="day">Past day</option>
              <option value="week">Past week</option>
              <option value="month">Past month</option>
              <option value="year">Past year</option>
            </select>
            <button type="submit" className={styles.searchButton} disabled={loading}>
              {loading ? 'Searching...' : 'Search'}
            </button>
          </div>
        </form>

        {searchResults.length > 0 && (
          <div className={styles.searchResults}>
            {searchResults.map((result, idx) => (
              <div key={idx} className={styles.searchResultCard}>
                <div className={styles.resultTitle}>{result.title}</div>
                <div className={styles.resultUrl}>{truncateUrl(result.url)}</div>
                <div className={styles.resultSnippet}>{result.snippet}</div>
                <button
                  className={styles.analyzeButton}
                  onClick={() => {
                    setSelectedResult(result);
                    handleFetchUrl(result.url);
                  }}
                  disabled={analyzing}
                >
                  {analyzing && selectedResult?.url === result.url ? 'Analyzing...' : 'Analyze'}
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Direct URL Section */}
      <div className={styles.researchSection}>
        <h4>Analyze URL</h4>
        <form onSubmit={handleDirectUrlAnalysis} className={styles.searchForm}>
          <div className={styles.searchRow}>
            <input
              type="url"
              placeholder="https://example.com/article"
              value={urlInput}
              onChange={e => setUrlInput(e.target.value)}
              className={styles.searchInput}
            />
            <select
              value={selectedFocus}
              onChange={e => setSelectedFocus(e.target.value as AnalysisFocus)}
              className={styles.typeSelect}
            >
              <option value="general">General</option>
              <option value="strategy">Strategy</option>
              <option value="alpha">Alpha</option>
              <option value="risk">Risk</option>
              <option value="token_analysis">Token</option>
            </select>
            <label className={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={saveToEngrams}
                onChange={e => setSaveToEngrams(e.target.checked)}
              />
              Save
            </label>
            <button type="submit" className={styles.searchButton} disabled={analyzing}>
              {analyzing ? 'Analyzing...' : 'Analyze'}
            </button>
          </div>
        </form>
      </div>

      {/* Error Display */}
      {error && (
        <div className={styles.errorMessage}>
          {error}
        </div>
      )}

      {/* Analysis Result */}
      {analysisResult && (
        <div className={styles.researchSection}>
          <h4>Analysis Result</h4>
          <div className={styles.analysisResultCard}>
            <div className={styles.analysisHeader}>
              <span
                className={styles.focusBadge}
                style={{ backgroundColor: FOCUS_COLORS[analysisResult.analysis_focus as AnalysisFocus] || '#666' }}
              >
                {FOCUS_LABELS[analysisResult.analysis_focus as AnalysisFocus] || analysisResult.analysis_focus}
              </span>
              <span className={styles.confidence}>
                Confidence: {((analysisResult.confidence || 0) * 100).toFixed(0)}%
              </span>
              {analysisResult.engram_saved && (
                <span className={styles.savedBadge}>Saved</span>
              )}
            </div>

            <div className={styles.summarySection}>
              <h5>Summary</h5>
              <p>{analysisResult.summary}</p>
            </div>

            {analysisResult.key_insights && analysisResult.key_insights.length > 0 && (
              <div className={styles.insightsSection}>
                <h5>Key Insights</h5>
                <ul>
                  {analysisResult.key_insights.map((insight: string, idx: number) => (
                    <li key={idx}>{insight}</li>
                  ))}
                </ul>
              </div>
            )}

            {analysisResult.extracted_tokens && analysisResult.extracted_tokens.length > 0 && (
              <div className={styles.tokensSection}>
                <h5>Mentioned Tokens</h5>
                <div className={styles.tokensList}>
                  {analysisResult.extracted_tokens.map((token: string, idx: number) => (
                    <span key={idx} className={styles.tokenBadge}>${token}</span>
                  ))}
                </div>
              </div>
            )}

            {analysisResult.extracted_strategies && analysisResult.extracted_strategies.length > 0 && (
              <div className={styles.strategiesSection}>
                <h5>Extracted Strategies</h5>
                <CodeBlock code={formatJson(analysisResult.extracted_strategies)} language="json" />
              </div>
            )}
          </div>
        </div>
      )}

      {/* Saved Research */}
      <div className={styles.researchSection}>
        <h4>Saved Research</h4>
        {savedResearch.length === 0 ? (
          <div className={styles.emptyState}>
            No saved research yet. Analyze URLs to build your research library.
          </div>
        ) : (
          <div className={styles.savedResearchList}>
            {savedResearch.map((item, idx) => (
              <div key={idx} className={styles.savedResearchCard}>
                <div className={styles.savedHeader}>
                  <span
                    className={styles.focusBadge}
                    style={{ backgroundColor: FOCUS_COLORS[item.analysis_focus as AnalysisFocus] || '#666' }}
                  >
                    {FOCUS_LABELS[item.analysis_focus as AnalysisFocus] || item.analysis_focus}
                  </span>
                  <span className={styles.timestamp}>
                    {formatTimestamp(item.analyzed_at)}
                  </span>
                </div>
                <div className={styles.savedTitle}>
                  {item.title || truncateUrl(item.source_url, 60)}
                </div>
                <div className={styles.savedSummary}>
                  {item.summary.substring(0, 200)}...
                </div>
                <div className={styles.savedMeta}>
                  <span>{item.insights_count} insights</span>
                  <span>{item.strategies_count} strategies</span>
                  <span>Conf: {(item.confidence * 100).toFixed(0)}%</span>
                </div>
                {item.tokens && item.tokens.length > 0 && (
                  <div className={styles.savedTokens}>
                    {item.tokens.slice(0, 5).map((token, tIdx) => (
                      <span key={tIdx} className={styles.tokenBadge}>${token}</span>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default WebResearchPanel;
