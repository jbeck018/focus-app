// components/chat/chat-input.tsx
// Modern chat input with send button, auto-resize, and keyboard shortcuts

import * as React from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Send, StopCircle, Loader2 } from "lucide-react";

interface ChatInputProps {
  /** Current input value */
  value: string;
  /** Called when input value changes */
  onChange: (value: string) => void;
  /** Called when message should be sent */
  onSend: () => void;
  /** Called when generation should be stopped */
  onStop?: () => void;
  /** Placeholder text */
  placeholder?: string;
  /** Whether the input is disabled */
  disabled?: boolean;
  /** Whether a message is currently being generated */
  isLoading?: boolean;
  /** Whether to allow multiline input (Shift+Enter for new line) */
  multiline?: boolean;
  /** Maximum number of rows for auto-resize */
  maxRows?: number;
  /** Additional class names */
  className?: string;
  /** Ref to the textarea element */
  inputRef?: React.RefObject<HTMLTextAreaElement | null>;
}

function ChatInput({
  value,
  onChange,
  onSend,
  onStop,
  placeholder = "Type a message...",
  disabled = false,
  isLoading = false,
  multiline = true,
  maxRows = 5,
  className,
  inputRef: externalRef,
}: ChatInputProps) {
  const internalRef = React.useRef<HTMLTextAreaElement>(null);
  const inputRef = externalRef || internalRef;

  // Auto-resize textarea
  React.useEffect(() => {
    const textarea = inputRef.current;
    if (!textarea) return;

    // Reset height to calculate new height
    textarea.style.height = "auto";

    // Calculate new height
    const lineHeight = parseInt(getComputedStyle(textarea).lineHeight);
    const maxHeight = lineHeight * maxRows;
    const newHeight = Math.min(textarea.scrollHeight, maxHeight);

    textarea.style.height = `${newHeight}px`;
  }, [value, maxRows, inputRef]);

  const handleKeyDown = React.useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      // Send on Enter (without Shift for multiline mode)
      if (e.key === "Enter") {
        if (multiline && e.shiftKey) {
          // Allow new line
          return;
        }
        if (!e.shiftKey) {
          e.preventDefault();
          if (!disabled && !isLoading && value.trim()) {
            onSend();
          }
        }
      }

      // Stop generation on Escape
      if (e.key === "Escape" && isLoading && onStop) {
        e.preventDefault();
        onStop();
      }
    },
    [multiline, disabled, isLoading, value, onSend, onStop]
  );

  const handleSend = React.useCallback(() => {
    if (!disabled && !isLoading && value.trim()) {
      onSend();
    }
  }, [disabled, isLoading, value, onSend]);

  const canSend = !disabled && !isLoading && value.trim().length > 0;

  return (
    <div
      className={cn(
        "relative flex items-end gap-2 p-2",
        "bg-background border-t",
        className
      )}
    >
      <div className="relative flex-1">
        <Textarea
          ref={inputRef}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          disabled={disabled}
          className={cn(
            "min-h-[44px] max-h-[200px] py-3 pr-12 resize-none",
            "rounded-xl border-muted-foreground/20",
            "focus-visible:ring-1 focus-visible:ring-ring",
            "placeholder:text-muted-foreground/60"
          )}
          rows={1}
          aria-label="Message input"
        />

        {/* Keyboard hint */}
        <div
          className={cn(
            "absolute right-3 bottom-2.5",
            "text-[10px] text-muted-foreground/50",
            "pointer-events-none select-none",
            "transition-opacity duration-200",
            value.length > 0 ? "opacity-0" : "opacity-100"
          )}
        >
          {multiline && (
            <kbd className="font-sans">
              Shift + Enter for new line
            </kbd>
          )}
        </div>
      </div>

      {/* Action button */}
      <div className="flex-shrink-0 pb-0.5">
        {isLoading && onStop ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="outline"
                size="icon"
                onClick={onStop}
                className="size-10 rounded-xl"
                aria-label="Stop generating"
              >
                <StopCircle className="size-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Stop generating (Esc)</TooltipContent>
          </Tooltip>
        ) : (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="icon"
                onClick={handleSend}
                disabled={!canSend}
                className={cn(
                  "size-10 rounded-xl transition-all duration-200",
                  canSend
                    ? "bg-primary hover:bg-primary/90"
                    : "bg-muted text-muted-foreground"
                )}
                aria-label="Send message"
              >
                {isLoading ? (
                  <Loader2 className="size-4 animate-spin" />
                ) : (
                  <Send className="size-4" />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              {canSend ? "Send message (Enter)" : "Type a message to send"}
            </TooltipContent>
          </Tooltip>
        )}
      </div>
    </div>
  );
}

// Simplified input variant (single line, minimal)
interface ChatInputSimpleProps {
  value: string;
  onChange: (value: string) => void;
  onSend: () => void;
  placeholder?: string;
  disabled?: boolean;
  isLoading?: boolean;
  className?: string;
}

function ChatInputSimple({
  value,
  onChange,
  onSend,
  placeholder = "Type a message...",
  disabled = false,
  isLoading = false,
  className,
}: ChatInputSimpleProps) {
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (!disabled && !isLoading && value.trim()) {
        onSend();
      }
    }
  };

  const canSend = !disabled && !isLoading && value.trim().length > 0;

  return (
    <div className={cn("flex gap-2", className)}>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled || isLoading}
        className={cn(
          "flex-1 h-10 px-4 rounded-lg",
          "bg-secondary/50 border border-transparent",
          "text-sm placeholder:text-muted-foreground/60",
          "focus:outline-none focus:border-ring focus:ring-1 focus:ring-ring",
          "disabled:opacity-50 disabled:cursor-not-allowed",
          "transition-colors duration-200"
        )}
        aria-label="Message input"
      />
      <Button
        size="icon"
        onClick={onSend}
        disabled={!canSend}
        className="size-10 rounded-lg"
        aria-label="Send message"
      >
        {isLoading ? (
          <Loader2 className="size-4 animate-spin" />
        ) : (
          <Send className="size-4" />
        )}
      </Button>
    </div>
  );
}

export { ChatInput, ChatInputSimple };
export type { ChatInputProps, ChatInputSimpleProps };
