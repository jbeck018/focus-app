// components/chat/chat-thinking.tsx
// Animated thinking/loading indicator for chat responses

import * as React from "react";
import { cn } from "@/lib/utils";
import { Bot, Loader2 } from "lucide-react";

interface ChatThinkingProps {
  /** The message to display while thinking */
  message?: string;
  /** Variant of the thinking indicator */
  variant?: "dots" | "spinner" | "pulse" | "typing";
  /** Whether to show the assistant avatar */
  showAvatar?: boolean;
  /** Additional class names */
  className?: string;
}

function ChatThinking({
  message = "Thinking",
  variant = "dots",
  showAvatar = true,
  className,
}: ChatThinkingProps) {
  return (
    <div
      className={cn(
        "flex gap-3 w-full",
        "animate-in fade-in-0 slide-in-from-bottom-2 duration-300",
        className
      )}
      role="status"
      aria-label="Assistant is thinking"
      aria-live="polite"
    >
      {/* Avatar */}
      {showAvatar && (
        <div className="flex-shrink-0 size-8 rounded-full flex items-center justify-center bg-secondary text-secondary-foreground">
          <Bot className="size-4" />
        </div>
      )}

      {/* Thinking indicator */}
      <div className="flex flex-col gap-1 items-start">
        <span className="text-xs font-medium text-muted-foreground ml-1">
          Coach
        </span>

        <div
          className={cn(
            "px-4 py-3 rounded-2xl rounded-tl-md",
            "bg-secondary text-secondary-foreground"
          )}
        >
          <div className="flex items-center gap-2">
            {variant === "dots" && <ThinkingDots />}
            {variant === "spinner" && <ThinkingSpinner message={message} />}
            {variant === "pulse" && <ThinkingPulse message={message} />}
            {variant === "typing" && <ThinkingTyping message={message} />}
          </div>
        </div>
      </div>
    </div>
  );
}

// Animated dots (default)
function ThinkingDots() {
  return (
    <div className="flex gap-1 py-1" aria-hidden="true">
      <span
        className="size-2 rounded-full bg-current opacity-60 animate-bounce"
        style={{ animationDelay: "0ms", animationDuration: "600ms" }}
      />
      <span
        className="size-2 rounded-full bg-current opacity-60 animate-bounce"
        style={{ animationDelay: "150ms", animationDuration: "600ms" }}
      />
      <span
        className="size-2 rounded-full bg-current opacity-60 animate-bounce"
        style={{ animationDelay: "300ms", animationDuration: "600ms" }}
      />
    </div>
  );
}

// Spinner with message
function ThinkingSpinner({ message }: { message: string }) {
  return (
    <>
      <Loader2 className="size-4 animate-spin text-muted-foreground" />
      <span className="text-sm text-muted-foreground">{message}...</span>
    </>
  );
}

// Pulsing text
function ThinkingPulse({ message }: { message: string }) {
  return (
    <span className="text-sm text-muted-foreground animate-pulse">
      {message}...
    </span>
  );
}

// Typing effect
function ThinkingTyping({ message }: { message: string }) {
  const [displayText, setDisplayText] = React.useState("");
  const [showCursor, setShowCursor] = React.useState(true);

  React.useEffect(() => {
    let index = 0;
    const text = `${message}...`;

    const typeInterval = setInterval(() => {
      if (index <= text.length) {
        setDisplayText(text.slice(0, index));
        index++;
      } else {
        // Reset and start over
        index = 0;
        setDisplayText("");
      }
    }, 100);

    return () => clearInterval(typeInterval);
  }, [message]);

  React.useEffect(() => {
    const cursorInterval = setInterval(() => {
      setShowCursor((prev) => !prev);
    }, 500);

    return () => clearInterval(cursorInterval);
  }, []);

  return (
    <span className="text-sm text-muted-foreground">
      {displayText}
      <span
        className={cn(
          "inline-block w-0.5 h-4 bg-current ml-0.5 align-middle",
          showCursor ? "opacity-100" : "opacity-0"
        )}
      />
    </span>
  );
}

// Minimal thinking indicator (just dots, no avatar)
function ChatThinkingMinimal({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "flex items-center gap-2 text-muted-foreground",
        "animate-in fade-in-0 duration-200",
        className
      )}
      role="status"
      aria-label="Processing"
    >
      <Loader2 className="size-4 animate-spin" />
      <span className="text-sm">Thinking...</span>
    </div>
  );
}

// Skeleton loader for message content
function ChatMessageSkeleton({ className }: { className?: string }) {
  return (
    <div
      className={cn("flex gap-3 w-full animate-pulse", className)}
      role="status"
      aria-label="Loading message"
    >
      {/* Avatar skeleton */}
      <div className="flex-shrink-0 size-8 rounded-full bg-muted" />

      {/* Content skeleton */}
      <div className="flex flex-col gap-2 flex-1 max-w-[70%]">
        <div className="h-3 w-16 rounded bg-muted" />
        <div className="space-y-2 p-4 rounded-2xl rounded-tl-md bg-muted/50">
          <div className="h-4 w-full rounded bg-muted" />
          <div className="h-4 w-4/5 rounded bg-muted" />
          <div className="h-4 w-3/5 rounded bg-muted" />
        </div>
      </div>
    </div>
  );
}

export {
  ChatThinking,
  ChatThinkingMinimal,
  ChatMessageSkeleton,
  ThinkingDots,
};
export type { ChatThinkingProps };
