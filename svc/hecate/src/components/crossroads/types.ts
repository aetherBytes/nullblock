import type { ServiceCategory } from './shared/CategoryBadge';
import type { ServiceStatus } from './shared/StatusBadge';

export interface ServiceListing {
  id: string;
  title: string;
  short_description: string;
  long_description?: string;
  listing_type: ServiceCategory;
  owner_address: string;
  owner_ens?: string;
  endpoint_url?: string;
  health_check_url?: string;
  documentation_url?: string;
  version: string;
  capabilities: string[];
  category_tags: string[];
  icon_url?: string;
  is_free: boolean;
  price_usd?: number;
  price_eth?: number;
  pricing_model?: 'Free' | 'Subscription' | 'OneTime' | 'PayPerUse' | 'TokenStaking';
  status: 'pending' | 'active' | 'inactive' | 'rejected';
  is_featured: boolean;
  health_status: ServiceStatus;
  rating_average?: number;
  rating_count: number;
  deployment_count: number;
  view_count: number;
  favorite_count: number;
  is_favorited?: boolean;
  is_coming_soon?: boolean;
  created_at: string;
  updated_at: string;
}

export interface FilterState {
  category?: ServiceCategory;
  search?: string;
  tags?: string[];
  is_free?: boolean;
  min_rating?: number;
  sort_by?: 'trending' | 'rating' | 'recent' | 'price';
}

export interface PaginatedListings {
  listings: ServiceListing[];
  total_count: number;
  page: number;
  per_page: number;
  total_pages: number;
}

