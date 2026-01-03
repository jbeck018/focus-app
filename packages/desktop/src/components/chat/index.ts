// components/chat/index.ts
// Clean exports for all chat components

// Container
export {
  ChatContainer,
  ScrollToBottomButton,
  type ChatContainerProps,
  type ChatContainerRef,
} from "./chat-container";

// Messages
export {
  ChatMessage,
  ChatMessageGroup,
  CodeBlock,
  type ChatMessageProps,
  type MessageRole,
} from "./chat-message";

// Input
export {
  ChatInput,
  ChatInputSimple,
  type ChatInputProps,
  type ChatInputSimpleProps,
} from "./chat-input";

// Suggestions
export {
  ChatSuggestions,
  SuggestionChip,
  FloatingSuggestions,
  FOCUS_SUGGESTIONS,
  QUICK_ACTIONS,
  type ChatSuggestionsProps,
  type Suggestion,
} from "./chat-suggestions";

// Loading/Thinking
export {
  ChatThinking,
  ChatThinkingMinimal,
  ChatMessageSkeleton,
  ThinkingDots,
  type ChatThinkingProps,
} from "./chat-thinking";

// Reasoning
export {
  ChatReasoning,
  InlineReasoning,
  StreamingReasoning,
  ReasoningBadge,
  type ChatReasoningProps,
} from "./chat-reasoning";

// Chat History
export {
  ChatHistoryPanel,
  type ChatHistoryPanelProps,
} from "./ChatHistoryPanel";

export {
  ChatHistoryToggle,
  type ChatHistoryToggleProps,
} from "./ChatHistoryToggle";

export {
  ConversationListItem,
  type ConversationListItemProps,
} from "./ConversationListItem";
