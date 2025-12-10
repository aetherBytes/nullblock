# Crossroads UI Design & Architecture

## 🎯 Vision

Transform Crossroads into a modern, Web3-native marketplace for AI agents, workflows, tools, and MCP servers. Think "App Store meets DeFi" with seamless wallet integration, on-chain identity, and real-time service discovery.

---

## 🏗️ Component Architecture

### 1. **Landing View** (Unauthenticated)
**Purpose**: First impression, education, CTA for wallet connection

**Components**:
- `<CrossroadsHero />` - Animated hero section with value proposition
- `<FeatureShowcase />` - Interactive cards showing key features
- `<TrendingServices />` - Preview of popular services (public data)
- `<StatsOverview />` - Live marketplace statistics
- `<ConnectPrompt />` - OnchainKit ConnectWallet integration

**Design Elements**:
```tsx
<CrossroadsHero>
  - Gradient background with animated particles
  - "Discover, Deploy, Monetize AI Services"
  - "1,234 Services • 5,678 Active Users • $X in Volume"
  - Large ConnectWallet button with OnchainKit
</CrossroadsHero>

<FeatureShowcase>
  - 🤖 Agent Marketplace - Deploy autonomous agents instantly
  - 🔗 MCP Integration - Connect to any MCP server
  - ⚡ One-Click Deploy - From discovery to deployment in seconds
  - 💰 Monetize Services - Earn from your creations
</FeatureShowcase>

<TrendingServices>
  - Top 6 services in grid
  - Each card shows: icon, name, category, rating, usage count
  - "Connect to see more" overlay on hover
</TrendingServices>
```

---

### 2. **Marketplace Browser** (Main View)
**Purpose**: Browse, search, filter all available services

**Components**:
- `<MarketplaceHeader />` - Search, filters, view toggle
- `<CategoryTabs />` - Quick filter by type (All, Agents, Workflows, Tools, MCP)
- `<ServiceGrid />` or `<ServiceList />` - Dynamic service display
- `<FilterSidebar />` - Advanced filters
- `<Pagination />` - Results pagination

**Layout**:
```
┌─────────────────────────────────────────────────────────┐
│ [Search Bar]  [Filters ▼] [Price ▼]  [Grid/List Toggle] │
├─────────────────────────────────────────────────────────┤
│ [All] [Agents] [Workflows] [Tools] [MCP] [Datasets]     │
├──────────────┬──────────────────────────────────────────┤
│ FILTERS      │  SERVICE GRID                             │
│              │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐     │
│ Category     │  │      │ │      │ │      │ │      │     │
│ ☑ Agents     │  │ Card │ │ Card │ │ Card │ │ Card │     │
│ ☐ Workflows  │  │      │ │      │ │      │ │      │     │
│              │  └──────┘ └──────┘ └──────┘ └──────┘     │
│ Price        │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐     │
│ ○ Free       │  │      │ │      │ │      │ │      │     │
│ ○ Paid       │  │ Card │ │ Card │ │ Card │ │ Card │     │
│ ○ All        │  │      │ │      │ │      │ │      │     │
│              │  └──────┘ └──────┘ └──────┘ └──────┘     │
│ Rating       │                                            │
│ ⭐⭐⭐⭐⭐        │  [1] [2] [3] ... [10]                     │
│              │                                            │
└──────────────┴──────────────────────────────────────────┘
```

**ServiceCard Design**:
```tsx
<ServiceCard>
  <CardHeader>
    <ServiceIcon /> {/* Agent/Workflow/Tool icon */}
    <CategoryBadge /> {/* Color-coded category */}
    <FeaturedStar /> {/* If featured */}
  </CardHeader>
  
  <CardBody>
    <Title>Service Name</Title>
    <Description>Short description (2 lines max)</Description>
    
    <OwnerInfo>
      <Avatar chain="base" address={owner} /> {/* OnchainKit */}
      <Name address={owner} /> {/* OnchainKit */}
      <Badge /> {/* OnchainKit - ENS/Basename badge */}
    </OwnerInfo>
    
    <Metrics>
      ⭐ 4.8 (234 reviews) • 🚀 1.2k deployments • ⚡ Active
    </Metrics>
    
    <PriceTag>
      {isFree ? "Free" : "$X/month or 0.X ETH"}
    </PriceTag>
  </CardBody>
  
  <CardActions>
    <ViewDetailsButton />
    <QuickDeployButton /> {/* If already configured */}
  </CardActions>
</ServiceCard>
```

---

### 3. **Service Detail View**
**Purpose**: Full information about a service before deployment/purchase

**Components**:
- `<ServiceHero />` - Banner with key info
- `<ServiceTabs />` - Overview, Configuration, Reviews, Analytics
- `<DeploymentPanel />` - Right sidebar for actions
- `<RelatedServices />` - Recommendations

**Layout**:
```
┌────────────────────────────────────────────┬──────────────┐
│ [← Back to Marketplace]                    │              │
├────────────────────────────────────────────┤ DEPLOYMENT   │
│ 🤖 Service Name                            │ PANEL        │
│ by [Avatar] [Name] [Badge]                 │              │
│ ⭐⭐⭐⭐⭐ 4.8 (234) • Category: Agent       │ Price: Free  │
│                                            │              │
│ [Overview] [Config] [Reviews] [Analytics]  │ [Deploy Now] │
├────────────────────────────────────────────┤              │
│                                            │ [Try Demo]   │
│ OVERVIEW TAB:                              │              │
│ - Full description                         │ Requirements:│
│ - Key features list                        │ • Base chain │
│ - Supported models/protocols               │ • 2GB RAM    │
│ - Use cases                                │              │
│ - Screenshots/demos                        │ Tags:        │
│ - Version history                          │ #trading     │
│                                            │ #defi        │
│ CONFIGURATION TAB:                         │              │
│ - Required environment variables           │ Owner:       │
│ - API endpoints                            │ [Identity]   │
│ - Resource requirements                    │              │
│ - Integration examples                     │ Share:       │
│                                            │ [🔗][𝕏][📋]  │
│ REVIEWS TAB:                               │              │
│ - User reviews with ratings                │              │
│ - Filter by rating/date                    │              │
│ - Reply threads                            │              │
│                                            │              │
│ ANALYTICS TAB:                             │              │
│ - Usage trends                             │              │
│ - Performance metrics                      │              │
│ - Uptime history                           │              │
└────────────────────────────────────────────┴──────────────┘
```

**DeploymentPanel Features**:
```tsx
<DeploymentPanel>
  <PriceDisplay>
    {isFree ? "Free Forever" : (
      <PricingOptions>
        <Option>$10/month subscription</Option>
        <Option>0.01 ETH one-time</Option>
        <Option>Pay-per-use</Option>
      </PricingOptions>
    )}
  </PriceDisplay>
  
  <ActionButtons>
    <DeployButton>
      {isOwned ? "Launch Instance" : "Deploy Service"}
    </DeployButton>
    <TryDemoButton>Try in Sandbox</TryDemoButton>
  </ActionButtons>
  
  <RequirementsList>
    ✅ Wallet connected
    ✅ Base chain selected
    ⚠️ Requires 0.1 ETH for gas
    ❌ Service not configured
  </RequirementsList>
  
  <OwnerIdentity>
    <Identity address={owner}>
      <Avatar />
      <Name />
      <Badge />
    </Identity>
    <VerificationStatus>✓ Verified Publisher</VerificationStatus>
  </OwnerIdentity>
  
  <SocialShare>
    <ShareButton platform="twitter" />
    <ShareButton platform="farcaster" />
    <CopyLinkButton />
  </SocialShare>
</DeploymentPanel>
```

---

### 4. **Publish Service View**
**Purpose**: Allow users to list their own services

**Components**:
- `<PublishWizard />` - Multi-step form
- `<ServiceTypeSelector />` - Choose what to publish
- `<MetadataForm />` - Title, description, tags
- `<ConfigurationForm />` - Technical details
- `<PricingForm />` - Set pricing model
- `<PreviewPanel />` - Live preview of listing

**Wizard Steps**:
```
Step 1: Service Type
- Select: Agent | Workflow | Tool | MCP Server | Dataset | Model
- Each has custom icon and description

Step 2: Basic Information
- Title (required)
- Short description (280 chars)
- Long description (markdown supported)
- Category tags (autocomplete)
- Icon/logo upload

Step 3: Technical Configuration
For Agents:
- Supported models
- API endpoint URL
- Health check endpoint
- Capabilities list
- Example requests

For MCP Servers:
- Protocol version
- Capabilities (resources, tools, prompts)
- Connection string
- Authentication method

For Workflows:
- Workflow steps (visual editor)
- Input/output schemas
- Required services

Step 4: Pricing & Distribution
- Free or Paid
- Pricing model: Subscription | One-time | Pay-per-use | Token staking
- License type
- Terms of service

Step 5: Review & Publish
- Preview how it will appear
- Verification checklist
- Submit for approval (if required)
- Publish immediately (if auto-approved)
```

---

### 5. **My Services Dashboard**
**Purpose**: Manage your published and deployed services

**Components**:
- `<MyServicesHeader />` - Tabs for Published vs Deployed
- `<ServiceManagementCard />` - Each service with actions
- `<AnalyticsDashboard />` - Earnings, usage, ratings
- `<EditServiceModal />` - Update service details

**Layout**:
```
┌─────────────────────────────────────────────────────────┐
│ My Services                                             │
│ [Published] [Deployed] [Favorites]                      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│ YOUR PUBLISHED SERVICES (3)                             │
│ ┌────────────────────────────────────────────────────┐  │
│ │ 🤖 Hecate Trading Agent     [Edit] [Analytics] [•] │  │
│ │ Status: Active • 234 deployments • $456 earned     │  │
│ │ ⭐ 4.8 (89 reviews) • Updated 2 days ago           │  │
│ │ [View Listing] [Pause] [Update]                    │  │
│ └────────────────────────────────────────────────────┘  │
│                                                         │
│ DEPLOYED SERVICES (5)                                   │
│ ┌────────────────────────────────────────────────────┐  │
│ │ 🔧 Data Analytics Tool         [Launch] [•]        │  │
│ │ Status: Running • Uptime: 99.8% • Cost: $5/mo     │  │
│ │ Last used: 2 hours ago                            │  │
│ │ [Open] [Stop] [Configure]                         │  │
│ └────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

### 6. **Discovery & Health Dashboard**
**Purpose**: Real-time view of ecosystem services

**Components**:
- `<ServiceHealthGrid />` - Live status of all services
- `<NetworkTopology />` - Visual graph of service connections
- `<RecentActivity />` - Feed of deployments, updates
- `<SystemMetrics />` - Ecosystem-wide stats

**Design**:
```
┌─────────────────────────────────────────────────────────┐
│ Discovery Dashboard              [Scan Now] [Auto: ON]  │
├─────────────────────────────────────────────────────────┤
│ SYSTEM HEALTH                                           │
│ ●●●●● Agents (5)    ●●●● Workflows (4)    ●● MCP (2)   │
│                                                         │
│ ACTIVE SERVICES                                         │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│ │🤖 Hecate │ │🎭 Siren  │ │🌐 MCP    │ │🔄 Workflow│  │
│ │ ● Healthy│ │ ● Healthy│ │ ● Healthy│ │ ● Healthy │  │
│ │ 12ms     │ │ 8ms      │ │ 45ms     │ │ N/A       │  │
│ └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
│                                                         │
│ RECENT ACTIVITY                                         │
│ • [Alice] deployed "DeFi Monitor Agent" (2 min ago)    │
│ • [Bob] published "Social Analytics Tool" (5 min ago)  │
│ • [MCP-01] came online (8 min ago)                     │
│                                                         │
│ NETWORK TOPOLOGY                [View Full Graph]       │
│ [Interactive service connection visualization]          │
└─────────────────────────────────────────────────────────┘
```

---

## 🎨 Design System

### Color Palette
```scss
// Primary - Brand colors
$crossroads-primary: #6366f1;    // Indigo
$crossroads-secondary: #8b5cf6;  // Purple
$crossroads-accent: #ec4899;     // Pink

// Status colors
$status-healthy: #10b981;        // Green
$status-warning: #f59e0b;        // Amber
$status-error: #ef4444;          // Red
$status-inactive: #6b7280;       // Gray

// Category colors
$category-agent: #3b82f6;        // Blue
$category-workflow: #8b5cf6;     // Purple
$category-tool: #f59e0b;         // Amber
$category-mcp: #10b981;          // Green
$category-dataset: #06b6d4;      // Cyan
$category-model: #ec4899;        // Pink
```

### Typography
```scss
// Headers
$font-display: 'Inter', -apple-system, sans-serif;
$font-body: 'Inter', -apple-system, sans-serif;
$font-mono: 'JetBrains Mono', 'Fira Code', monospace;

// Sizes
$text-xs: 0.75rem;    // 12px
$text-sm: 0.875rem;   // 14px
$text-base: 1rem;     // 16px
$text-lg: 1.125rem;   // 18px
$text-xl: 1.25rem;    // 20px
$text-2xl: 1.5rem;    // 24px
$text-3xl: 1.875rem;  // 30px
$text-4xl: 2.25rem;   // 36px
```

### Spacing & Layout
```scss
$grid-gap: 1.5rem;              // 24px between cards
$sidebar-width: 280px;          // Filter sidebar
$panel-width: 360px;            // Deployment panel
$card-radius: 0.75rem;          // 12px rounded corners
$modal-radius: 1rem;            // 16px for modals
```

---

## 🔗 OnchainKit Integration Points

### 1. **Identity Components**
```tsx
// Service owner display
import { Identity, Avatar, Name, Badge } from '@coinbase/onchainkit/identity';

<Identity address={service.owner_address} chain={base}>
  <Avatar />
  <Name />
  <Badge />
</Identity>
```

### 2. **Wallet Connection**
```tsx
import { ConnectWallet } from '@coinbase/onchainkit/wallet';

<ConnectWallet>
  <Avatar />
  <Name />
  <Badge />
</ConnectWallet>
```

### 3. **Transaction Flows**
```tsx
import { Transaction } from '@coinbase/onchainkit/transaction';

// For paid services
<Transaction
  contracts={[{
    address: MARKETPLACE_CONTRACT,
    abi: marketplaceAbi,
    functionName: 'purchaseService',
    args: [serviceId, pricingTier]
  }]}
  onSuccess={handleDeployment}
/>
```

### 4. **Swap Integration** (for token payments)
```tsx
import { Swap } from '@coinbase/onchainkit/swap';

// Allow users to pay in any token
<Swap>
  <SwapAmountInput />
  <SwapToggleButton />
  <SwapAmountInput />
  <SwapButton />
</Swap>
```

---

## 📊 State Management

### Global Context
```tsx
interface CrossroadsContext {
  // Marketplace state
  services: Service[];
  filters: FilterState;
  searchQuery: string;
  viewMode: 'grid' | 'list';
  
  // User state
  userServices: UserService[];
  deployedServices: DeployedService[];
  favorites: string[];
  
  // Actions
  fetchServices: () => Promise<void>;
  searchServices: (query: string) => Promise<void>;
  filterServices: (filters: FilterState) => void;
  publishService: (service: ServiceInput) => Promise<void>;
  deployService: (serviceId: string) => Promise<void>;
}
```

---

## 🔄 User Flows

### Flow 1: Discover & Deploy a Service
```
1. User lands on Crossroads (unauthenticated)
2. Sees trending services and features
3. Clicks "Connect Wallet" → OnchainKit wallet modal
4. Wallet connected → redirected to Marketplace Browser
5. Browses services, applies filters
6. Clicks service card → Service Detail View
7. Reviews details, configuration, reviews
8. Clicks "Deploy Now" → checks requirements
9. If paid: Transaction modal (OnchainKit)
10. Service deploys → redirected to "My Services"
11. Can launch/configure deployed service
```

### Flow 2: Publish a Service
```
1. User navigates to "Publish" tab
2. Enters publish wizard
3. Selects service type (e.g., Agent)
4. Fills basic information form
5. Provides technical configuration
6. Sets pricing model
7. Reviews preview
8. Submits for approval/publishes
9. Service appears in "My Published Services"
10. Can track analytics, earnings, reviews
```

### Flow 3: Service Discovery Scan
```
1. User on Discovery Dashboard
2. Clicks "Scan Now" button
3. Backend triggers discovery across:
   - nullblock-agents API
   - nullblock-protocols MCP servers
   - Registered external services
4. Real-time updates appear in dashboard
5. New services auto-added to marketplace
6. Health status updates continuously
```

---

## 🎯 Key Interactions

### Search & Filter
- **Instant search** as user types (debounced 300ms)
- **Multi-select filters** with count badges
- **Sort options**: Trending, Rating, Recent, Price
- **Clear all filters** button
- **Save filter presets** for power users

### Service Cards
- **Hover effects**: Slight lift, shadow increase
- **Quick actions**: Pin/favorite, share, quick preview
- **Status indicators**: Active (green dot), Maintenance (amber), Offline (gray)
- **Category badges**: Color-coded, clickable to filter

### Deployment
- **One-click deploy** for free services
- **Guided setup** for complex services
- **Sandbox mode** to test before deploying
- **Configuration presets** for common setups
- **Health monitoring** post-deployment

---

## 🔧 Technical Implementation

### Component Structure
```
svc/hecate/src/
├── components/
│   ├── crossroads/
│   │   ├── landing/
│   │   │   ├── CrossroadsHero.tsx
│   │   │   ├── FeatureShowcase.tsx
│   │   │   ├── TrendingServices.tsx
│   │   │   └── StatsOverview.tsx
│   │   ├── marketplace/
│   │   │   ├── MarketplaceBrowser.tsx
│   │   │   ├── ServiceGrid.tsx
│   │   │   ├── ServiceCard.tsx
│   │   │   ├── FilterSidebar.tsx
│   │   │   └── CategoryTabs.tsx
│   │   ├── service/
│   │   │   ├── ServiceDetailView.tsx
│   │   │   ├── ServiceHero.tsx
│   │   │   ├── ServiceTabs.tsx
│   │   │   ├── DeploymentPanel.tsx
│   │   │   ├── ReviewsList.tsx
│   │   │   └── AnalyticsView.tsx
│   │   ├── publish/
│   │   │   ├── PublishWizard.tsx
│   │   │   ├── ServiceTypeSelector.tsx
│   │   │   ├── MetadataForm.tsx
│   │   │   ├── ConfigurationForm.tsx
│   │   │   ├── PricingForm.tsx
│   │   │   └── PreviewPanel.tsx
│   │   ├── dashboard/
│   │   │   ├── MyServicesDashboard.tsx
│   │   │   ├── PublishedServices.tsx
│   │   │   ├── DeployedServices.tsx
│   │   │   └── ServiceAnalytics.tsx
│   │   ├── discovery/
│   │   │   ├── DiscoveryDashboard.tsx
│   │   │   ├── ServiceHealthGrid.tsx
│   │   │   ├── NetworkTopology.tsx
│   │   │   └── RecentActivity.tsx
│   │   ├── shared/
│   │   │   ├── OwnerIdentity.tsx
│   │   │   ├── PriceDisplay.tsx
│   │   │   ├── StatusBadge.tsx
│   │   │   ├── RatingDisplay.tsx
│   │   │   └── CategoryBadge.tsx
│   │   └── Crossroads.tsx (main router)
│   └── ...
├── styles/
│   ├── crossroads/
│   │   ├── _variables.scss
│   │   ├── _mixins.scss
│   │   ├── marketplace.module.scss
│   │   ├── service-detail.module.scss
│   │   ├── publish-wizard.module.scss
│   │   └── discovery.module.scss
│   └── ...
└── ...
```

### API Integration Pattern
```tsx
// Custom hook for marketplace data
export const useMarketplace = () => {
  const [services, setServices] = useState<Service[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchServices = async (filters?: FilterState) => {
    setLoading(true);
    try {
      const params = new URLSearchParams();
      if (filters?.category) params.append('category', filters.category);
      if (filters?.search) params.append('q', filters.search);
      
      const response = await fetch(`/api/marketplace/listings?${params}`);
      const data = await response.json();
      setServices(data.listings);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  };

  return { services, loading, error, fetchServices };
};
```

---

## 📋 Backend Requirements (Follow-up TODOs)

### Database Schema

#### 1. **listings** table
```sql
CREATE TABLE crossroads_listings (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  listing_type VARCHAR(50) NOT NULL, -- Agent, Workflow, Tool, McpServer, Dataset, Model
  title VARCHAR(255) NOT NULL,
  short_description TEXT,
  long_description TEXT,
  owner_address VARCHAR(42) NOT NULL,
  owner_user_id UUID REFERENCES users(id),
  
  -- Technical details
  endpoint_url TEXT,
  health_check_url TEXT,
  documentation_url TEXT,
  repository_url TEXT,
  version VARCHAR(50),
  
  -- Configuration
  configuration_schema JSONB,
  capabilities TEXT[],
  supported_models TEXT[],
  protocol_version VARCHAR(50),
  
  -- Marketplace
  category_tags TEXT[],
  icon_url TEXT,
  screenshots TEXT[],
  
  -- Pricing
  is_free BOOLEAN DEFAULT true,
  pricing_model VARCHAR(50), -- Free, Subscription, OneTime, PayPerUse, TokenStaking
  price_usd DECIMAL(10,2),
  price_eth DECIMAL(18,8),
  
  -- Status
  status VARCHAR(50) DEFAULT 'pending', -- pending, active, inactive, rejected
  is_featured BOOLEAN DEFAULT false,
  verification_status VARCHAR(50) DEFAULT 'unverified',
  
  -- Metadata
  metadata JSONB,
  
  -- Stats (denormalized for performance)
  deployment_count INTEGER DEFAULT 0,
  rating_average DECIMAL(3,2),
  rating_count INTEGER DEFAULT 0,
  view_count INTEGER DEFAULT 0,
  favorite_count INTEGER DEFAULT 0,
  
  -- Timestamps
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  published_at TIMESTAMPTZ,
  last_health_check TIMESTAMPTZ
);

CREATE INDEX idx_listings_type ON crossroads_listings(listing_type);
CREATE INDEX idx_listings_owner ON crossroads_listings(owner_address);
CREATE INDEX idx_listings_status ON crossroads_listings(status);
CREATE INDEX idx_listings_featured ON crossroads_listings(is_featured) WHERE is_featured = true;
CREATE INDEX idx_listings_tags ON crossroads_listings USING GIN(category_tags);
CREATE INDEX idx_listings_search ON crossroads_listings USING GIN(to_tsvector('english', title || ' ' || COALESCE(short_description, '')));
```

#### 2. **reviews** table
```sql
CREATE TABLE crossroads_reviews (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  listing_id UUID NOT NULL REFERENCES crossroads_listings(id) ON DELETE CASCADE,
  reviewer_address VARCHAR(42) NOT NULL,
  reviewer_user_id UUID REFERENCES users(id),
  
  rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
  title VARCHAR(255),
  content TEXT,
  
  -- Verification
  is_verified_purchase BOOLEAN DEFAULT false,
  
  -- Engagement
  helpful_count INTEGER DEFAULT 0,
  
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_reviews_listing ON crossroads_reviews(listing_id);
CREATE INDEX idx_reviews_rating ON crossroads_reviews(rating);
```

#### 3. **deployments** table
```sql
CREATE TABLE crossroads_deployments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  listing_id UUID NOT NULL REFERENCES crossroads_listings(id),
  deployer_address VARCHAR(42) NOT NULL,
  deployer_user_id UUID REFERENCES users(id),
  
  -- Instance details
  instance_name VARCHAR(255),
  instance_endpoint TEXT,
  configuration JSONB,
  
  -- Status
  status VARCHAR(50) DEFAULT 'deploying', -- deploying, running, stopped, failed
  health_status VARCHAR(50) DEFAULT 'unknown',
  last_health_check TIMESTAMPTZ,
  
  -- Billing (if applicable)
  payment_tx_hash VARCHAR(66),
  subscription_expires_at TIMESTAMPTZ,
  
  -- Metrics
  uptime_percentage DECIMAL(5,2),
  total_requests INTEGER DEFAULT 0,
  
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  stopped_at TIMESTAMPTZ
);

CREATE INDEX idx_deployments_listing ON crossroads_deployments(listing_id);
CREATE INDEX idx_deployments_deployer ON crossroads_deployments(deployer_address);
CREATE INDEX idx_deployments_status ON crossroads_deployments(status);
```

#### 4. **favorites** table
```sql
CREATE TABLE crossroads_favorites (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  listing_id UUID NOT NULL REFERENCES crossroads_listings(id) ON DELETE CASCADE,
  user_address VARCHAR(42) NOT NULL,
  user_id UUID REFERENCES users(id),
  
  created_at TIMESTAMPTZ DEFAULT NOW(),
  
  UNIQUE(listing_id, user_address)
);

CREATE INDEX idx_favorites_user ON crossroads_favorites(user_address);
CREATE INDEX idx_favorites_listing ON crossroads_favorites(listing_id);
```

#### 5. **discovery_scans** table
```sql
CREATE TABLE crossroads_discovery_scans (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  scan_type VARCHAR(50) NOT NULL, -- full, agents, mcp, workflows
  status VARCHAR(50) DEFAULT 'running',
  
  services_discovered INTEGER DEFAULT 0,
  services_updated INTEGER DEFAULT 0,
  services_removed INTEGER DEFAULT 0,
  
  results JSONB,
  error_message TEXT,
  
  started_at TIMESTAMPTZ DEFAULT NOW(),
  completed_at TIMESTAMPTZ
);

CREATE INDEX idx_discovery_scans_status ON crossroads_discovery_scans(status);
CREATE INDEX idx_discovery_scans_started ON crossroads_discovery_scans(started_at DESC);
```

---

### API Endpoints to Implement

#### Marketplace Listings
```
GET    /api/marketplace/listings
  - Query params: category, search, tags[], is_free, min_rating, sort_by, page, per_page
  - Returns: paginated list of listings with stats

GET    /api/marketplace/listings/:id
  - Returns: full listing details with owner info, stats, related services

POST   /api/marketplace/listings
  - Body: CreateListingRequest
  - Auth: Required (wallet signature)
  - Returns: created listing

PUT    /api/marketplace/listings/:id
  - Body: UpdateListingRequest
  - Auth: Required (owner only)
  - Returns: updated listing

DELETE /api/marketplace/listings/:id
  - Auth: Required (owner only)
  - Returns: success message

POST   /api/marketplace/search
  - Body: advanced search criteria
  - Returns: search results with highlighting
```

#### Reviews
```
GET    /api/marketplace/listings/:id/reviews
  - Query params: sort_by, page, per_page
  - Returns: paginated reviews

POST   /api/marketplace/listings/:id/reviews
  - Body: { rating, title, content }
  - Auth: Required
  - Returns: created review

PUT    /api/marketplace/reviews/:id/helpful
  - Auth: Required
  - Returns: updated helpful count
```

#### Deployments
```
POST   /api/marketplace/listings/:id/deploy
  - Body: { instance_name, configuration }
  - Auth: Required
  - Returns: deployment record

GET    /api/marketplace/deployments
  - Auth: Required (user's deployments only)
  - Returns: list of user's deployments

GET    /api/marketplace/deployments/:id
  - Auth: Required (owner only)
  - Returns: deployment details with metrics

POST   /api/marketplace/deployments/:id/start
POST   /api/marketplace/deployments/:id/stop
DELETE /api/marketplace/deployments/:id
```

#### Favorites
```
GET    /api/marketplace/favorites
  - Auth: Required
  - Returns: user's favorite listings

POST   /api/marketplace/listings/:id/favorite
DELETE /api/marketplace/listings/:id/favorite
  - Auth: Required
  - Returns: success message
```

#### Analytics
```
GET    /api/marketplace/listings/:id/analytics
  - Auth: Required (owner only)
  - Query params: time_range (7d, 30d, 90d, all)
  - Returns: views, deployments, revenue, ratings over time

GET    /api/marketplace/stats
  - Returns: marketplace-wide statistics
```

#### Discovery
```
POST   /api/discovery/scan
  - Body: { scan_type: 'full' | 'agents' | 'mcp' | 'workflows' }
  - Auth: Optional (rate-limited for anon)
  - Returns: scan_id, status

GET    /api/discovery/scan/:id
  - Returns: scan status and results
```

---

### Business Logic

#### Service Discovery
- **Periodic scans**: Run every 5 minutes
- **Auto-registration**: New services detected → create listing as "pending"
- **Health checks**: Every 2 minutes for active services
- **Stale detection**: Mark inactive if unhealthy for 30 minutes

#### Listing Approval Workflow
- **Auto-approve**: Services from verified publishers
- **Manual review**: First-time publishers or flagged content
- **Moderation queue**: Admin dashboard for approval/rejection
- **Appeals**: Allow resubmission with changes

#### Pricing & Payments
- **Free services**: No transaction required
- **Paid services**: Smart contract integration
  - Escrow for subscriptions
  - Revenue split (platform fee: 5%)
  - Automatic payouts to publishers

#### Rating System
- **Verified reviews**: Only from users who deployed the service
- **Rating calculation**: Weighted by deployment duration
- **Anti-spam**: Rate limit, verification required
- **Moderation**: Flag inappropriate reviews

---

## 🚀 Phased Implementation

### Phase 1: Core UI (Week 1)
- ✅ Landing page with hero and features
- ✅ Marketplace browser with grid view
- ✅ Basic service cards
- ✅ Simple search and category filter
- ✅ Service detail view (read-only)
- ✅ OnchainKit wallet integration

### Phase 2: Backend Foundation (Week 2)
- ✅ PostgreSQL schema creation
- ✅ CRUD API endpoints for listings
- ✅ Search and filter implementation
- ✅ Discovery service integration
- ✅ Health check system

### Phase 3: Publishing & Management (Week 3)
- ✅ Publish wizard UI
- ✅ Service submission API
- ✅ My Services dashboard
- ✅ Edit/update functionality
- ✅ Basic analytics

### Phase 4: Social & Engagement (Week 4)
- ✅ Reviews and ratings
- ✅ Favorites system
- ✅ User profiles with OnchainKit Identity
- ✅ Service sharing
- ✅ Activity feeds

### Phase 5: Deployments (Week 5)
- ✅ Deployment API
- ✅ Configuration management
- ✅ Instance monitoring
- ✅ Start/stop controls
- ✅ Health tracking

### Phase 6: Monetization (Week 6)
- ✅ Smart contract integration
- ✅ Payment flows with OnchainKit Transaction
- ✅ Subscription management
- ✅ Revenue tracking
- ✅ Payout system

---

## 🎨 Visual Examples

### Service Card States
```
┌──────────────────┐
│ [Icon]  ⭐ NEW   │  <- Featured badge
│                  │
│ Service Name     │
│ Short desc here  │
│                  │
│ [Avatar] Name    │
│                  │
│ ⭐ 4.8  🚀 234   │
│                  │
│ Free             │
│ [View Details]   │
└──────────────────┘

Hover state: Lift shadow, scale 1.02
Click: Navigate to detail view
Quick actions: ❤️ Favorite, 🔗 Share
```

### Empty States
```
No services found
[Illustration]
Try adjusting your filters or search terms
[Clear Filters Button]
```

### Loading States
```
[Skeleton cards with shimmer effect]
Loading marketplace services...
```

### Error States
```
⚠️ Failed to load services
Network error or service unavailable
[Retry Button]
```

---

## 📱 Responsive Design

### Mobile (< 768px)
- Stack layout (no sidebar)
- Filters in modal/drawer
- Single column grid
- Bottom navigation
- Simplified cards

### Tablet (768px - 1024px)
- 2-column grid
- Collapsible sidebar
- Compact navigation
- Touch-optimized

### Desktop (> 1024px)
- Full layout as designed
- Persistent sidebar
- 4-column grid
- Hover interactions
- Keyboard shortcuts

---

## ♿ Accessibility

- **ARIA labels**: All interactive elements
- **Keyboard navigation**: Tab order, shortcuts
- **Screen reader support**: Semantic HTML, descriptions
- **Color contrast**: WCAG AA compliance
- **Focus indicators**: Clear focus states
- **Alt text**: All images and icons

---

## 🔐 Security Considerations

- **Wallet signature verification**: All mutations
- **Rate limiting**: Search, deployment, publishing
- **Input validation**: XSS prevention, SQL injection
- **Content moderation**: Review spam, malicious services
- **Sandboxing**: Demo/test environments isolated
- **CORS**: Proper origin validation

---

## 📈 Analytics & Tracking

### Events to Track
- Service views
- Search queries
- Filter usage
- Deployments
- Reviews submitted
- Favorites added
- Service publishes
- Transactions completed

### Metrics to Monitor
- Marketplace conversion rate
- Average time to deploy
- Top categories
- User retention
- Revenue metrics
- Service health trends

---

This design provides a modern, Web3-native marketplace experience. Ready to start implementing? Let me know which phase or component you'd like to tackle first!

