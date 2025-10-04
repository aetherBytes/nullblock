import React, { useState, useEffect, useCallback } from 'react';
import styles from '../crossroads.module.scss';
import ServiceGrid from './ServiceGrid';
import CommandBar from './CommandBar';
import type { ServiceListing, FilterState, ServiceCategory } from '../types';

interface MarketplaceBrowserProps {
  onServiceClick?: (service: ServiceListing) => void;
}

const categories: Array<{ value: ServiceCategory | 'All'; label: string }> = [
  { value: 'All', label: 'All Services' },
  { value: 'Agent', label: 'Agents' },
  { value: 'Workflow', label: 'Workflows' },
  { value: 'Tool', label: 'Tools' },
  { value: 'McpServer', label: 'MCP Servers' },
  { value: 'Dataset', label: 'Datasets' },
  { value: 'Model', label: 'Models' },
];

const mockServices: ServiceListing[] = [
  {
    id: '1',
    title: 'Hecate Orchestrator',
    short_description: 'Main conversational interface and orchestration engine for multi-agent coordination',
    listing_type: 'Agent',
    owner_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    endpoint_url: 'http://localhost:9003/hecate',
    version: '1.0.0',
    capabilities: ['chat', 'reasoning', 'model_switching', 'task_execution'],
    category_tags: ['orchestration', 'conversational', 'ai'],
    is_free: true,
    status: 'active',
    is_featured: true,
    health_status: 'healthy',
    rating_average: 4.8,
    rating_count: 234,
    deployment_count: 1200,
    view_count: 5600,
    favorite_count: 450,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '2',
    title: 'Siren Marketing Agent',
    short_description: 'Content generation and social media management for Web3 communities',
    listing_type: 'Agent',
    owner_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    endpoint_url: 'http://localhost:9003/siren',
    version: '1.0.0',
    capabilities: ['content_generation', 'social_media', 'marketing', 'community_engagement'],
    category_tags: ['marketing', 'social', 'content'],
    is_free: true,
    status: 'active',
    is_featured: false,
    health_status: 'healthy',
    rating_average: 4.6,
    rating_count: 89,
    deployment_count: 456,
    view_count: 2100,
    favorite_count: 120,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '3',
    title: 'DeFi Analytics Workflow',
    short_description: 'End-to-end workflow for DeFi protocol analysis and yield optimization',
    listing_type: 'Workflow',
    owner_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    version: '1.2.0',
    capabilities: ['analytics', 'defi', 'yield_optimization'],
    category_tags: ['defi', 'analytics', 'automation'],
    is_free: false,
    price_usd: 29.99,
    pricing_model: 'Subscription',
    status: 'active',
    is_featured: true,
    health_status: 'healthy',
    rating_average: 4.9,
    rating_count: 156,
    deployment_count: 789,
    view_count: 3400,
    favorite_count: 280,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '4',
    title: 'NullBlock MCP Server',
    short_description: 'Model Context Protocol server with agent interoperability',
    listing_type: 'McpServer',
    owner_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    endpoint_url: 'http://localhost:8001',
    version: '1.0.0',
    capabilities: ['resources', 'tools', 'prompts', 'a2a_protocol'],
    category_tags: ['mcp', 'protocol', 'integration'],
    is_free: true,
    status: 'active',
    is_featured: false,
    health_status: 'healthy',
    rating_average: 4.7,
    rating_count: 67,
    deployment_count: 234,
    view_count: 1200,
    favorite_count: 90,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '5',
    title: 'Price Oracle Tool',
    short_description: 'Real-time price feeds from multiple DEXs and CEXs',
    listing_type: 'Tool',
    owner_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    version: '2.1.0',
    capabilities: ['price_feeds', 'aggregation', 'websocket'],
    category_tags: ['trading', 'price', 'oracle'],
    is_free: false,
    price_usd: 9.99,
    pricing_model: 'Subscription',
    status: 'active',
    is_featured: false,
    health_status: 'healthy',
    rating_average: 4.5,
    rating_count: 312,
    deployment_count: 1567,
    view_count: 8900,
    favorite_count: 670,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '6',
    title: 'Historical DeFi Dataset',
    short_description: 'Comprehensive dataset of DeFi protocols, TVL, and transaction history',
    listing_type: 'Dataset',
    owner_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    version: '3.0.0',
    capabilities: ['historical_data', 'analytics', 'api_access'],
    category_tags: ['data', 'defi', 'analytics'],
    is_free: false,
    price_usd: 99.99,
    pricing_model: 'OneTime',
    status: 'active',
    is_featured: false,
    health_status: 'healthy',
    rating_average: 4.8,
    rating_count: 45,
    deployment_count: 123,
    view_count: 890,
    favorite_count: 56,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
];

const MarketplaceBrowser: React.FC<MarketplaceBrowserProps> = ({ onServiceClick }) => {
  const [services, setServices] = useState<ServiceListing[]>(mockServices);
  const [filteredServices, setFilteredServices] = useState<ServiceListing[]>(mockServices);
  const [loading, setLoading] = useState(false);
  const [filters, setFilters] = useState<FilterState>({});
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<ServiceCategory | 'All'>('All');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');

  const applyFilters = useCallback(() => {
    let filtered = [...services];

    if (selectedCategory !== 'All') {
      filtered = filtered.filter((s) => s.listing_type === selectedCategory);
    }

    if (filters.category && selectedCategory === 'All') {
      filtered = filtered.filter((s) => s.listing_type === filters.category);
    }

    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (s) =>
          s.title.toLowerCase().includes(query) ||
          s.short_description.toLowerCase().includes(query) ||
          s.category_tags.some((tag) => tag.toLowerCase().includes(query))
      );
    }

    if (filters.is_free !== undefined) {
      filtered = filtered.filter((s) => s.is_free === filters.is_free);
    }

    if (filters.min_rating) {
      filtered = filtered.filter((s) => (s.rating_average || 0) >= filters.min_rating!);
    }

    if (filters.sort_by) {
      switch (filters.sort_by) {
        case 'rating':
          filtered.sort((a, b) => (b.rating_average || 0) - (a.rating_average || 0));
          break;
        case 'trending':
          filtered.sort((a, b) => b.deployment_count - a.deployment_count);
          break;
        case 'recent':
          filtered.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
          break;
        case 'price':
          filtered.sort((a, b) => {
            const priceA = a.is_free ? 0 : a.price_usd || a.price_eth || 0;
            const priceB = b.is_free ? 0 : b.price_usd || b.price_eth || 0;
            return priceA - priceB;
          });
          break;
      }
    }

    setFilteredServices(filtered);
  }, [services, filters, searchQuery, selectedCategory]);

  useEffect(() => {
    applyFilters();
  }, [applyFilters]);

  const handleFilterChange = (newFilters: FilterState) => {
    setFilters(newFilters);
  };

  const handleCategoryTabClick = (category: ServiceCategory | 'All') => {
    setSelectedCategory(category);
  };

  return (
    <div className={styles.marketplaceBrowser}>
      <div className={styles.marketplaceHeader}>
        <h2>Marketplace</h2>
        <div className={styles.headerControls}>
          <div className={styles.searchBar}>
            <span className={styles.searchIcon}>🔍</span>
            <input
              type="text"
              placeholder="Search services..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
          <div className={styles.viewToggle}>
            <button
              className={viewMode === 'grid' ? styles.active : ''}
              onClick={() => setViewMode('grid')}
            >
              ⊞ Grid
            </button>
            <button
              className={viewMode === 'list' ? styles.active : ''}
              onClick={() => setViewMode('list')}
            >
              ☰ List
            </button>
          </div>
        </div>
      </div>

      <div className={styles.categoryTabs}>
        {categories.map((cat) => (
          <button
            key={cat.value}
            className={`${styles.categoryTab} ${selectedCategory === cat.value ? styles.active : ''}`}
            onClick={() => handleCategoryTabClick(cat.value)}
          >
            {cat.label}
          </button>
        ))}
      </div>

      <div className={styles.marketplaceContent}>
        <CommandBar filters={filters} onFilterChange={handleFilterChange} />
        <ServiceGrid
          services={filteredServices}
          loading={loading}
          onServiceClick={onServiceClick}
        />
      </div>

      {filteredServices.length > 0 && (
        <div className={styles.pagination}>
          <button disabled>← Previous</button>
          <button className={styles.active}>1</button>
          <button>2</button>
          <button>3</button>
          <button>Next →</button>
        </div>
      )}
    </div>
  );
};

export default MarketplaceBrowser;

