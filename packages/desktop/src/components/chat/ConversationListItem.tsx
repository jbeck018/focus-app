// components/chat/ConversationListItem.tsx
// Single conversation item in the history list

import * as React from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { MessageSquare, Trash2, MoreVertical } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { Conversation } from "@focusflow/types";

export interface ConversationListItemProps {
  conversation: Conversation;
  isActive?: boolean;
  onClick?: (id: string) => void;
  onDelete?: (id: string) => void;
  className?: string;
}

export function ConversationListItem({
  conversation,
  isActive = false,
  onClick,
  onDelete,
  className,
}: ConversationListItemProps) {
  const [showDeleteDialog, setShowDeleteDialog] = React.useState(false);
  const [isDeleting, setIsDeleting] = React.useState(false);

  const handleClick = React.useCallback(() => {
    onClick?.(conversation.id);
  }, [onClick, conversation.id]);

  const handleDelete = React.useCallback(async () => {
    setIsDeleting(true);
    try {
      await onDelete?.(conversation.id);
      setShowDeleteDialog(false);
    } finally {
      setIsDeleting(false);
    }
  }, [onDelete, conversation.id]);

  const formattedDate = React.useMemo(() => {
    try {
      const date = new Date(conversation.updatedAt);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffMins = Math.floor(diffMs / 60000);
      const diffHours = Math.floor(diffMs / 3600000);
      const diffDays = Math.floor(diffMs / 86400000);

      if (diffMins < 1) return "Just now";
      if (diffMins < 60) return `${diffMins}m ago`;
      if (diffHours < 24) return `${diffHours}h ago`;
      if (diffDays < 7) return `${diffDays}d ago`;

      return date.toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
      });
    } catch {
      return "";
    }
  }, [conversation.updatedAt]);

  const truncatedLastMessage = React.useMemo(() => {
    if (!conversation.lastMessage) return "No messages yet";
    const maxLength = 60;
    return conversation.lastMessage.length > maxLength
      ? `${conversation.lastMessage.slice(0, maxLength)}...`
      : conversation.lastMessage;
  }, [conversation.lastMessage]);

  return (
    <>
      <div
        className={cn(
          "group relative flex flex-col gap-1 px-3 py-2.5 rounded-lg cursor-pointer transition-colors",
          "hover:bg-accent/50",
          isActive && "bg-accent",
          className
        )}
        onClick={handleClick}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            handleClick();
          }
        }}
        aria-label={`Conversation: ${conversation.title}`}
        aria-current={isActive ? "true" : "false"}
      >
        {/* Header: Title and Actions */}
        <div className="flex items-start justify-between gap-2">
          <div className="flex items-center gap-2 min-w-0 flex-1">
            <MessageSquare className="h-4 w-4 flex-shrink-0 text-muted-foreground" />
            <h4 className="text-sm font-medium truncate">{conversation.title}</h4>
          </div>

          {/* Actions Menu */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild onClick={(e) => e.stopPropagation()}>
              <Button
                variant="ghost"
                size="icon-sm"
                className={cn(
                  "h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity",
                  isActive && "opacity-100"
                )}
                aria-label="Conversation actions"
              >
                <MoreVertical className="h-3.5 w-3.5" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" onClick={(e) => e.stopPropagation()}>
              <DropdownMenuItem
                className="text-destructive focus:text-destructive"
                onClick={(e) => {
                  e.stopPropagation();
                  setShowDeleteDialog(true);
                }}
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        {/* Last Message Preview */}
        <p className="text-xs text-muted-foreground line-clamp-2 pl-6">
          {truncatedLastMessage}
        </p>

        {/* Footer: Date and Message Count */}
        <div className="flex items-center justify-between text-xs text-muted-foreground/70 pl-6">
          <span>{formattedDate}</span>
          <span>{conversation.messageCount} messages</span>
        </div>
      </div>

      {/* Delete Confirmation Dialog */}
      <Dialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <DialogContent onClick={(e) => e.stopPropagation()}>
          <DialogHeader>
            <DialogTitle>Delete Conversation</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete "{conversation.title}"? This action cannot be
              undone and all messages in this conversation will be permanently deleted.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowDeleteDialog(false)} disabled={isDeleting}>
              Cancel
            </Button>
            <Button
              onClick={handleDelete}
              disabled={isDeleting}
              variant="destructive"
            >
              {isDeleting ? "Deleting..." : "Delete"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
