import React, { useState } from 'react';
import styles from '../crossroads.module.scss';
import type { MCPTool, DiscoveredAgent, DiscoveredProtocol, ToolCategory } from '../../../types/crossroads';
import HotToolsSection from './HotToolsSection';
import ToolList from './ToolList';
import ToolDetailPanel from './ToolDetailPanel';
import AgentList from './AgentList';
import ProtocolList from './ProtocolList';

type TabType = 'tools' | 'agents' | 'protocols';

interface MarketplaceBrowserProps {
  activeTab: TabType;
  filteredTools: MCPTool[];
  hotTools: MCPTool[];
  agents: DiscoveredAgent[];
  protocols: DiscoveredProtocol[];
  loading: boolean;
  searchQuery: string;
  selectedCategory: ToolCategory | 'All';
}

const MarketplaceBrowser: React.FC<MarketplaceBrowserProps> = ({
  activeTab,
  filteredTools,
  hotTools,
  agents,
  protocols,
  loading,
  searchQuery,
  selectedCategory,
}) => {
  const [selectedTool, setSelectedTool] = useState<MCPTool | null>(null);

  const handleToolClick = (tool: MCPTool) => {
    setSelectedTool(tool);
  };

  const handleCloseDetail = () => {
    setSelectedTool(null);
  };

  return (
    <div className={styles.marketplaceBrowser}>
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
