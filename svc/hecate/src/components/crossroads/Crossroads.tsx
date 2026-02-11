import React, { useState, useEffect, useCallback } from 'react';
import styles from './crossroads.module.scss';
import CrossroadsLanding from './landing/CrossroadsLanding';
import MarketplaceBrowser from './marketplace/MarketplaceBrowser';
import PipBoyBar from './marketplace/PipBoyBar';
import NexusView from './nexus/NexusView';
import type { ServiceListing } from './types';
import type { MCPTool, DiscoveredAgent, DiscoveredProtocol, ToolCategory, CategorySummary } from '../../types/crossroads';
import {
  discoverAllMcpTools,
  discoverAgents,
  discoverProtocols,
  searchTools,
  filterToolsByCategory,
  getHotTools,
} from '../../common/services/crossroads-api';

type AnimationPhase = 'idle' | 'black' | 'stars' | 'background' | 'navbar' | 'complete';
type TabType = 'tools' | 'agents' | 'protocols';

interface CrossroadsProps {
  publicKey: string | null;
  onConnectWallet: (walletType?: 'phantom' | 'metamask') => void;
  showMarketplace?: boolean;
  resetToLanding?: boolean;
  animationPhase?: AnimationPhase;
}

type View = 'nexus' | 'landing' | 'marketplace' | 'service-detail' | 'my-services';

const Crossroads: React.FC<CrossroadsProps> = ({
  publicKey,
  onConnectWallet: _onConnectWallet,
  showMarketplace,
  resetToLanding,
  animationPhase: _animationPhase = 'complete',
}) => {
  const [currentView, setCurrentView] = useState<View>('nexus');
  const [selectedService, setSelectedService] = useState<ServiceListing | null>(null);
  const [showMarketplaceFadeIn, setShowMarketplaceFadeIn] = useState<boolean>(false);
  const previousView = React.useRef<View>('nexus');

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

  React.useEffect(() => {
    if (publicKey) {
      if (currentView === 'landing') setCurrentView('nexus');
    } else {
      setSelectedService(null);
    }
  }, [publicKey]);

  React.useEffect(() => {
    if (showMarketplace) {
      if (previousView.current === 'landing') {
        setShowMarketplaceFadeIn(true);
        setTimeout(() => setShowMarketplaceFadeIn(false), 1000);
      }
      setCurrentView('marketplace');
    }
  }, [showMarketplace]);

  React.useEffect(() => {
    previousView.current = currentView;
  }, [currentView]);

  React.useEffect(() => {
    if (resetToLanding) {
      setCurrentView('landing');
      setSelectedService(null);
    }
  }, [resetToLanding]);

  const handleBackToMarketplace = () => {
    setSelectedService(null);
    setCurrentView('marketplace');
  };

  const handleViewChange = (view: 'nexus' | 'marketplace') => {
    setCurrentView(view);
  };

  const pipboyView = currentView === 'marketplace' || currentView === 'nexus'
    ? currentView
    : currentView === 'service-detail' ? 'marketplace' : 'nexus';

  const renderView = () => {
    switch (currentView) {
      case 'landing':
        return <CrossroadsLanding />;

      case 'nexus':
        return (
          <NexusView
            toolCount={tools.length}
            agentCount={agents.length}
            protocolCount={protocols.length}
          />
        );

      case 'marketplace':
        return (
          <div className={showMarketplaceFadeIn ? styles.marketplaceFadeIn : ''}>
            <MarketplaceBrowser
              activeTab={activeTab}
              filteredTools={filteredTools}
              hotTools={hotTools}
              agents={agents}
              protocols={protocols}
              loading={loading}
              searchQuery={searchQuery}
              selectedCategory={selectedCategory}
            />
          </div>
        );

      case 'service-detail':
        return (
          <div>
            <button
              onClick={handleBackToMarketplace}
              style={{
                padding: '0.75rem 1.5rem',
                background: 'rgba(30, 41, 59, 0.8)',
                border: '1px solid rgba(100, 116, 139, 0.3)',
                color: '#e2e8f0',
                borderRadius: '0.5rem',
                cursor: 'pointer',
                marginBottom: '1.5rem',
              }}
            >
              Back to Marketplace
            </button>
            {selectedService && (
              <div
                style={{
                  background: 'rgba(30, 41, 59, 0.6)',
                  padding: '2rem',
                  borderRadius: '0.75rem',
                  border: '1px solid rgba(100, 116, 139, 0.3)',
                }}
              >
                <h2 style={{ fontSize: '2rem', marginBottom: '1rem', color: '#f1f5f9' }}>
                  {selectedService.title}
                </h2>
                <p style={{ color: '#94a3b8', marginBottom: '1.5rem' }}>
                  {selectedService.short_description}
                </p>
                <div
                  style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap', marginBottom: '2rem' }}
                >
                  {selectedService.capabilities.map((cap) => (
                    <span
                      key={cap}
                      style={{
                        padding: '0.5rem 1rem',
                        background: 'rgba(99, 102, 241, 0.1)',
                        border: '1px solid rgba(99, 102, 241, 0.3)',
                        borderRadius: '0.5rem',
                        color: '#6366f1',
                        fontSize: '0.875rem',
                      }}
                    >
                      {cap}
                    </span>
                  ))}
                </div>
                <p style={{ color: '#cbd5e1', marginBottom: '1rem' }}>
                  <strong>Version:</strong> {selectedService.version}
                </p>
                <p style={{ color: '#cbd5e1', marginBottom: '1rem' }}>
                  <strong>Endpoint:</strong> {selectedService.endpoint_url || 'N/A'}
                </p>
                <p style={{ color: '#cbd5e1', marginBottom: '1rem' }}>
                  <strong>Deployments:</strong> {selectedService.deployment_count}
                </p>
                <p style={{ color: '#cbd5e1', marginBottom: '2rem' }}>
                  <strong>Rating:</strong>{' '}
                  {selectedService.rating_average
                    ? `${selectedService.rating_average}/5`
                    : 'No ratings yet'}{' '}
                  ({selectedService.rating_count} reviews)
                </p>
                <button
                  style={{
                    padding: '1rem 2rem',
                    background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)',
                    color: 'white',
                    border: 'none',
                    borderRadius: '0.5rem',
                    fontSize: '1.125rem',
                    fontWeight: '600',
                    cursor: 'pointer',
                  }}
                  onClick={() => alert('Deploy functionality coming soon!')}
                >
                  Deploy Service
                </button>
              </div>
            )}
          </div>
        );

      case 'my-services':
        return (
          <div>
            <h2>My Services</h2>
            <p>Coming soon...</p>
          </div>
        );

      default:
        return <div>Unknown view</div>;
    }
  };

  const showPipBoy = currentView !== 'landing';

  return (
    <div className={styles.crossroadsContainer}>
      <div className={styles.crossroadsContent}>
        {renderView()}
      </div>
      {showPipBoy && (
        <PipBoyBar
          activeView={pipboyView}
          onViewChange={handleViewChange}
          activeTab={activeTab}
          onTabChange={setActiveTab}
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          categories={categories}
          selectedCategory={selectedCategory}
          onCategoryChange={setSelectedCategory}
          toolCount={tools.length}
          agentCount={agents.length}
          protocolCount={protocols.length}
          discoveryTime={discoveryTime}
          footerVisible={!publicKey}
        />
      )}
    </div>
  );
};

export default Crossroads;
