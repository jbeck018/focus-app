// components/chat/chat-suggestions.tsx
// Quick action chips/suggestions for chat prompts

import * as React from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Sparkles, Zap, Target, Brain, TrendingUp, Clock } from "lucide-react";

interface Suggestion {
  /** Unique identifier */
  id: string;
  /** Display text */
  text: string;
  /** Optional icon name */
  icon?: "sparkles" | "zap" | "target" | "brain" | "trending" | "clock";
  /** Optional category for grouping */
  category?: string;
}

interface ChatSuggestionsProps {
  /** List of suggestions to display */
  suggestions: Suggestion[] | string[];
  /** Called when a suggestion is clicked */
  onSelect: (suggestion: string) => void;
  /** Whether suggestions are disabled */
  disabled?: boolean;
  /** Layout variant */
  variant?: "inline" | "grid" | "compact";
  /** Maximum number of suggestions to show (0 for all) */
  maxVisible?: number;
  /** Additional class names */
  className?: string;
  /** Title above suggestions */
  title?: string;
}

const iconMap = {
  sparkles: Sparkles,
  zap: Zap,
  target: Target,
  brain: Brain,
  trending: TrendingUp,
  clock: Clock,
};

function ChatSuggestions({
  suggestions,
  onSelect,
  disabled = false,
  variant = "inline",
  maxVisible = 0,
  className,
  title,
}: ChatSuggestionsProps) {
  // Normalize suggestions to objects
  const normalizedSuggestions: Suggestion[] = React.useMemo(() => {
    return suggestions.map((s, i) => {
      if (typeof s === "string") {
        return { id: `suggestion-${i}`, text: s };
      }
      return s;
    });
  }, [suggestions]);

  const visibleSuggestions = React.useMemo(() => {
    if (maxVisible === 0 || maxVisible >= normalizedSuggestions.length) {
      return normalizedSuggestions;
    }
    return normalizedSuggestions.slice(0, maxVisible);
  }, [normalizedSuggestions, maxVisible]);

  if (visibleSuggestions.length === 0) {
    return null;
  }

  return (
    <div
      className={cn("animate-in fade-in-0 slide-in-from-bottom-2 duration-300", className)}
      role="group"
      aria-label={title || "Suggested prompts"}
    >
      {title && <p className="text-xs font-medium text-muted-foreground mb-2">{title}</p>}

      <div
        className={cn(
          variant === "inline" && "flex flex-wrap gap-2",
          variant === "grid" && "grid grid-cols-2 gap-2",
          variant === "compact" && "flex flex-wrap gap-1.5"
        )}
      >
        {visibleSuggestions.map((suggestion) => (
          <SuggestionChip
            key={suggestion.id}
            suggestion={suggestion}
            onClick={() => onSelect(suggestion.text)}
            disabled={disabled}
            variant={variant}
          />
        ))}
      </div>
    </div>
  );
}

interface SuggestionChipProps {
  suggestion: Suggestion;
  onClick: () => void;
  disabled?: boolean;
  variant?: "inline" | "grid" | "compact";
}

function SuggestionChip({
  suggestion,
  onClick,
  disabled = false,
  variant = "inline",
}: SuggestionChipProps) {
  const Icon = suggestion.icon ? iconMap[suggestion.icon] : null;

  return (
    <Button
      variant="outline"
      size={variant === "compact" ? "sm" : "default"}
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "group/chip h-auto whitespace-normal text-left justify-start",
        "border-muted-foreground/20 bg-background/50",
        "hover:bg-secondary hover:border-muted-foreground/30",
        "transition-all duration-200",
        variant === "inline" && "px-3 py-2 text-xs",
        variant === "grid" && "px-3 py-2.5 text-sm flex-col items-start gap-1.5",
        variant === "compact" && "px-2.5 py-1.5 text-xs"
      )}
    >
      {Icon && (
        <Icon
          className={cn(
            "flex-shrink-0 text-muted-foreground",
            "group-hover/chip:text-foreground transition-colors",
            variant === "grid" ? "size-4" : "size-3.5 mr-1.5"
          )}
        />
      )}
      <span className="line-clamp-2">{suggestion.text}</span>
    </Button>
  );
}

// Preset suggestion sets
const FOCUS_SUGGESTIONS: Suggestion[] = [
  {
    id: "plan-session",
    text: "Help me plan a focused work session",
    icon: "target",
  },
  {
    id: "distraction-tips",
    text: "Give me tips to avoid distractions",
    icon: "brain",
  },
  {
    id: "productivity",
    text: "How can I improve my productivity?",
    icon: "trending",
  },
  {
    id: "break-activity",
    text: "Suggest a refreshing break activity",
    icon: "zap",
  },
];

const QUICK_ACTIONS: Suggestion[] = [
  { id: "start-focus", text: "Start a 25-min focus session", icon: "clock" },
  { id: "review-patterns", text: "Review my focus patterns", icon: "trending" },
  { id: "set-goal", text: "Set a daily focus goal", icon: "target" },
];

// Floating suggestions that appear above input
interface FloatingSuggestionsProps {
  suggestions: Suggestion[] | string[];
  onSelect: (suggestion: string) => void;
  visible: boolean;
  className?: string;
}

function FloatingSuggestions({
  suggestions,
  onSelect,
  visible,
  className,
}: FloatingSuggestionsProps) {
  if (!visible || suggestions.length === 0) {
    return null;
  }

  return (
    <div
      className={cn(
        "absolute bottom-full left-0 right-0 mb-2 p-3",
        "bg-background/95 backdrop-blur-sm border rounded-xl shadow-lg",
        "animate-in fade-in-0 slide-in-from-bottom-2 duration-200",
        className
      )}
    >
      <ChatSuggestions
        suggestions={suggestions}
        onSelect={onSelect}
        variant="inline"
        title="Try asking..."
      />
    </div>
  );
}

export { ChatSuggestions, SuggestionChip, FloatingSuggestions, FOCUS_SUGGESTIONS, QUICK_ACTIONS };
export type { ChatSuggestionsProps, Suggestion };
