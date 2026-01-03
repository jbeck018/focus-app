// components/chat/chat-reasoning.tsx
// Expandable reasoning display for showing LLM thought process

import * as React from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { ChevronDown, ChevronRight, Brain, Sparkles } from "lucide-react";

interface ChatReasoningProps {
  /** The reasoning/thinking content */
  content: string;
  /** Whether the reasoning is initially expanded */
  defaultExpanded?: boolean;
  /** Title for the reasoning section */
  title?: string;
  /** Variant style */
  variant?: "default" | "minimal" | "bordered";
  /** Additional class names */
  className?: string;
}

function ChatReasoning({
  content,
  defaultExpanded = false,
  title = "Reasoning",
  variant = "default",
  className,
}: ChatReasoningProps) {
  const [isExpanded, setIsExpanded] = React.useState(defaultExpanded);
  const contentRef = React.useRef<HTMLDivElement>(null);
  const [contentHeight, setContentHeight] = React.useState(0);

  // Measure content height for smooth animation
  React.useEffect(() => {
    if (contentRef.current) {
      setContentHeight(contentRef.current.scrollHeight);
    }
  }, [content]);

  const toggleExpanded = React.useCallback(() => {
    setIsExpanded((prev) => !prev);
  }, []);

  return (
    <div
      className={cn(
        "w-full mt-2",
        variant === "bordered" && "border rounded-lg overflow-hidden",
        className
      )}
    >
      {/* Header/Toggle */}
      <Button
        variant="ghost"
        size="sm"
        onClick={toggleExpanded}
        className={cn(
          "w-full justify-start gap-2 h-auto py-2 px-3",
          "text-muted-foreground hover:text-foreground",
          "hover:bg-muted/50",
          variant === "bordered" && "rounded-none border-b",
          variant === "minimal" && "px-0"
        )}
        aria-expanded={isExpanded}
        aria-controls="reasoning-content"
      >
        {isExpanded ? (
          <ChevronDown className="size-4 flex-shrink-0" />
        ) : (
          <ChevronRight className="size-4 flex-shrink-0" />
        )}

        {variant !== "minimal" && <Brain className="size-4 flex-shrink-0 text-purple-500" />}

        <span className="text-xs font-medium">{title}</span>

        {!isExpanded && (
          <span className="text-xs text-muted-foreground/60 truncate flex-1 text-left ml-1">
            {content.slice(0, 50)}...
          </span>
        )}
      </Button>

      {/* Content */}
      <div
        id="reasoning-content"
        className={cn(
          "overflow-hidden transition-all duration-300 ease-in-out",
          isExpanded ? "opacity-100" : "opacity-0"
        )}
        style={{
          maxHeight: isExpanded ? contentHeight : 0,
        }}
      >
        <div
          ref={contentRef}
          className={cn(
            "text-sm text-muted-foreground leading-relaxed",
            "whitespace-pre-wrap",
            variant === "default" && "px-3 py-2 bg-muted/30 rounded-lg mx-1 mb-1",
            variant === "bordered" && "p-3 bg-muted/20",
            variant === "minimal" && "py-2"
          )}
        >
          {content}
        </div>
      </div>
    </div>
  );
}

// Inline reasoning that shows as a collapsible section within a message
interface InlineReasoningProps {
  content: string;
  className?: string;
}

function InlineReasoning({ content, className }: InlineReasoningProps) {
  const [isExpanded, setIsExpanded] = React.useState(false);

  return (
    <div className={cn("text-xs", className)}>
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className={cn(
          "inline-flex items-center gap-1 px-2 py-1 rounded-md",
          "text-muted-foreground hover:text-foreground",
          "hover:bg-muted/50 transition-colors",
          "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        )}
        aria-expanded={isExpanded}
      >
        <Sparkles className="size-3" />
        <span>{isExpanded ? "Hide reasoning" : "Show reasoning"}</span>
        {isExpanded ? <ChevronDown className="size-3" /> : <ChevronRight className="size-3" />}
      </button>

      {isExpanded && (
        <div
          className={cn(
            "mt-2 p-2 rounded-md bg-muted/40 border border-muted",
            "text-muted-foreground whitespace-pre-wrap",
            "animate-in fade-in-0 slide-in-from-top-1 duration-200"
          )}
        >
          {content}
        </div>
      )}
    </div>
  );
}

// Streaming reasoning that shows content as it comes in
interface StreamingReasoningProps {
  content: string;
  isStreaming?: boolean;
  className?: string;
}

function StreamingReasoning({ content, isStreaming = false, className }: StreamingReasoningProps) {
  const [isExpanded, setIsExpanded] = React.useState(true);

  return (
    <div className={cn("w-full rounded-lg border bg-muted/20 overflow-hidden", className)}>
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className={cn(
          "w-full flex items-center gap-2 px-3 py-2",
          "text-left text-xs font-medium text-muted-foreground",
          "hover:bg-muted/30 transition-colors",
          "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
        )}
        aria-expanded={isExpanded}
      >
        {isExpanded ? <ChevronDown className="size-4" /> : <ChevronRight className="size-4" />}
        <Brain className="size-4 text-purple-500" />
        <span>Thinking</span>
        {isStreaming && (
          <span className="ml-auto flex items-center gap-1">
            <span className="size-1.5 rounded-full bg-purple-500 animate-pulse" />
          </span>
        )}
      </button>

      {isExpanded && (
        <div
          className={cn(
            "px-3 pb-3 text-sm text-muted-foreground",
            "whitespace-pre-wrap leading-relaxed",
            "max-h-60 overflow-y-auto",
            "scrollbar-thin scrollbar-thumb-muted-foreground/20"
          )}
        >
          {content}
          {isStreaming && (
            <span className="inline-block w-2 h-4 bg-purple-500/50 animate-pulse ml-0.5" />
          )}
        </div>
      )}
    </div>
  );
}

// Compact reasoning badge that expands on click
interface ReasoningBadgeProps {
  content: string;
  className?: string;
}

function ReasoningBadge({ content, className }: ReasoningBadgeProps) {
  const [isOpen, setIsOpen] = React.useState(false);

  return (
    <div className={cn("relative inline-block", className)}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          "inline-flex items-center gap-1 px-2 py-0.5 rounded-full",
          "text-[10px] font-medium",
          "bg-purple-500/10 text-purple-600 dark:text-purple-400",
          "hover:bg-purple-500/20 transition-colors",
          "focus:outline-none focus-visible:ring-2 focus-visible:ring-purple-500"
        )}
      >
        <Brain className="size-3" />
        Reasoning
      </button>

      {isOpen && (
        <>
          {/* Backdrop */}
          <div className="fixed inset-0 z-40" onClick={() => setIsOpen(false)} />

          {/* Popover */}
          <div
            className={cn(
              "absolute top-full left-0 mt-1 z-50",
              "w-72 max-h-48 overflow-y-auto",
              "p-3 rounded-lg border bg-background shadow-lg",
              "text-xs text-muted-foreground whitespace-pre-wrap",
              "animate-in fade-in-0 zoom-in-95 duration-200"
            )}
          >
            {content}
          </div>
        </>
      )}
    </div>
  );
}

export { ChatReasoning, InlineReasoning, StreamingReasoning, ReasoningBadge };
export type { ChatReasoningProps };
