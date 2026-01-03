# Chat History UI Preview

This document provides a visual preview of the chat history UI components.

## Component Layout

```
┌─────────────────────────────────────────────────────────────┐
│  AI Coach Header                                            │
│  ┌──────┐                                                   │
│  │ ≡≡≡≡ │  AI Focus Coach                    [Model] [⚙️]  │
│  └──────┘                                                    │
│  History Toggle                                             │
│  (shows badge with count)                                   │
└─────────────────────────────────────────────────────────────┘
```

## History Panel (Sheet/Drawer)

When user clicks the history toggle, a panel slides in from the left:

```
┌────────────────────────────────────┐
│  Chat History              [+ New] │
│  Your recent conversations         │
├────────────────────────────────────┤
│                                    │
│  ┌──────────────────────────────┐ │
│  │ 💬 Focus session planning   │ │
│  │ Let's create a plan for...  │ │
│  │ 2h ago • 12 messages         │ │
│  └──────────────────────────────┘ │
│                                    │
│  ┌──────────────────────────────┐ │
│  │ 💬 Productivity tips         │ │
│  │ Here are some tips to...    │ │
│  │ 1d ago • 8 messages   [⋮]   │ │
│  └──────────────────────────────┘ │
│                                    │
│  ┌──────────────────────────────┐ │
│  │ 💬 Break scheduling          │ │
│  │ When planning breaks...      │ │
│  │ 3d ago • 15 messages         │ │
│  └──────────────────────────────┘ │
│                                    │
│  Showing 20 of 45 conversations   │
└────────────────────────────────────┘
```

## Conversation List Item States

### Normal State
```
┌──────────────────────────────────┐
│ 💬 Focus session planning       │
│ Let's create a plan for today.. │
│ 2h ago • 12 messages             │
└──────────────────────────────────┘
```

### Active/Selected State
```
┌──────────────────────────────────┐
│ 💬 Focus session planning    [⋮]│  ← Highlighted
│ Let's create a plan for today.. │
│ 2h ago • 12 messages             │
└──────────────────────────────────┘
```

### Hover State
```
┌──────────────────────────────────┐
│ 💬 Productivity tips         [⋮]│  ← Actions menu appears
│ Here are some tips to improve..│
│ 1d ago • 8 messages              │
└──────────────────────────────────┘
```

## Delete Confirmation Dialog

When user clicks delete from the actions menu:

```
┌─────────────────────────────────────┐
│  Delete Conversation          [×]  │
├─────────────────────────────────────┤
│                                     │
│  Are you sure you want to delete   │
│  "Focus session planning"? This    │
│  action cannot be undone and all   │
│  messages will be permanently      │
│  deleted.                           │
│                                     │
│               [Cancel]  [Delete]    │
└─────────────────────────────────────┘
```

## Viewing History Mode

When user selects a historical conversation:

```
┌─────────────────────────────────────────────────────────────┐
│  Chat Messages Area                                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ [Coach] Hi! How can I help you today?               │   │
│  │                                                      │   │
│  │                   [User] I need help planning... │   │   │
│  │                                                      │   │
│  │ [Coach] Let me help you create a plan...            │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  ⚫ Viewing conversation from Jan 15, 2026                  │
│                      [Continue Conversation]  [New Chat]    │
├─────────────────────────────────────────────────────────────┤
│  Ask me anything about focus... (disabled)                  │
└─────────────────────────────────────────────────────────────┘
```

## Empty State

When user has no conversations yet:

```
┌────────────────────────────────────┐
│  Chat History              [+ New] │
│  Your recent conversations         │
├────────────────────────────────────┤
│                                    │
│            💬                      │
│                                    │
│      No conversations yet          │
│                                    │
│   Start chatting with your AI      │
│   coach to see your conversation   │
│   history here.                    │
│                                    │
│      [Start New Chat]              │
│                                    │
└────────────────────────────────────┘
```

## Loading State

While fetching conversations:

```
┌────────────────────────────────────┐
│  Chat History              [+ New] │
│  Your recent conversations         │
├────────────────────────────────────┤
│                                    │
│  ┌──────────────────────────────┐ │
│  │ ▓▓▓ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓        │ │  ← Skeleton
│  │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓     │ │     loaders
│  │ ▓▓▓▓▓▓▓                      │ │
│  └──────────────────────────────┘ │
│                                    │
│  ┌──────────────────────────────┐ │
│  │ ▓▓▓ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓        │ │
│  │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓     │ │
│  │ ▓▓▓▓▓▓▓                      │ │
│  └──────────────────────────────┘ │
│                                    │
└────────────────────────────────────┘
```

## Error State

When conversation loading fails:

```
┌────────────────────────────────────┐
│  Chat History              [+ New] │
│  Your recent conversations         │
├────────────────────────────────────┤
│                                    │
│            ⚠️                      │
│                                    │
│        Failed to load              │
│                                    │
│   Failed to load conversations.    │
│   Please try again.                │
│                                    │
│         [Try Again]                │
│                                    │
└────────────────────────────────────┘
```

## Chat History Toggle States

### Normal State
```
┌──────┐
│  ⟲   │  ← History icon
└──────┘
```

### With Count Badge
```
┌──────┐
│  ⟲  ⑮│  ← Shows count (e.g., 15 conversations)
└──────┘
```

### Active State (panel open)
```
┌──────┐
│  ⟲   │  ← Highlighted/active
└──────┘
```

## Interactive Elements

### History Toggle
- Click: Opens history panel
- Hover: Shows tooltip "Chat History (15 conversations)"
- Badge: Shows total conversation count

### Conversation Item
- Click: Loads conversation in view-only mode
- Hover: Shows actions menu (⋮)
- Actions: Delete option

### New Chat Button
- Click: Starts fresh conversation
- Clears current messages
- Shows welcome message

### Continue Conversation
- Click: Enables input for historical conversation
- Allows sending new messages
- Updates conversation

## Responsive Behavior

### Desktop (> 768px)
- Panel slides from left
- Max width: 384px (sm:max-w-md)
- Overlay background

### Mobile (< 768px)
- Panel takes 75% of screen width
- Slides from left
- Full-height
- Tap outside to close

## Accessibility Features

- **Keyboard Navigation:** Tab through items, Enter to select
- **ARIA Labels:** Proper labels on all buttons and interactive elements
- **Screen Reader:** Announces state changes
- **Focus Management:** Trapped in panel when open
- **Color Contrast:** WCAG AA compliant
- **Loading States:** Announced to screen readers

## Design System

### Colors
- Primary: Default theme primary color
- Secondary: For assistant messages and backgrounds
- Muted: For timestamps and less important text
- Destructive: For delete actions

### Typography
- Title: `text-sm font-medium`
- Preview: `text-xs text-muted-foreground`
- Timestamp: `text-[10px] text-muted-foreground/70`

### Spacing
- Item padding: `px-3 py-2.5`
- Gap between items: `space-y-2`
- Panel padding: `px-4 py-3`

### Animations
- Panel slide: 300ms ease-in-out
- Hover transitions: opacity changes
- Badge pulse: Subtle pulse animation
