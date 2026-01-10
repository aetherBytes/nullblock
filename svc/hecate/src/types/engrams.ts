export type EngramType = 'persona' | 'preference' | 'strategy' | 'knowledge' | 'compliance';

export interface Engram {
  id: string;
  wallet_address: string;
  engram_type: EngramType;
  key: string;
  content: string;
  metadata?: Record<string, unknown>;
  version: number;
  is_public: boolean;
  parent_id?: string;
  created_at: string;
  updated_at: string;
}

export interface EngramListResponse {
  success: boolean;
  data?: Engram[];
  total?: number;
  error?: string;
  message?: string;
}

export interface EngramResponse {
  success: boolean;
  data?: Engram;
  error?: string;
  message?: string;
}

export interface CreateEngramRequest {
  wallet_address: string;
  engram_type: EngramType;
  key: string;
  content: string;
  metadata?: Record<string, unknown>;
}

export interface UpdateEngramRequest {
  content?: string;
  metadata?: Record<string, unknown>;
}

export const ENGRAM_TYPE_LABELS: Record<EngramType, string> = {
  persona: 'Persona',
  preference: 'Preference',
  strategy: 'Strategy',
  knowledge: 'Knowledge',
  compliance: 'Compliance',
};

export const ENGRAM_TYPE_ICONS: Record<EngramType, string> = {
  persona: '👤',
  preference: '⚙️',
  strategy: '🎯',
  knowledge: '📚',
  compliance: '🔒',
};

export const ENGRAM_TYPE_COLORS: Record<EngramType, string> = {
  persona: '#8b5cf6',
  preference: '#06b6d4',
  strategy: '#f59e0b',
  knowledge: '#22c55e',
  compliance: '#ef4444',
};
