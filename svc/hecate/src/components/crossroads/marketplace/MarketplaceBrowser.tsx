import React, { useState, useEffect, useCallback } from 'react';
import styles from '../crossroads.module.scss';
import ServiceGrid from './ServiceGrid';
import ServiceList from './ServiceList';
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
    title: 'X Plugin',
    short_description: 'Social media integration for X (Twitter) via MCP protocol',
    listing_type: 'Tool',
    owner_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    version: '1.0.0',
    capabilities: ['social_media', 'mcp_tool', 'api_integration'],
    category_tags: ['mcp', 'social', 'integration'],
    is_free: false,
    price_usd: 14.99,
    pricing_model: 'Subscription',
    status: 'active',
    is_featured: false,
    health_status: 'healthy',
    rating_average: 4.6,
    rating_count: 42,
    deployment_count: 180,
    view_count: 1200,
    favorite_count: 95,
    is_coming_soon: true,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '4',
    title: 'Multi-Agent SDK',
    short_description: 'SDK tooling for tasking multi-agents via Hecate with MCP, A2A protocols',
    listing_type: 'Tool',
    owner_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    version: '0.9.0',
    capabilities: ['multi_agent', 'mcp', 'a2a_protocol', 'orchestration'],
    category_tags: ['sdk', 'agents', 'orchestration', 'mcp'],
    is_free: true,
    status: 'active',
    is_featured: false,
    health_status: 'healthy',
    rating_average: 4.8,
    rating_count: 34,
    deployment_count: 145,
    view_count: 890,
    favorite_count: 72,
    is_coming_soon: true,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '5',
    title: 'Context Nodes',
    short_description: 'Memory cards and context management for persistent agent knowledge',
    listing_type: 'Tool',
    owner_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    version: '0.5.0',
    capabilities: ['memory', 'context', 'persistence', 'knowledge_graph'],
    category_tags: ['memory', 'context', 'storage'],
    is_free: false,
    price_usd: 19.99,
    pricing_model: 'Subscription',
    status: 'active',
    is_featured: false,
    health_status: 'healthy',
    rating_average: 4.7,
    rating_count: 28,
    deployment_count: 98,
    view_count: 650,
    favorite_count: 45,
    is_coming_soon: true,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '6',
    title: 'Context Keeping Tools',
    short_description: 'Free engram-based context storage for agent memory, personas, strategies, and preferences via MCP',
    listing_type: 'Tool',
    owner_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    endpoint_url: 'http://localhost:8001/mcp',
    version: '1.0.0',
    capabilities: ['context_storage', 'persona_management', 'strategy_templates', 'knowledge_base', 'preference_tracking'],
    category_tags: ['context', 'memory', 'engrams', 'protocol', 'nullblock', 'mcp'],
    is_free: true,
    pricing_model: 'Free',
    status: 'active',
    is_featured: true,
    health_status: 'healthy',
    rating_average: 4.9,
    rating_count: 156,
    deployment_count: 890,
    view_count: 3400,
    favorite_count: 312,
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
            const priceA = a.is_free ? 0 : a.price_usd || a.price_mon || 0;
            const priceB = b.is_free ? 0 : b.price_usd || b.price_mon || 0;
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
      {/* Left Sidebar with all controls */}
      <aside className={styles.marketplaceSidebar}>
        <div className={styles.sidebarSection}>
          <h3 className={styles.sidebarTitle}>Search</h3>
          <div className={styles.searchBar}>
            <input
              type="text"
              placeholder="Search services..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        <div className={styles.sidebarSection}>
          <h3 className={styles.sidebarTitle}>View Mode</h3>
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

        <div className={styles.sidebarSection}>
          <h3 className={styles.sidebarTitle}>Categories</h3>
          <div className={styles.categoryList}>
            {categories.map((cat) => (
              <button
                key={cat.value}
                className={`${styles.categoryButton} ${selectedCategory === cat.value ? styles.active : ''}`}
                onClick={() => handleCategoryTabClick(cat.value)}
              >
                {cat.label}
              </button>
            ))}
          </div>
        </div>

        <div className={styles.sidebarSection}>
          <h3 className={styles.sidebarTitle}>Filters</h3>
          <CommandBar filters={filters} onFilterChange={handleFilterChange} />
        </div>
      </aside>

      {/* Main Content Area */}
      <main className={styles.marketplaceMain}>
        {viewMode === 'grid' ? (
          <ServiceGrid
            services={filteredServices}
            loading={loading}
            viewMode={viewMode}
            onServiceClick={onServiceClick}
          />
        ) : (
          <ServiceList
            services={filteredServices}
            loading={loading}
            onServiceClick={onServiceClick}
          />
        )}

        {filteredServices.length > 0 && (
          <div className={styles.pagination}>
            <button disabled>← Previous</button>
            <button className={styles.active}>1</button>
            <button>2</button>
            <button>3</button>
            <button>Next →</button>
          </div>
        )}
      </main>
    </div>
  );
};

export default MarketplaceBrowser;

