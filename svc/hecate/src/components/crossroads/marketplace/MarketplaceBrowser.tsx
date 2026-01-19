import React, { useState, useEffect, useCallback } from 'react';
import styles from '../crossroads.module.scss';
import type { MCPTool, DiscoveredAgent, DiscoveredProtocol, ToolCategory, CategorySummary } from '../../../types/crossroads';
import { TOOL_CATEGORY_LABELS } from '../../../types/crossroads';
import {
  discoverAllMcpTools,
  discoverAgents,
  discoverProtocols,
  searchTools,
  filterToolsByCategory,
  getHotTools,
} from '../../../common/services/crossroads-api';
import HotToolsSection from './HotToolsSection';
import ToolList from './ToolList';
import ToolDetailPanel from './ToolDetailPanel';
import AgentList from './AgentList';
import ProtocolList from './ProtocolList';

type TabType = 'tools' | 'agents' | 'protocols';

interface MarketplaceBrowserProps {
  onServiceClick?: (service: unknown) => void;
}

const MarketplaceBrowser: React.FC<MarketplaceBrowserProps> = () => {
  const [activeTab, setActiveTab] = useState<TabType>('tools');
  const [tools, setTools] = useState<MCPTool[]>([]);
  const [agents, setAgents] = useState<DiscoveredAgent[]>([]);
  const [protocols, setProtocols] = useState<DiscoveredProtocol[]>([]);
  const [categories, setCategories] = useState<CategorySummary[]>([]);
  const [filteredTools, setFilteredTools] = useState<MCPTool[]>([]);
  const [hotTools, setHotTools] = useState<MCPTool[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<ToolCategory | 'All'>('All');
  const [selectedTool, setSelectedTool] = useState<MCPTool | null>(null);
  const [discoveryTime, setDiscoveryTime] = useState<number>(0);

  const loadTools = useCallback(async () => {
    try {
      setLoading(true);
      const response = await discoverAllMcpTools();
      setTools(response.tools);
      setCategories(response.categories);
      setHotTools(getHotTools(response.tools));
      setDiscoveryTime(response.discoveryTimeMs);
    } catch (error) {
      console.error('Failed to load tools:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadAgents = useCallback(async () => {
    try {
      setLoading(true);
      const response = await discoverAgents();
      setAgents(response.agents);
    } catch (error) {
      console.error('Failed to load agents:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadProtocols = useCallback(async () => {
    try {
      setLoading(true);
      const response = await discoverProtocols();
      setProtocols(response.protocols);
    } catch (error) {
      console.error('Failed to load protocols:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (activeTab === 'tools') {
      loadTools();
    } else if (activeTab === 'agents') {
      loadAgents();
    } else if (activeTab === 'protocols') {
      loadProtocols();
    }
  }, [activeTab, loadTools, loadAgents, loadProtocols]);

  useEffect(() => {
    let filtered = tools;

    if (selectedCategory !== 'All') {
      filtered = filterToolsByCategory(filtered, selectedCategory);
    }

    if (searchQuery) {
      filtered = searchTools(filtered, searchQuery);
    }

    setFilteredTools(filtered);
  }, [tools, searchQuery, selectedCategory]);

  const handleToolClick = (tool: MCPTool) => {
    setSelectedTool(tool);
  };

  const handleCloseDetail = () => {
    setSelectedTool(null);
  };

  const handleCategoryClick = (category: ToolCategory | 'All') => {
    setSelectedCategory(category);
  };

  return (
    <div className={styles.marketplaceBrowser}>
      <aside className={styles.marketplaceSidebar}>
        <div className={styles.sidebarSection}>
          <h3 className={styles.sidebarTitle}>Search</h3>
          <div className={styles.searchBar}>
            <input
              type="text"
              placeholder={activeTab === 'tools' ? 'Search MCP tools...' : `Search ${activeTab}...`}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        <div className={styles.sidebarSection}>
          <h3 className={styles.sidebarTitle}>Asset Type</h3>
          <div className={styles.tabButtons}>
            <button
              className={`${styles.tabButton} ${activeTab === 'tools' ? styles.active : ''}`}
              onClick={() => setActiveTab('tools')}
            >
              🔧 Tools
              {tools.length > 0 && <span className={styles.tabCount}>{tools.length}</span>}
            </button>
            <button
              className={`${styles.tabButton} ${activeTab === 'agents' ? styles.active : ''}`}
              onClick={() => setActiveTab('agents')}
            >
              🤖 Agents
              {agents.length > 0 && <span className={styles.tabCount}>{agents.length}</span>}
            </button>
            <button
              className={`${styles.tabButton} ${activeTab === 'protocols' ? styles.active : ''}`}
              onClick={() => setActiveTab('protocols')}
            >
              📡 Protocols
              {protocols.length > 0 && <span className={styles.tabCount}>{protocols.length}</span>}
            </button>
          </div>
        </div>

        {activeTab === 'tools' && categories.length > 0 && (
          <div className={styles.sidebarSection}>
            <h3 className={styles.sidebarTitle}>Categories</h3>
            <div className={styles.categoryList}>
              <button
                className={`${styles.categoryButton} ${selectedCategory === 'All' ? styles.active : ''}`}
                onClick={() => handleCategoryClick('All')}
              >
                All Tools
                <span className={styles.categoryCount}>{tools.length}</span>
              </button>
              {categories.map((cat) => (
                <button
                  key={cat.category}
                  className={`${styles.categoryButton} ${selectedCategory === cat.category ? styles.active : ''}`}
                  onClick={() => handleCategoryClick(cat.category)}
                >
                  {cat.icon} {TOOL_CATEGORY_LABELS[cat.category]}
                  <span className={styles.categoryCount}>{cat.count}</span>
                </button>
              ))}
            </div>
          </div>
        )}

        {discoveryTime > 0 && (
          <div className={styles.sidebarFooter}>
            <span className={styles.discoveryTime}>
              Discovery: {discoveryTime}ms
            </span>
          </div>
        )}
      </aside>

      <main className={styles.marketplaceMain}>
        {activeTab === 'tools' && (
          <>
            {hotTools.length > 0 && selectedCategory === 'All' && !searchQuery && (
              <HotToolsSection tools={hotTools} onToolClick={handleToolClick} />
            )}
            <ToolList
              tools={filteredTools}
              onToolClick={handleToolClick}
              loading={loading}
            />
          </>
        )}

        {activeTab === 'agents' && (
          <AgentList agents={agents} loading={loading} />
        )}

        {activeTab === 'protocols' && (
          <ProtocolList protocols={protocols} loading={loading} />
        )}
      </main>

      {selectedTool && (
        <ToolDetailPanel tool={selectedTool} onClose={handleCloseDetail} />
      )}
    </div>
  );
};

export default MarketplaceBrowser;
