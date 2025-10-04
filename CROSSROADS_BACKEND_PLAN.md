# Crossroads Backend Implementation Plan

## 🎯 Overview

This document outlines the backend work required to support the new Crossroads UI design. Each section maps to frontend features and API requirements.

---

## 📊 Database Schema Changes

### New Tables to Create

#### 1. `crossroads_listings`
**Purpose**: Store all marketplace listings (agents, workflows, tools, MCP servers)

```sql
-- Migration: 001_create_crossroads_listings.sql
CREATE TABLE crossroads_listings (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  
  -- Basic info
  listing_type VARCHAR(50) NOT NULL CHECK (listing_type IN ('Agent', 'Workflow', 'Tool', 'McpServer', 'Dataset', 'Model')),
  title VARCHAR(255) NOT NULL,
  short_description TEXT,
  long_description TEXT,
  
  -- Owner info (links to users table)
  owner_address VARCHAR(42) NOT NULL,
  owner_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
  
  -- Technical details
  endpoint_url TEXT,
  health_check_url TEXT,
  documentation_url TEXT,
  repository_url TEXT,
  version VARCHAR(50) DEFAULT '1.0.0',
  
  -- Configuration (flexible JSONB for different types)
  configuration_schema JSONB DEFAULT '{}'::jsonb,
  capabilities TEXT[] DEFAULT ARRAY[]::TEXT[],
  supported_models TEXT[] DEFAULT ARRAY[]::TEXT[],
  protocol_version VARCHAR(50),
  requirements JSONB DEFAULT '{}'::jsonb,
  
  -- Marketplace metadata
  category_tags TEXT[] DEFAULT ARRAY[]::TEXT[],
  icon_url TEXT,
  screenshots TEXT[] DEFAULT ARRAY[]::TEXT[],
  
  -- Pricing
  is_free BOOLEAN DEFAULT true,
  pricing_model VARCHAR(50) DEFAULT 'Free' CHECK (pricing_model IN ('Free', 'Subscription', 'OneTime', 'PayPerUse', 'TokenStaking', 'RevenueShare')),
  price_usd DECIMAL(10,2),
  price_eth DECIMAL(18,8),
  subscription_period VARCHAR(20), -- 'monthly', 'yearly', etc.
  
  -- Status and moderation
  status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'inactive', 'rejected', 'archived')),
  is_featured BOOLEAN DEFAULT false,
  verification_status VARCHAR(50) DEFAULT 'unverified' CHECK (verification_status IN ('unverified', 'pending', 'verified', 'flagged', 'rejected')),
  rejection_reason TEXT,
  
  -- Additional metadata (extensible)
  metadata JSONB DEFAULT '{}'::jsonb,
  
  -- Denormalized stats (updated via triggers/background jobs)
  deployment_count INTEGER DEFAULT 0,
  rating_average DECIMAL(3,2),
  rating_count INTEGER DEFAULT 0,
  view_count INTEGER DEFAULT 0,
  favorite_count INTEGER DEFAULT 0,
  
  -- Timestamps
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  published_at TIMESTAMPTZ,
  last_health_check TIMESTAMPTZ,
  
  -- Constraints
  CONSTRAINT positive_price CHECK (price_usd IS NULL OR price_usd >= 0),
  CONSTRAINT positive_stats CHECK (
    deployment_count >= 0 AND
    rating_count >= 0 AND
    view_count >= 0 AND
    favorite_count >= 0
  )
);

-- Indexes for performance
CREATE INDEX idx_listings_type ON crossroads_listings(listing_type);
CREATE INDEX idx_listings_owner ON crossroads_listings(owner_address);
CREATE INDEX idx_listings_owner_user ON crossroads_listings(owner_user_id) WHERE owner_user_id IS NOT NULL;
CREATE INDEX idx_listings_status ON crossroads_listings(status);
CREATE INDEX idx_listings_active ON crossroads_listings(status, is_featured) WHERE status = 'active';
CREATE INDEX idx_listings_featured ON crossroads_listings(is_featured, created_at DESC) WHERE is_featured = true;
CREATE INDEX idx_listings_tags ON crossroads_listings USING GIN(category_tags);
CREATE INDEX idx_listings_rating ON crossroads_listings(rating_average DESC NULLS LAST) WHERE status = 'active';
CREATE INDEX idx_listings_deployments ON crossroads_listings(deployment_count DESC) WHERE status = 'active';

-- Full-text search index
CREATE INDEX idx_listings_search ON crossroads_listings 
  USING GIN(to_tsvector('english', 
    title || ' ' || 
    COALESCE(short_description, '') || ' ' || 
    COALESCE(long_description, '') || ' ' ||
    array_to_string(category_tags, ' ')
  ));

-- Trigger to update updated_at
CREATE TRIGGER update_listings_updated_at
  BEFORE UPDATE ON crossroads_listings
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();
```

#### 2. `crossroads_reviews`
**Purpose**: User reviews and ratings for listings

```sql
-- Migration: 002_create_crossroads_reviews.sql
CREATE TABLE crossroads_reviews (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  
  listing_id UUID NOT NULL REFERENCES crossroads_listings(id) ON DELETE CASCADE,
  
  -- Reviewer info
  reviewer_address VARCHAR(42) NOT NULL,
  reviewer_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
  
  -- Review content
  rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
  title VARCHAR(255),
  content TEXT,
  
  -- Verification
  is_verified_purchase BOOLEAN DEFAULT false,
  deployment_id UUID REFERENCES crossroads_deployments(id) ON DELETE SET NULL,
  
  -- Engagement
  helpful_count INTEGER DEFAULT 0,
  
  -- Moderation
  is_flagged BOOLEAN DEFAULT false,
  is_hidden BOOLEAN DEFAULT false,
  moderation_note TEXT,
  
  -- Timestamps
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  
  -- Constraints
  CONSTRAINT one_review_per_listing_per_user UNIQUE(listing_id, reviewer_address)
);

CREATE INDEX idx_reviews_listing ON crossroads_reviews(listing_id, created_at DESC);
CREATE INDEX idx_reviews_reviewer ON crossroads_reviews(reviewer_address);
CREATE INDEX idx_reviews_rating ON crossroads_reviews(listing_id, rating);
CREATE INDEX idx_reviews_verified ON crossroads_reviews(listing_id) WHERE is_verified_purchase = true;

-- Trigger to update updated_at
CREATE TRIGGER update_reviews_updated_at
  BEFORE UPDATE ON crossroads_reviews
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();
```

#### 3. `crossroads_deployments`
**Purpose**: Track service deployments and instances

```sql
-- Migration: 003_create_crossroads_deployments.sql
CREATE TABLE crossroads_deployments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  
  listing_id UUID NOT NULL REFERENCES crossroads_listings(id) ON DELETE RESTRICT,
  
  -- Deployer info
  deployer_address VARCHAR(42) NOT NULL,
  deployer_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
  
  -- Instance details
  instance_name VARCHAR(255) NOT NULL,
  instance_endpoint TEXT,
  configuration JSONB DEFAULT '{}'::jsonb,
  
  -- Status
  status VARCHAR(50) DEFAULT 'deploying' CHECK (status IN ('deploying', 'running', 'stopped', 'failed', 'terminated')),
  health_status VARCHAR(50) DEFAULT 'unknown' CHECK (health_status IN ('healthy', 'unhealthy', 'degraded', 'unknown')),
  last_health_check TIMESTAMPTZ,
  health_check_result JSONB,
  error_message TEXT,
  
  -- Billing (if applicable)
  payment_tx_hash VARCHAR(66),
  payment_status VARCHAR(50) DEFAULT 'pending',
  subscription_expires_at TIMESTAMPTZ,
  
  -- Metrics
  uptime_percentage DECIMAL(5,2) DEFAULT 0.00,
  total_requests INTEGER DEFAULT 0,
  total_errors INTEGER DEFAULT 0,
  last_request_at TIMESTAMPTZ,
  
  -- Resource usage
  resource_usage JSONB DEFAULT '{}'::jsonb, -- CPU, memory, network
  
  -- Timestamps
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  deployed_at TIMESTAMPTZ,
  stopped_at TIMESTAMPTZ,
  terminated_at TIMESTAMPTZ
);

CREATE INDEX idx_deployments_listing ON crossroads_deployments(listing_id);
CREATE INDEX idx_deployments_deployer ON crossroads_deployments(deployer_address);
CREATE INDEX idx_deployments_deployer_user ON crossroads_deployments(deployer_user_id) WHERE deployer_user_id IS NOT NULL;
CREATE INDEX idx_deployments_status ON crossroads_deployments(status, created_at DESC);
CREATE INDEX idx_deployments_active ON crossroads_deployments(listing_id, status) WHERE status IN ('running', 'deploying');

-- Trigger to update updated_at
CREATE TRIGGER update_deployments_updated_at
  BEFORE UPDATE ON crossroads_deployments
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();
```

#### 4. `crossroads_favorites`
**Purpose**: User favorites/bookmarks

```sql
-- Migration: 004_create_crossroads_favorites.sql
CREATE TABLE crossroads_favorites (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  
  listing_id UUID NOT NULL REFERENCES crossroads_listings(id) ON DELETE CASCADE,
  
  user_address VARCHAR(42) NOT NULL,
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  
  created_at TIMESTAMPTZ DEFAULT NOW(),
  
  UNIQUE(listing_id, user_address)
);

CREATE INDEX idx_favorites_user ON crossroads_favorites(user_address, created_at DESC);
CREATE INDEX idx_favorites_user_id ON crossroads_favorites(user_id, created_at DESC) WHERE user_id IS NOT NULL;
CREATE INDEX idx_favorites_listing ON crossroads_favorites(listing_id);
```

#### 5. `crossroads_discovery_scans`
**Purpose**: Track discovery scan operations

```sql
-- Migration: 005_create_crossroads_discovery_scans.sql
CREATE TABLE crossroads_discovery_scans (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  
  scan_type VARCHAR(50) NOT NULL CHECK (scan_type IN ('full', 'agents', 'mcp', 'workflows', 'tools')),
  status VARCHAR(50) DEFAULT 'running' CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),
  
  -- Results
  services_discovered INTEGER DEFAULT 0,
  services_updated INTEGER DEFAULT 0,
  services_removed INTEGER DEFAULT 0,
  services_failed INTEGER DEFAULT 0,
  
  results JSONB DEFAULT '{}'::jsonb,
  error_message TEXT,
  
  -- Timestamps
  started_at TIMESTAMPTZ DEFAULT NOW(),
  completed_at TIMESTAMPTZ,
  duration_ms INTEGER
);

CREATE INDEX idx_discovery_scans_status ON crossroads_discovery_scans(status, started_at DESC);
CREATE INDEX idx_discovery_scans_type ON crossroads_discovery_scans(scan_type, started_at DESC);
CREATE INDEX idx_discovery_scans_recent ON crossroads_discovery_scans(started_at DESC);
```

#### 6. `crossroads_analytics_events`
**Purpose**: Track user interactions and events

```sql
-- Migration: 006_create_crossroads_analytics_events.sql
CREATE TABLE crossroads_analytics_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  
  event_type VARCHAR(50) NOT NULL, -- 'view', 'search', 'deploy', 'review', 'favorite'
  
  listing_id UUID REFERENCES crossroads_listings(id) ON DELETE SET NULL,
  user_address VARCHAR(42),
  user_id UUID REFERENCES users(id) ON DELETE SET NULL,
  
  -- Event data
  event_data JSONB DEFAULT '{}'::jsonb,
  
  -- Context
  user_agent TEXT,
  ip_address INET,
  referrer TEXT,
  
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_analytics_type ON crossroads_analytics_events(event_type, created_at DESC);
CREATE INDEX idx_analytics_listing ON crossroads_analytics_events(listing_id, created_at DESC) WHERE listing_id IS NOT NULL;
CREATE INDEX idx_analytics_user ON crossroads_analytics_events(user_address, created_at DESC) WHERE user_address IS NOT NULL;

-- Partition by month for performance (optional)
-- CREATE TABLE crossroads_analytics_events_YYYYMM PARTITION OF crossroads_analytics_events
--   FOR VALUES FROM ('YYYY-MM-01') TO ('YYYY-MM+1-01');
```

#### Helper Function
```sql
-- Migration: 000_create_helper_functions.sql
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

---

## 🔧 Rust Backend Updates

### 1. Models (`svc/erebus/src/resources/crossroads/models.rs`)

Add new structs:

```rust
// Request models
#[derive(Debug, Deserialize)]
pub struct ListingsQueryParams {
    pub category: Option<ListingType>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_free: Option<bool>,
    pub min_rating: Option<f32>,
    pub sort_by: Option<SortBy>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedListings {
    pub listings: Vec<ListingWithStats>,
    pub total_count: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Serialize)]
pub struct ListingWithStats {
    #[serde(flatten)]
    pub listing: Listing,
    pub owner_info: OwnerInfo,
    pub is_favorited: Option<bool>, // If user is authenticated
}

#[derive(Debug, Serialize)]
pub struct OwnerInfo {
    pub address: String,
    pub ens_name: Option<String>,
    pub avatar_url: Option<String>,
}

// Review models
#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub rating: i32,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Review {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub reviewer_address: String,
    pub rating: i32,
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_verified_purchase: bool,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

// Deployment models
#[derive(Debug, Deserialize)]
pub struct DeployServiceRequest {
    pub instance_name: String,
    pub configuration: Value,
}

#[derive(Debug, Serialize)]
pub struct Deployment {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub instance_name: String,
    pub status: DeploymentStatus,
    pub health_status: HealthStatus,
    pub instance_endpoint: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deployed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Deploying,
    Running,
    Stopped,
    Failed,
    Terminated,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
    Unknown,
}
```

### 2. Database Service (`svc/erebus/src/resources/crossroads/db.rs` - NEW)

Create database layer:

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::resources::crossroads::models::*;

pub struct CrossroadsDb {
    pool: PgPool,
}

impl CrossroadsDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_listings(
        &self,
        params: &ListingsQueryParams,
        user_address: Option<&str>,
    ) -> Result<PaginatedListings, sqlx::Error> {
        // Implementation with dynamic query building
    }

    pub async fn get_listing_by_id(
        &self,
        id: Uuid,
        user_address: Option<&str>,
    ) -> Result<Option<ListingWithStats>, sqlx::Error> {
        // Implementation
    }

    pub async fn create_listing(
        &self,
        request: &CreateListingRequest,
        owner_address: &str,
    ) -> Result<Listing, sqlx::Error> {
        // Implementation
    }

    pub async fn update_listing(
        &self,
        id: Uuid,
        owner_address: &str,
        updates: &UpdateListingRequest,
    ) -> Result<Listing, sqlx::Error> {
        // Implementation with ownership check
    }

    // ... more methods for reviews, deployments, favorites, etc.
}
```

### 3. Routes Updates (`svc/erebus/src/resources/crossroads/routes.rs`)

Replace stub implementations with real database calls:

```rust
async fn get_listings(
    State(app_state): State<crate::AppState>,
    Query(params): Query<ListingsQueryParams>,
    auth: Option<AuthUser>, // Custom extractor for authenticated user
) -> Result<Json<PaginatedListings>, StatusCode> {
    let db = CrossroadsDb::new(app_state.db_pool.clone());
    
    let user_address = auth.as_ref().map(|u| u.address.as_str());
    
    match db.get_listings(&params, user_address).await {
        Ok(listings) => Ok(Json(listings)),
        Err(e) => {
            error!("Failed to fetch listings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Similar pattern for other endpoints...
```

---

## 🔍 Search Implementation

### PostgreSQL Full-Text Search

```rust
// In CrossroadsDb
pub async fn search_listings(
    &self,
    query: &str,
    filters: &SearchFilters,
) -> Result<Vec<ListingWithStats>, sqlx::Error> {
    let search_query = sqlx::query_as!(
        Listing,
        r#"
        SELECT l.*, 
               ts_rank(
                 to_tsvector('english', l.title || ' ' || COALESCE(l.short_description, '')),
                 plainto_tsquery('english', $1)
               ) as rank
        FROM crossroads_listings l
        WHERE 
          to_tsvector('english', l.title || ' ' || COALESCE(l.short_description, '')) 
          @@ plainto_tsquery('english', $1)
          AND l.status = 'active'
          AND ($2::text IS NULL OR l.listing_type = $2)
          AND ($3::boolean IS NULL OR l.is_free = $3)
        ORDER BY rank DESC, l.rating_average DESC NULLS LAST
        LIMIT $4 OFFSET $5
        "#,
        query,
        filters.category.as_ref().map(|c| c.to_string()),
        filters.is_free,
        filters.per_page.unwrap_or(20),
        filters.offset.unwrap_or(0)
    )
    .fetch_all(&self.pool)
    .await?;
    
    Ok(search_query)
}
```

---

## 🤖 Service Discovery

### Enhanced Discovery Service

```rust
// In services.rs
impl NullblockServiceIntegrator {
    /// Run full discovery scan and update database
    pub async fn run_discovery_scan(
        &self,
        db: &CrossroadsDb,
        scan_type: ScanType,
    ) -> Result<DiscoveryScanResult, Box<dyn std::error::Error>> {
        let scan_id = Uuid::new_v4();
        
        // Record scan start
        db.create_discovery_scan(scan_id, scan_type).await?;
        
        let mut result = DiscoveryScanResult {
            discovered: 0,
            updated: 0,
            removed: 0,
        };
        
        match scan_type {
            ScanType::Agents => {
                let agents = self.discover_agents_from_service().await?;
                result.discovered = agents.len();
                
                for agent in agents {
                    // Upsert into database
                    db.upsert_discovered_service(agent).await?;
                }
            }
            ScanType::McpServers => {
                let mcp_servers = self.discover_mcp_servers_from_service().await?;
                result.discovered = mcp_servers.len();
                
                for server in mcp_servers {
                    db.upsert_discovered_service(server).await?;
                }
            }
            // ... other scan types
        }
        
        // Update scan record
        db.complete_discovery_scan(scan_id, &result).await?;
        
        Ok(result)
    }
}
```

---

## 📊 Analytics Implementation

### Event Tracking

```rust
// In db.rs
pub async fn track_event(
    &self,
    event_type: &str,
    listing_id: Option<Uuid>,
    user_address: Option<&str>,
    event_data: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO crossroads_analytics_events 
          (event_type, listing_id, user_address, event_data)
        VALUES ($1, $2, $3, $4)
        "#,
        event_type,
        listing_id,
        user_address,
        event_data
    )
    .execute(&self.pool)
    .await?;
    
    Ok(())
}

// In routes - track views
async fn get_listing(
    Path(id): Path<Uuid>,
    State(app_state): State<crate::AppState>,
    auth: Option<AuthUser>,
) -> Result<Json<ListingWithStats>, StatusCode> {
    let db = CrossroadsDb::new(app_state.db_pool.clone());
    
    // Track view
    let user_address = auth.as_ref().map(|u| u.address.as_str());
    let _ = db.track_event("view", Some(id), user_address, json!({})).await;
    
    // Increment view count
    let _ = db.increment_view_count(id).await;
    
    // Fetch listing
    match db.get_listing_by_id(id, user_address).await {
        Ok(Some(listing)) => Ok(Json(listing)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to fetch listing: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
```

---

## 🔐 Authentication & Authorization

### Wallet Signature Verification

```rust
// Custom extractor for authenticated users
pub struct AuthUser {
    pub address: String,
    pub user_id: Option<Uuid>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract signature and message from headers
        let signature = parts
            .headers
            .get("X-Wallet-Signature")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;
            
        let message = parts
            .headers
            .get("X-Wallet-Message")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        // Verify signature and recover address
        let address = verify_wallet_signature(signature, message)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        
        // Optional: Look up user_id from database
        let user_id = lookup_user_id(&address).await;
        
        Ok(AuthUser { address, user_id })
    }
}

// Usage in routes
async fn create_listing(
    State(app_state): State<crate::AppState>,
    auth: AuthUser, // Requires authentication
    Json(request): Json<CreateListingRequest>,
) -> Result<Json<Listing>, StatusCode> {
    let db = CrossroadsDb::new(app_state.db_pool.clone());
    
    match db.create_listing(&request, &auth.address).await {
        Ok(listing) => Ok(Json(listing)),
        Err(e) => {
            error!("Failed to create listing: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
```

---

## 🚀 Deployment System

### Service Deployment Handler

```rust
// In services.rs
impl NullblockServiceIntegrator {
    pub async fn deploy_service(
        &self,
        db: &CrossroadsDb,
        listing_id: Uuid,
        deployer_address: &str,
        config: DeployServiceRequest,
    ) -> Result<Deployment, Box<dyn std::error::Error>> {
        // 1. Fetch listing
        let listing = db.get_listing_by_id(listing_id, None).await?
            .ok_or("Listing not found")?;
        
        // 2. Validate configuration against schema
        validate_config(&listing.configuration_schema, &config.configuration)?;
        
        // 3. Create deployment record
        let deployment = db.create_deployment(
            listing_id,
            deployer_address,
            &config,
        ).await?;
        
        // 4. Trigger actual deployment based on type
        match listing.listing.listing_type {
            ListingType::Agent => {
                self.deploy_agent(&deployment, &config).await?;
            }
            ListingType::Workflow => {
                self.deploy_workflow(&deployment, &config).await?;
            }
            ListingType::McpServer => {
                self.deploy_mcp_server(&deployment, &config).await?;
            }
            _ => {
                return Err("Unsupported deployment type".into());
            }
        }
        
        // 5. Update deployment status
        db.update_deployment_status(deployment.id, DeploymentStatus::Running).await?;
        
        Ok(deployment)
    }
    
    async fn deploy_agent(
        &self,
        deployment: &Deployment,
        config: &DeployServiceRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Call nullblock-agents to spin up new agent instance
        // This could involve Docker, Kubernetes, or direct process spawning
        todo!("Implement agent deployment")
    }
}
```

---

## 📈 Background Jobs

### Periodic Tasks

```rust
// In main.rs or separate worker
use tokio::time::{interval, Duration};

async fn start_background_tasks(app_state: Arc<AppState>) {
    // Discovery scan every 5 minutes
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            
            let db = CrossroadsDb::new(app_state.db_pool.clone());
            let integrator = NullblockServiceIntegrator::new(app_state.external_service.clone());
            
            if let Err(e) = integrator.run_discovery_scan(&db, ScanType::Full).await {
                error!("Discovery scan failed: {}", e);
            }
        }
    });
    
    // Health checks every 2 minutes
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(120));
        loop {
            interval.tick().await;
            
            let db = CrossroadsDb::new(app_state.db_pool.clone());
            if let Err(e) = check_all_service_health(&db).await {
                error!("Health check failed: {}", e);
            }
        }
    });
    
    // Update stats every 10 minutes
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(600));
        loop {
            interval.tick().await;
            
            let db = CrossroadsDb::new(app_state.db_pool.clone());
            if let Err(e) = update_marketplace_stats(&db).await {
                error!("Stats update failed: {}", e);
            }
        }
    });
}
```

---

## 🧪 Testing Strategy

### Unit Tests
- Model serialization/deserialization
- Query builders
- Signature verification

### Integration Tests
- Full API endpoint tests
- Database operations
- Discovery scans

### E2E Tests
- Complete user flows
- Deployment workflows
- Payment flows

---

## 📦 Migration Order

1. `000_create_helper_functions.sql`
2. `001_create_crossroads_listings.sql`
3. `002_create_crossroads_reviews.sql`
4. `003_create_crossroads_deployments.sql`
5. `004_create_crossroads_favorites.sql`
6. `005_create_crossroads_discovery_scans.sql`
7. `006_create_crossroads_analytics_events.sql`

Run with:
```bash
./scripts/run-erebus-migrations.sh
```

---

## ⚙️ Configuration

### Environment Variables

```bash
# .env or config
DATABASE_URL=postgresql://user:pass@localhost:5432/erebus
CROSSROADS_SCAN_INTERVAL=300  # seconds
CROSSROADS_HEALTH_CHECK_INTERVAL=120  # seconds
CROSSROADS_MAX_DEPLOYMENTS_PER_USER=10
CROSSROADS_PLATFORM_FEE_PERCENT=5
```

---

## 📝 API Documentation

Generate OpenAPI spec:

```rust
// Use utoipa crate for automatic API docs
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_listings,
        create_listing,
        get_listing,
        // ... all endpoints
    ),
    components(schemas(
        Listing,
        CreateListingRequest,
        Review,
        Deployment,
        // ... all models
    ))
)]
struct ApiDoc;

// Serve at /api/crossroads/docs
```

---

## 🎯 Priority Order

### Phase 1: Core Backend (Week 2)
1. ✅ Database migrations
2. ✅ Basic CRUD for listings
3. ✅ Search and filter
4. ✅ Authentication middleware

### Phase 2: Discovery & Health (Week 2-3)
1. ✅ Enhanced discovery scans
2. ✅ Health check system
3. ✅ Background jobs
4. ✅ Auto-registration

### Phase 3: Reviews & Social (Week 3-4)
1. ✅ Review system
2. ✅ Favorites
3. ✅ Analytics tracking
4. ✅ User profiles

### Phase 4: Deployments (Week 4-5)
1. ✅ Deployment API
2. ✅ Configuration management
3. ✅ Instance monitoring
4. ✅ Health tracking

### Phase 5: Monetization (Week 5-6)
1. ✅ Payment verification
2. ✅ Subscription tracking
3. ✅ Revenue tracking
4. ✅ Platform fees

---

This plan provides a complete backend foundation for the Crossroads marketplace. Each section can be implemented incrementally while maintaining functionality.

