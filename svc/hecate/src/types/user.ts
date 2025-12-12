export interface UserProfile {
  id: string;
  source_identifier: string;
  network: string;
  user_type: string;
  source_type: {
    type: string;
    provider?: string;
    metadata?: Record<string, unknown>;
  };
  email?: string;
  metadata?: Record<string, unknown>;
  preferences?: Record<string, unknown>;
  is_active: boolean;
  created_at: string;
  updated_at?: string;
}

export interface UserLookupRequest {
  source_identifier: string;
  network: string;
}

export interface UserLookupResponse {
  success: boolean;
  data?: UserProfile;
  error?: string;
  message?: string;
}

export type ApiKeyProvider =
  | 'openai'
  | 'anthropic'
  | 'groq'
  | 'openrouter'
  | 'huggingface'
  | 'ollama';

export interface ApiKeyResponse {
  id: string;
  user_id: string;
  provider: string;
  key_prefix?: string;
  key_suffix?: string;
  key_name?: string;
  last_used_at?: string;
  usage_count: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateApiKeyRequest {
  provider: string;
  api_key: string;
  key_name?: string;
}

export interface ApiKeyListResponse {
  success: boolean;
  data?: ApiKeyResponse[];
  total?: number;
  error?: string;
  message?: string;
  timestamp?: string;
}

export interface ApiKeySingleResponse {
  success: boolean;
  data?: ApiKeyResponse;
  error?: string;
  message?: string;
}
