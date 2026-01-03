// components/chat/ChatHistoryPanel.tsx
// Sidebar panel showing conversation history

import * as React from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { Plus, MessageSquareOff, AlertCircle } from "lucide-react";
import { ConversationListItem } from "./ConversationListItem";
import { useConversations, useDeleteConversation } from "@/hooks/useChatHistory";
import type { Conversation } from "@focusflow/types";

export interface ChatHistoryPanelProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  activeConversationId?: string | null;
  onConversationSelect?: (id: string) => void;
  onNewChat?: () => void;
  className?: string;
}

export function ChatHistoryPanel({
  open,
  onOpenChange,
  activeConversationId = null,
  onConversationSelect,
  onNewChat,
  className,
}: ChatHistoryPanelProps) {
  const {
    data: conversationsData,
    isLoading,
    isError,
    error,
    refetch,
  } = useConversations({
    limit: 50,
    daysBack: 30,
  });

  const deleteConversation = useDeleteConversation();

  const handleConversationClick = React.useCallback(
    (id: string) => {
      onConversationSelect?.(id);
      // Optionally close the panel on mobile after selection
      // onOpenChange(false);
    },
    [onConversationSelect]
  );

  const handleDelete = React.useCallback(
    async (id: string) => {
      try {
        await deleteConversation.mutateAsync(id);
        // If the deleted conversation was active, trigger new chat
        if (id === activeConversationId) {
          onNewChat?.();
        }
      } catch (err) {
        console.error("Failed to delete conversation:", err);
      }
    },
    [deleteConversation, activeConversationId, onNewChat]
  );

  const handleNewChat = React.useCallback(() => {
    onNewChat?.();
    // Optionally close the panel on mobile after creating new chat
    // onOpenChange(false);
  }, [onNewChat]);

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="left" className={cn("w-full sm:max-w-md p-0 flex flex-col", className)}>
        <SheetHeader className="px-4 pt-4 pb-3 border-b">
          <div className="flex items-center justify-between">
            <div>
              <SheetTitle>Chat History</SheetTitle>
              <SheetDescription>Your recent conversations (last 30 days)</SheetDescription>
            </div>
            <Button variant="default" size="sm" onClick={handleNewChat} className="ml-2">
              <Plus className="h-4 w-4 mr-1" />
              New
            </Button>
          </div>
        </SheetHeader>

        {/* Conversation List */}
        <div className="flex-1 overflow-y-auto px-4 py-3 min-h-0">
          {isLoading && <LoadingState />}

          {isError && (
            <ErrorState
              message={(error as Error | undefined)?.message ?? "Failed to load conversations"}
              onRetry={refetch}
            />
          )}

          {!isLoading && !isError && conversationsData?.conversations && (
            <>
              {conversationsData.conversations.length === 0 ? (
                <EmptyState onNewChat={handleNewChat} />
              ) : (
                <div className="space-y-2">
                  {conversationsData.conversations.map((conversation: Conversation) => (
                    <ConversationListItem
                      key={conversation.id}
                      conversation={conversation}
                      isActive={conversation.id === activeConversationId}
                      onClick={handleConversationClick}
                      onDelete={handleDelete}
                    />
                  ))}
                </div>
              )}

              {/* Load More Indicator */}
              {conversationsData.hasMore && (
                <div className="mt-4 text-center">
                  <p className="text-xs text-muted-foreground">
                    Showing {conversationsData.conversations.length} of {conversationsData.total}{" "}
                    conversations
                  </p>
                </div>
              )}
            </>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}

// Loading skeleton state
function LoadingState() {
  return (
    <div className="space-y-3">
      {Array.from({ length: 5 }).map((_, i) => (
        <div key={i} className="space-y-2 px-3 py-2.5">
          <div className="flex items-center gap-2">
            <Skeleton className="h-4 w-4 rounded" />
            <Skeleton className="h-4 w-32" />
          </div>
          <Skeleton className="h-3 w-full ml-6" />
          <Skeleton className="h-3 w-24 ml-6" />
        </div>
      ))}
    </div>
  );
}

// Empty state
interface EmptyStateProps {
  onNewChat?: () => void;
}

function EmptyState({ onNewChat }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
      <MessageSquareOff className="h-12 w-12 text-muted-foreground/50 mb-4" />
      <h3 className="text-sm font-medium mb-2">No conversations yet</h3>
      <p className="text-xs text-muted-foreground mb-4 max-w-xs">
        Start chatting with your AI coach to see your conversation history here.
      </p>
      <Button variant="outline" size="sm" onClick={onNewChat}>
        <Plus className="h-4 w-4 mr-2" />
        Start New Chat
      </Button>
    </div>
  );
}

// Error state
interface ErrorStateProps {
  message: string;
  onRetry?: () => void;
}

function ErrorState({ message, onRetry }: ErrorStateProps) {
  return (
    <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
      <AlertCircle className="h-12 w-12 text-destructive/50 mb-4" />
      <h3 className="text-sm font-medium mb-2">Failed to load</h3>
      <p className="text-xs text-muted-foreground mb-4 max-w-xs">{message}</p>
      {onRetry && (
        <Button variant="outline" size="sm" onClick={onRetry}>
          Try Again
        </Button>
      )}
    </div>
  );
}
