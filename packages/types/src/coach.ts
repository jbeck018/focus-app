// types/coach.ts - AI Coach types

/** Chat message in the coach conversation */
export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  timestamp: string;
}

/** Coach response with optional suggestions */
export interface CoachResponse {
  message: string;
  suggestions: string[];
}

/** User context for personalized coaching */
export interface UserContext {
  total_focus_hours_today: number;
  sessions_completed_today: number;
  current_streak_days: number;
  top_trigger: string | null;
  average_session_minutes: number;
}

/** Conversation metadata */
export interface Conversation {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  messageCount: number;
  lastMessage?: string;
}

/** Full conversation with messages */
export interface ConversationDetail extends Conversation {
  messages: ChatMessage[];
}

/** Conversation list response */
export interface ConversationListResponse {
  conversations: Conversation[];
  total: number;
  hasMore: boolean;
}

/** Request parameters for listing conversations */
export interface ListConversationsRequest {
  limit?: number;
  offset?: number;
  daysBack?: number; // Last N days, defaults to 30
}

/** Request to create a new conversation */
export interface CreateConversationRequest {
  title?: string;
  initialMessage?: string;
}
