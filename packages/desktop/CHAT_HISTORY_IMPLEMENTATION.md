# Chat History Implementation

This document describes the chat history feature implementation for FocusFlow's AI Coach.

## Overview

The chat history feature allows users to:
- View past conversations with the AI Coach (last 30 days)
- Load and review historical conversations
- Continue from a past conversation
- Delete old conversations
- Start new conversations

## Architecture

### Type Definitions

**Location:** `/packages/types/src/coach.ts`

New types added:
- `Conversation` - Conversation metadata (id, title, dates, message count)
- `ConversationDetail` - Full conversation with messages
- `ConversationListResponse` - Paginated conversation list
- `ListConversationsRequest` - Parameters for listing conversations
- `CreateConversationRequest` - Parameters for creating conversations

### React Hooks

**Location:** `/packages/desktop/src/hooks/useChatHistory.ts`

Provides React Query-based hooks for conversation management:

```typescript
// List conversations with pagination
useConversations(params?: ListConversationsRequest)

// Get single conversation with messages
useConversation(id: string | null)

// Create new conversation
useCreateConversation()

// Delete conversation
useDeleteConversation()

// Update conversation title
useUpdateConversationTitle()

// Add message to conversation
useAddMessageToConversation()
```

All hooks use React Query for caching, optimistic updates, and automatic refetching.

### UI Components

#### 1. ChatHistoryToggle
**Location:** `/packages/desktop/src/components/chat/ChatHistoryToggle.tsx`

A button to toggle the chat history panel open/closed.

**Features:**
- Shows conversation count badge
- Tooltip with conversation count
- Active state indicator
- Accessible (ARIA labels)

**Props:**
```typescript
{
  onClick?: () => void;
  conversationCount?: number;
  isActive?: boolean;
  className?: string;
}
```

#### 2. ConversationListItem
**Location:** `/packages/desktop/src/components/chat/ConversationListItem.tsx`

Individual conversation item in the history list.

**Features:**
- Shows title, last message preview, date, message count
- Active state highlighting
- Delete confirmation dialog
- Keyboard navigation support
- Relative time formatting (e.g., "2h ago", "3d ago")

**Props:**
```typescript
{
  conversation: Conversation;
  isActive?: boolean;
  onClick?: (id: string) => void;
  onDelete?: (id: string) => void;
  className?: string;
}
```

#### 3. ChatHistoryPanel
**Location:** `/packages/desktop/src/components/chat/ChatHistoryPanel.tsx`

Slide-out panel showing conversation list.

**Features:**
- Sheet/drawer component (mobile-friendly)
- Scrollable conversation list
- "New Chat" button in header
- Loading skeletons
- Empty state (no conversations)
- Error state with retry
- Shows total conversation count
- Pagination support

**Props:**
```typescript
{
  open: boolean;
  onOpenChange: (open: boolean) => void;
  activeConversationId?: string | null;
  onConversationSelect?: (id: string) => void;
  onNewChat?: () => void;
  className?: string;
}
```

### Integration with AICoach

**Location:** `/packages/desktop/src/features/AICoach.tsx`

The AICoach component has been updated with:

1. **State Management:**
   - `historyOpen` - Controls history panel visibility
   - `activeConversationId` - Currently selected conversation
   - `isViewingHistory` - Read-only mode when viewing past conversations

2. **Conversation Loading:**
   - When a conversation is selected, messages are loaded from history
   - Conversation is set to read-only mode initially

3. **View History Banner:**
   - Shows when viewing a historical conversation
   - Displays conversation date
   - Provides "Continue Conversation" button to enable editing
   - Provides "New Chat" button to start fresh

4. **New Chat Flow:**
   - Clears active conversation
   - Resets to editable mode
   - Shows welcome message

5. **Header Integration:**
   - Chat history toggle button added to header
   - Shows conversation count badge

## Tauri Backend Commands

The following Tauri commands need to be implemented in the Rust backend:

### `list_conversations`
Lists conversations with pagination.

**Request:**
```typescript
{
  limit?: number;      // Default: 20
  offset?: number;     // Default: 0
  daysBack?: number;   // Default: 30
}
```

**Response:**
```typescript
{
  conversations: Conversation[];
  total: number;
  hasMore: boolean;
}
```

### `get_conversation`
Gets a single conversation with all messages.

**Request:**
```typescript
{
  conversationId: string;
}
```

**Response:**
```typescript
ConversationDetail
```

### `create_conversation`
Creates a new conversation.

**Request:**
```typescript
{
  title?: string;
  initialMessage?: string;
}
```

**Response:**
```typescript
Conversation
```

### `delete_conversation`
Deletes a conversation.

**Request:**
```typescript
{
  conversationId: string;
}
```

**Response:**
```typescript
void
```

### `update_conversation_title`
Updates conversation title.

**Request:**
```typescript
{
  conversationId: string;
  title: string;
}
```

**Response:**
```typescript
Conversation
```

### `add_message_to_conversation`
Adds a message to an existing conversation.

**Request:**
```typescript
{
  conversationId: string;
  message: string;
  role: "user" | "assistant";
}
```

**Response:**
```typescript
ConversationDetail
```

## Database Schema Recommendations

Suggested tables for the backend:

### `conversations`
```sql
CREATE TABLE conversations (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  created_at DATETIME NOT NULL,
  updated_at DATETIME NOT NULL,
  message_count INTEGER DEFAULT 0
);
```

### `conversation_messages`
```sql
CREATE TABLE conversation_messages (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  conversation_id TEXT NOT NULL,
  role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
  content TEXT NOT NULL,
  timestamp DATETIME NOT NULL,
  FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
```

### Indexes
```sql
CREATE INDEX idx_conversations_updated_at ON conversations(updated_at DESC);
CREATE INDEX idx_messages_conversation_id ON conversation_messages(conversation_id);
CREATE INDEX idx_messages_timestamp ON conversation_messages(timestamp);
```

## User Experience Flow

1. **Initial Load:**
   - User sees AI Coach with welcome message
   - History toggle shows conversation count badge

2. **Opening History:**
   - User clicks history toggle
   - Panel slides in from left
   - Shows recent conversations (last 30 days)
   - Loading skeletons during fetch

3. **Viewing Conversation:**
   - User clicks conversation item
   - Messages load and display
   - Input is disabled (read-only mode)
   - Banner shows "Viewing conversation from [date]"

4. **Continuing Conversation:**
   - User clicks "Continue Conversation"
   - Input becomes enabled
   - User can send new messages
   - Conversation updates in real-time

5. **Starting New Chat:**
   - User clicks "New Chat" button
   - Current conversation clears
   - Welcome message appears
   - Ready for new conversation

6. **Deleting Conversation:**
   - User clicks more menu on conversation
   - Selects "Delete"
   - Confirmation dialog appears
   - On confirm, conversation is deleted
   - If active conversation, switches to new chat

## Styling

All components use:
- shadcn/ui component library
- Tailwind CSS for styling
- Consistent design tokens
- Mobile-responsive layouts
- Dark mode support
- Accessibility features (ARIA labels, keyboard navigation)

## Accessibility

- **Keyboard Navigation:** All interactive elements are keyboard accessible
- **Screen Readers:** Proper ARIA labels and roles
- **Focus Management:** Sheet/dialog components trap focus appropriately
- **Color Contrast:** Meets WCAG 2.1 AA standards
- **Loading States:** Announced to screen readers

## Testing Considerations

When backend is implemented, test:

1. **Pagination:** Ensure conversations load in pages
2. **Real-time Updates:** New messages update conversation list
3. **Deletion:** Cascade deletes work correctly
4. **Date Filtering:** Only last 30 days are shown
5. **Empty States:** Proper handling when no conversations
6. **Error States:** Graceful degradation on API failures
7. **Performance:** Large conversation lists render smoothly
8. **Mobile:** Panel works on small screens

## Next Steps

1. **Backend Implementation:** Implement all Tauri commands
2. **Database Migration:** Create conversation tables
3. **Auto-Title Generation:** Generate conversation titles from first message
4. **Search:** Add search functionality to find conversations
5. **Archive:** Allow archiving old conversations
6. **Export:** Export conversations to markdown/JSON
7. **Analytics:** Track conversation engagement metrics
