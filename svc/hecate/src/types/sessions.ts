export interface SessionMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  model_used?: string;
}

export interface SessionData {
  session_id: string;
  title: string;
  message_count: number;
  messages: SessionMessage[];
  created_at: string;
  updated_at: string;
}

export interface SessionSummary {
  session_id: string;
  engram_id: string;
  title: string;
  summary?: string;
  message_count: number;
  created_at: string;
  updated_at: string;
  is_pinned: boolean;
}

export interface SessionListResponse {
  success: boolean;
  data: SessionSummary[];
  total: number;
}

export interface SessionResponse {
  success: boolean;
  data: SessionData | SessionSummary;
}
