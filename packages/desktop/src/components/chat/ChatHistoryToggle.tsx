// components/chat/ChatHistoryToggle.tsx
// Button to toggle chat history panel

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { History } from "lucide-react";
import { Badge } from "@/components/ui/badge";

export interface ChatHistoryToggleProps {
  onClick?: () => void;
  conversationCount?: number;
  isActive?: boolean;
  className?: string;
}

export function ChatHistoryToggle({
  onClick,
  conversationCount,
  isActive = false,
  className,
}: ChatHistoryToggleProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant={isActive ? "secondary" : "ghost"}
          size="icon"
          onClick={onClick}
          className={cn("h-8 w-8 relative", className)}
          aria-label="Toggle chat history"
          aria-pressed={isActive}
        >
          <History className="h-4 w-4" />
          {conversationCount !== undefined && conversationCount > 0 && (
            <Badge
              variant="secondary"
              className="absolute -top-1 -right-1 h-4 min-w-4 px-1 flex items-center justify-center text-[10px] font-medium"
            >
              {conversationCount > 99 ? "99+" : conversationCount}
            </Badge>
          )}
        </Button>
      </TooltipTrigger>
      <TooltipContent side="bottom">
        <p>Chat History</p>
        {conversationCount !== undefined && conversationCount > 0 && (
          <p className="text-xs text-muted-foreground">
            {conversationCount} conversation{conversationCount !== 1 ? "s" : ""}
          </p>
        )}
      </TooltipContent>
    </Tooltip>
  );
}
