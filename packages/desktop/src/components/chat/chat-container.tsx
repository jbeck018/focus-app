// components/chat/chat-container.tsx
// Main chat wrapper with intelligent scroll management

import * as React from "react";
import { cn } from "@/lib/utils";

interface ChatContainerProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Whether to auto-scroll when new content is added */
  autoScroll?: boolean;
  /** Distance from bottom to trigger auto-scroll lock (in pixels) */
  scrollThreshold?: number;
  children: React.ReactNode;
}

interface ChatContainerRef {
  scrollToBottom: (behavior?: ScrollBehavior) => void;
  isScrolledToBottom: () => boolean;
}

const ChatContainer = React.forwardRef<ChatContainerRef, ChatContainerProps>(
  ({ className, children, autoScroll = true, scrollThreshold = 100, ...props }, ref) => {
    const containerRef = React.useRef<HTMLDivElement>(null);
    const [isUserScrolling, setIsUserScrolling] = React.useState(false);
    const lastScrollTop = React.useRef(0);
    const childCountRef = React.useRef(0);

    const isScrolledToBottom = React.useCallback(() => {
      const container = containerRef.current;
      if (!container) return true;
      const { scrollTop, scrollHeight, clientHeight } = container;
      return scrollHeight - scrollTop - clientHeight < scrollThreshold;
    }, [scrollThreshold]);

    const scrollToBottom = React.useCallback((behavior?: ScrollBehavior) => {
      const container = containerRef.current;
      if (!container) return;
      container.scrollTo({
        top: container.scrollHeight,
        behavior: behavior ?? "smooth",
      });
    }, []);

    // Expose methods via ref
    React.useImperativeHandle(
      ref,
      () => ({
        scrollToBottom,
        isScrolledToBottom,
      }),
      [scrollToBottom, isScrolledToBottom]
    );

    // Handle scroll events to detect user scrolling up
    const handleScroll = React.useCallback(() => {
      const container = containerRef.current;
      if (!container) return;

      const { scrollTop } = container;
      const scrollingUp = scrollTop < lastScrollTop.current;
      lastScrollTop.current = scrollTop;

      if (scrollingUp && !isScrolledToBottom()) {
        setIsUserScrolling(true);
      } else if (isScrolledToBottom()) {
        setIsUserScrolling(false);
      }
    }, [isScrolledToBottom]);

    // Auto-scroll when children change (new messages)
    React.useEffect(() => {
      const childCount = React.Children.count(children);

      if (autoScroll && !isUserScrolling && childCount > childCountRef.current) {
        // Use requestAnimationFrame for smoother scrolling
        requestAnimationFrame(() => {
          scrollToBottom();
        });
      }

      childCountRef.current = childCount;
    }, [children, autoScroll, isUserScrolling, scrollToBottom]);

    // Scroll to bottom on mount
    React.useEffect(() => {
      scrollToBottom("instant");
    }, [scrollToBottom]);

    return (
      <div
        ref={containerRef}
        className={cn(
          "flex flex-col overflow-y-auto overflow-x-hidden",
          "scrollbar-thin scrollbar-thumb-muted-foreground/20 scrollbar-track-transparent",
          "hover:scrollbar-thumb-muted-foreground/30",
          className
        )}
        onScroll={handleScroll}
        role="log"
        aria-live="polite"
        aria-label="Chat messages"
        {...props}
      >
        <div className="flex-1" />
        <div className="flex flex-col gap-1 p-4">{children}</div>
      </div>
    );
  }
);

ChatContainer.displayName = "ChatContainer";

// Scroll to bottom button component
interface ScrollToBottomButtonProps {
  onClick: () => void;
  visible: boolean;
  className?: string;
}

function ScrollToBottomButton({ onClick, visible, className }: ScrollToBottomButtonProps) {
  if (!visible) return null;

  return (
    <button
      onClick={onClick}
      className={cn(
        "absolute bottom-20 left-1/2 -translate-x-1/2",
        "flex items-center gap-1.5 px-3 py-1.5",
        "bg-background/95 backdrop-blur-sm border rounded-full shadow-lg",
        "text-xs font-medium text-muted-foreground",
        "hover:text-foreground hover:bg-background",
        "transition-all duration-200",
        "animate-in fade-in-0 slide-in-from-bottom-2",
        className
      )}
      aria-label="Scroll to latest messages"
    >
      <svg className="size-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
      </svg>
      New messages
    </button>
  );
}

export { ChatContainer, ScrollToBottomButton };
export type { ChatContainerProps, ChatContainerRef };
