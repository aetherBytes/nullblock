import React, { useState, useEffect, useCallback } from 'react';
import styles from './crossroads.module.scss';
import MarketplaceBrowser from './marketplace/MarketplaceBrowser';
import HypeView from './nexus/HypeView';
import type { MCPTool, DiscoveredAgent, DiscoveredProtocol, CategorySummary } from '../../types/crossroads';
import {
  discoverAllMcpTools,
  discoverAgents,
  discoverProtocols,
  getHotTools,
} from '../../common/services/crossroads-api';
import type { CrossroadsSection } from '../hud/hud';

type AnimationPhase = 'idle' | 'black' | 'stars' | 'background' | 'navbar' | 'complete';

interface CrossroadsProps {
  publicKey: string | null;
  onConnectWallet: (walletType?: 'phantom' | 'metamask') => void;
  crossroadsSection?: CrossroadsSection;
  resetToLanding?: boolean;
  animationPhase?: AnimationPhase;
}

const Crossroads: React.FC<CrossroadsProps> = ({
  publicKey: _publicKey,
  onConnectWallet: _onConnectWallet,
  crossroadsSection = 'hype',
  resetToLanding: _resetToLanding,
  animationPhase: _animationPhase = 'complete',
}) => {
  const [tools, setTools] = useState<MCPTool[]>([]);
  const [agents, setAgents] = useState<DiscoveredAgent[]>([]);
  const [protocols, setProtocols] = useState<DiscoveredProtocol[]>([]);
  const [_categories, setCategories] = useState<CategorySummary[]>([]);
  const [filteredTools, setFilteredTools] = useState<MCPTool[]>([]);
  const [hotTools, setHotTools] = useState<MCPTool[]>([]);
  const [loading, setLoading] = useState(true);
  const [_discoveryTime, setDiscoveryTime] = useState<number>(0);

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
    if (crossroadsSection === 'tools') {
      loadTools();
    } else if (crossroadsSection === 'agents') {
      loadAgents();
    } else if (crossroadsSection === 'hype') {
      loadTools();
      loadAgents();
      loadProtocols();
    }
  }, [crossroadsSection, loadTools, loadAgents, loadProtocols]);

  useEffect(() => {
    setFilteredTools(tools);
  }, [tools]);

  const renderView = () => {
    switch (crossroadsSection) {
      case 'hype':
        return (
          <HypeView
            toolCount={tools.length}
            agentCount={agents.length}
            protocolCount={protocols.length}
          />
        );

      case 'tools':
        return (
          <MarketplaceBrowser
            activeTab="tools"
            filteredTools={filteredTools}
            hotTools={hotTools}
            agents={agents}
            protocols={protocols}
            loading={loading}
            searchQuery=""
            selectedCategory="All"
          />
        );

      case 'agents':
        return (
          <MarketplaceBrowser
            activeTab="agents"
            filteredTools={filteredTools}
            hotTools={hotTools}
            agents={agents}
            protocols={protocols}
            loading={loading}
            searchQuery=""
            selectedCategory="All"
          />
        );

      case 'cows':
        return (
          <HypeView
            toolCount={0}
            agentCount={0}
            protocolCount={0}
          />
        );

      default:
        return null;
    }
  };

  return (
    <div className={styles.crossroadsContainer}>
      {renderView()}
    </div>
  );
};

export default Crossroads;
