# Chat History & Memory - Quick Start Guide

## Overview

The chat history system gives FocusFlow's AI coach **memory**. It remembers:
- Previous conversations
- Your preferences and habits
- Patterns it's identified
- Goals you've mentioned

## Basic Usage (Frontend)

### 1. Start a New Conversation

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Get or create the active conversation
const conversationId = await invoke<string>('get_or_create_active_conversation');
```

### 2. Send a Message

```typescript
interface CoachResponse {
  message: string;
  suggestions: string[];
}

const response = await invoke<CoachResponse>('send_coach_message', {
  conversationId,  // or null to auto-create
  message: 'I keep getting distracted by social media',
  context: null    // auto-fetches user stats if not provided
});

console.log(response.message);
// "I notice social media is a common trigger for you. Let's create a pre-commitment..."

console.log(response.suggestions);
// ["Block social media", "Start a focus session", "Log this trigger"]
```

### 3. View Conversation History

```typescript
interface ConversationSummary {
  id: string;
  title: string;
  summary?: string;
  message_count: number;
  created_at: string;
  updated_at: string;
}

// Get recent conversations
const conversations = await invoke<ConversationSummary[]>('list_conversations', {
  limit: 20,
  offset: 0,
  includeArchived: false
});

// Display in UI
conversations.forEach(conv => {
  console.log(`${conv.title} (${conv.message_count} messages)`);
});
```

### 4. Load Full Conversation

```typescript
interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  created_at: string;
}

interface ConversationWithMessages {
  conversation: {
    id: string;
    title: string;
    message_count: number;
    // ...other fields
  };
  messages: Message[];
}

const full = await invoke<ConversationWithMessages>('get_conversation', {
  conversationId: 'conversation-uuid-here'
});

// Render messages in chat UI
full.messages.forEach(msg => {
  if (msg.role === 'user') {
    renderUserMessage(msg.content);
  } else {
    renderAssistantMessage(msg.content);
  }
});
```

## Managing Memories

### Save a Memory

```typescript
await invoke('save_memory', {
  key: 'preferred_work_time',
  value: 'early mornings, 6-9am',
  category: 'preference',
  confidence: 0.9,
  sourceConversationId: conversationId
});
```

### Memory Categories

- **preference**: User preferences (session length, break frequency)
- **fact**: Factual info (job role, timezone, schedule)
- **pattern**: Behavioral patterns (distraction triggers, productivity peaks)
- **goal**: User goals (target focus hours, skills to develop)
- **context**: Temporary context (upcoming deadlines, current projects)

### Get All Memories

```typescript
interface Memory {
  id: string;
  key: string;
  value: string;
  category: 'preference' | 'fact' | 'pattern' | 'goal' | 'context';
  confidence: number;
  created_at: string;
  last_updated: string;
}

// Get memories by category
const preferences = await invoke<Memory[]>('get_memories', {
  category: 'preference',
  limit: 50
});

// Get all memories
const allMemories = await invoke<Memory[]>('get_memories', {
  category: null,
  limit: 100
});
```

### Get Relevant Memories

```typescript
// Find memories relevant to current context
const relevant = await invoke<Memory[]>('get_relevant_memories', {
  context: 'planning my morning routine',
  limit: 5
});

// This might return:
// - preferred_work_time: "early mornings, 6-9am"
// - morning_ritual: "coffee + 10min meditation"
// - first_task_preference: "hardest task first"
```

## Conversation Management

### Archive Old Conversations

```typescript
await invoke('archive_conversation', {
  conversationId: 'old-conversation-id'
});

// Archived conversations won't appear in default list
// but can still be retrieved with includeArchived: true
const archived = await invoke<ConversationSummary[]>('list_conversations', {
  limit: 20,
  offset: 0,
  includeArchived: true
});
```

### Delete Conversations

```typescript
// Soft delete (can be recovered from DB if needed)
await invoke('delete_conversation', {
  conversationId: 'conversation-to-delete'
});
```

### Update Conversation Title

```typescript
await invoke('update_conversation_title', {
  conversationId: conversationId,
  title: 'Focus Strategy Planning'
});
```

## Background Maintenance

### Auto-Archive Old Conversations

```typescript
// Run periodically (e.g., daily)
const archivedCount = await invoke<number>('auto_archive_old_conversations');
console.log(`Archived ${archivedCount} old conversations`);
```

### Cleanup Expired Memories

```typescript
// Run periodically to remove expired memories
const cleanedCount = await invoke<number>('cleanup_expired_memories');
console.log(`Cleaned up ${cleanedCount} expired memories`);
```

## Example: Building a Chat UI

```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { useState, useEffect } from 'react';

function CoachChat() {
  const [conversationId, setConversationId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');

  useEffect(() => {
    // Load or create conversation on mount
    loadConversation();
  }, []);

  async function loadConversation() {
    const id = await invoke<string>('get_or_create_active_conversation');
    setConversationId(id);

    // Load existing messages
    const conv = await invoke<ConversationWithMessages>('get_conversation', {
      conversationId: id
    });
    setMessages(conv.messages);
  }

  async function sendMessage() {
    if (!input.trim()) return;

    // Optimistically add user message
    const userMsg: Message = {
      id: 'temp',
      role: 'user',
      content: input,
      created_at: new Date().toISOString()
    };
    setMessages([...messages, userMsg]);
    setInput('');

    // Send to coach
    const response = await invoke<CoachResponse>('send_coach_message', {
      conversationId,
      message: input,
      context: null
    });

    // Add assistant response
    const assistantMsg: Message = {
      id: 'temp2',
      role: 'assistant',
      content: response.message,
      created_at: new Date().toISOString()
    };
    setMessages(msgs => [...msgs, assistantMsg]);
  }

  return (
    <div className="chat-container">
      <div className="messages">
        {messages.map((msg, i) => (
          <div key={i} className={`message ${msg.role}`}>
            {msg.content}
          </div>
        ))}
      </div>
      <input
        value={input}
        onChange={e => setInput(e.target.value)}
        onKeyPress={e => e.key === 'Enter' && sendMessage()}
        placeholder="Ask your coach..."
      />
    </div>
  );
}
```

## Example: Memory Dashboard

```typescript
function MemoryDashboard() {
  const [memories, setMemories] = useState<Memory[]>([]);

  useEffect(() => {
    loadMemories();
  }, []);

  async function loadMemories() {
    const all = await invoke<Memory[]>('get_memories', {
      category: null,
      limit: 100
    });
    setMemories(all);
  }

  function groupByCategory(mems: Memory[]) {
    return mems.reduce((acc, mem) => {
      if (!acc[mem.category]) acc[mem.category] = [];
      acc[mem.category].push(mem);
      return acc;
    }, {} as Record<string, Memory[]>);
  }

  const grouped = groupByCategory(memories);

  return (
    <div className="memory-dashboard">
      <h2>What I Remember</h2>
      {Object.entries(grouped).map(([category, mems]) => (
        <div key={category} className="category-section">
          <h3>{category}</h3>
          <ul>
            {mems.map(mem => (
              <li key={mem.id}>
                <strong>{mem.key}:</strong> {mem.value}
                <span className="confidence">
                  {(mem.confidence * 100).toFixed(0)}% confident
                </span>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
}
```

## Pro Tips

### 1. Token Management

The system automatically estimates token counts. For longer conversations, consider:

```typescript
// Get recent messages only
const recent = await invoke<Message[]>('get_recent_messages', {
  conversationId,
  limit: 20  // Last 20 messages only
});
```

### 2. Context Building

The `send_coach_message` command automatically:
- Loads recent messages
- Finds relevant memories
- Builds an enhanced context prompt
- Saves the conversation

No need to manually manage context!

### 3. Memory Confidence

Lower confidence = less reliable. Filter by confidence:

```typescript
const highConfidence = memories.filter(m => m.confidence >= 0.8);
```

### 4. Conversation Summaries

For long conversations, check if a summary exists:

```typescript
const conv = await invoke<ConversationWithMessages>('get_conversation', {
  conversationId
});

if (conv.conversation.summary) {
  console.log('Summary:', conv.conversation.summary);
}
```

## Common Patterns

### Pattern 1: New Chat Session

```typescript
// User clicks "New Chat"
const newConvId = await invoke<string>('create_conversation', {
  title: null  // Auto-generates from first message
});
setConversationId(newConvId);
setMessages([]);
```

### Pattern 2: Load Previous Chat

```typescript
// User selects from conversation list
const conv = await invoke<ConversationWithMessages>('get_conversation', {
  conversationId: selectedId
});
setConversationId(conv.conversation.id);
setMessages(conv.messages);
```

### Pattern 3: Extract and Save Insights

```typescript
// After coach identifies a pattern
await invoke('save_memory', {
  key: 'distraction_pattern',
  value: 'Loses focus around 3pm, needs break or snack',
  category: 'pattern',
  confidence: 0.85,
  sourceConversationId: conversationId
});
```

### Pattern 4: Context-Aware Greeting

```typescript
// On app open, show personalized greeting
const memories = await invoke<Memory[]>('get_memories', {
  category: 'preference',
  limit: 5
});

const greeting = `Welcome back! ${
  memories.length > 0
    ? `I remember you prefer ${memories[0].value}.`
    : "Let's build your focus routine."
}`;
```

## Troubleshooting

### Conversation not found

```typescript
try {
  const conv = await invoke('get_conversation', { conversationId });
} catch (error) {
  console.error('Conversation not found or deleted');
  // Create new conversation
  const newId = await invoke('get_or_create_active_conversation');
  setConversationId(newId);
}
```

### Too many messages

If performance degrades:

```typescript
// Archive old conversations
await invoke('auto_archive_old_conversations');

// Or load only recent messages
const recent = await invoke('get_recent_messages', {
  conversationId,
  limit: 50
});
```

## Next Steps

1. **Build Chat UI**: Use examples above to create chat interface
2. **Memory Viewer**: Let users see and edit their memories
3. **Conversation List**: Show conversation history with search
4. **Export**: Add export conversations to markdown/JSON
5. **Smart Suggestions**: Use memories to pre-fill suggestions

## API Quick Reference

| Command | Purpose |
|---------|---------|
| `send_coach_message` | Send message with full context |
| `get_or_create_active_conversation` | Get/create current chat |
| `list_conversations` | Get conversation list |
| `get_conversation` | Load full conversation |
| `create_conversation` | Start new conversation |
| `delete_conversation` | Remove conversation |
| `archive_conversation` | Archive old conversation |
| `add_message` | Add message manually |
| `get_recent_messages` | Get recent N messages |
| `save_memory` | Save/update memory |
| `get_memories` | Get memories by category |
| `get_relevant_memories` | Find relevant memories |
| `auto_archive_old_conversations` | Cleanup old conversations |
| `cleanup_expired_memories` | Remove expired memories |
| `update_conversation_title` | Change conversation title |

Happy coding! 🚀
