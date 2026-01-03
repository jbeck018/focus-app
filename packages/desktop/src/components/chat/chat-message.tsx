// components/chat/chat-message.tsx
// Message bubble with support for user/assistant/system roles, timestamps, and actions

import * as React from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { Check, Copy, Bot, User } from "lucide-react";

type MessageRole = "user" | "assistant" | "system";

interface ChatMessageProps {
  /** The message content (supports markdown) */
  content: string;
  /** The role of the message sender */
  role: MessageRole;
  /** ISO timestamp string */
  timestamp?: string;
  /** Whether to show the copy action */
  showCopy?: boolean;
  /** Whether this message is part of a group (consecutive messages from same role) */
  isGrouped?: boolean;
  /** Whether this is the first message in a group */
  isGroupStart?: boolean;
  /** Whether this is the last message in a group */
  isGroupEnd?: boolean;
  /** Additional class names */
  className?: string;
  /** Children for additional content (like reasoning) */
  children?: React.ReactNode;
}

function ChatMessage({
  content,
  role,
  timestamp,
  showCopy = true,
  isGrouped: _isGrouped = false,
  isGroupStart = true,
  isGroupEnd = true,
  className,
  children,
}: ChatMessageProps) {
  // _isGrouped is used for future message grouping features
  void _isGrouped;
  const [copied, setCopied] = React.useState(false);
  const isUser = role === "user";
  const isSystem = role === "system";

  const handleCopy = React.useCallback(async () => {
    try {
      await navigator.clipboard.writeText(content);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy message:", err);
    }
  }, [content]);

  const formattedTime = React.useMemo(() => {
    if (!timestamp) return null;
    try {
      const date = new Date(timestamp);
      return date.toLocaleTimeString("en-US", {
        hour: "numeric",
        minute: "2-digit",
        hour12: true,
      });
    } catch {
      return null;
    }
  }, [timestamp]);

  // Render content with basic markdown support
  const renderedContent = React.useMemo(() => {
    return renderMarkdown(content);
  }, [content]);

  if (isSystem) {
    return (
      <div
        className={cn("flex justify-center py-2", className)}
        role="status"
        aria-label="System message"
      >
        <div className="px-3 py-1.5 rounded-full bg-muted/50 text-muted-foreground text-xs">
          {content}
        </div>
      </div>
    );
  }

  return (
    <div
      className={cn(
        "group flex gap-3 w-full",
        isUser ? "flex-row-reverse" : "flex-row",
        !isGroupEnd && "pb-0.5",
        className
      )}
      role="article"
      aria-label={`${role} message`}
    >
      {/* Avatar */}
      {isGroupStart && (
        <div
          className={cn(
            "flex-shrink-0 size-8 rounded-full flex items-center justify-center",
            isUser ? "bg-primary text-primary-foreground" : "bg-secondary text-secondary-foreground"
          )}
        >
          {isUser ? <User className="size-4" /> : <Bot className="size-4" />}
        </div>
      )}

      {/* Spacer for grouped messages without avatar */}
      {!isGroupStart && <div className="w-8 flex-shrink-0" />}

      {/* Message bubble */}
      <div
        className={cn(
          "flex flex-col gap-1 max-w-[80%] min-w-0",
          isUser ? "items-end" : "items-start"
        )}
      >
        {/* Label for assistant (only on group start) */}
        {!isUser && isGroupStart && (
          <span className="text-xs font-medium text-muted-foreground ml-1">Coach</span>
        )}

        {/* Bubble */}
        <div
          className={cn(
            "relative px-4 py-2.5 text-sm leading-relaxed",
            isUser
              ? "bg-primary text-primary-foreground"
              : "bg-secondary text-secondary-foreground",
            // Rounded corners based on position in group
            isUser
              ? cn(
                  "rounded-2xl",
                  isGroupStart && "rounded-tr-md",
                  !isGroupStart && !isGroupEnd && "rounded-r-md",
                  isGroupEnd && !isGroupStart && "rounded-br-md"
                )
              : cn(
                  "rounded-2xl",
                  isGroupStart && "rounded-tl-md",
                  !isGroupStart && !isGroupEnd && "rounded-l-md",
                  isGroupEnd && !isGroupStart && "rounded-bl-md"
                )
          )}
        >
          {/* Content */}
          <div className="prose prose-sm dark:prose-invert max-w-none break-words">
            {renderedContent}
          </div>

          {/* Copy button (appears on hover) */}
          {showCopy && !isUser && (
            <div
              className={cn(
                "absolute -right-1 top-1 translate-x-full",
                "opacity-0 group-hover:opacity-100 transition-opacity"
              )}
            >
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    className="size-7"
                    onClick={handleCopy}
                    aria-label={copied ? "Copied" : "Copy message"}
                  >
                    {copied ? (
                      <Check className="size-3.5 text-green-500" />
                    ) : (
                      <Copy className="size-3.5" />
                    )}
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="right">{copied ? "Copied!" : "Copy message"}</TooltipContent>
              </Tooltip>
            </div>
          )}
        </div>

        {/* Children (e.g., reasoning component) */}
        {children}

        {/* Timestamp */}
        {formattedTime && isGroupEnd && (
          <span
            className={cn("text-[10px] text-muted-foreground/70 mt-0.5", isUser ? "mr-1" : "ml-1")}
          >
            {formattedTime}
          </span>
        )}
      </div>
    </div>
  );
}

// Simple markdown renderer (no external dependencies)
function renderMarkdown(text: string): React.ReactNode {
  const lines = text.split("\n");
  const elements: React.ReactNode[] = [];
  let inCodeBlock = false;
  let codeBlockContent: string[] = [];
  let codeBlockLang = "";
  let inList = false;
  let listItems: string[] = [];
  let listType: "ul" | "ol" = "ul";

  const flushList = (index: number) => {
    if (listItems.length > 0) {
      const ListTag = listType === "ul" ? "ul" : "ol";
      elements.push(
        <ListTag
          key={`list-${index}`}
          className={cn("my-2 ml-4 space-y-1", listType === "ul" ? "list-disc" : "list-decimal")}
        >
          {listItems.map((item, idx) => (
            <li key={idx} className="text-sm">
              {renderInlineMarkdown(item)}
            </li>
          ))}
        </ListTag>
      );
      listItems = [];
      inList = false;
    }
  };

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Code block handling
    if (line.startsWith("```")) {
      flushList(i);
      if (!inCodeBlock) {
        inCodeBlock = true;
        codeBlockLang = line.slice(3).trim();
        codeBlockContent = [];
      } else {
        elements.push(
          <CodeBlock key={i} language={codeBlockLang}>
            {codeBlockContent.join("\n")}
          </CodeBlock>
        );
        inCodeBlock = false;
        codeBlockLang = "";
      }
      continue;
    }

    if (inCodeBlock) {
      codeBlockContent.push(line);
      continue;
    }

    // Headers
    const headerMatch = line.match(/^(#{1,6})\s+(.+)$/);
    if (headerMatch) {
      flushList(i);
      const level = headerMatch[1].length;
      const HeaderTag = `h${level}` as "h1" | "h2" | "h3" | "h4" | "h5" | "h6";
      const headerClasses =
        {
          1: "text-xl font-bold mt-4 mb-2",
          2: "text-lg font-bold mt-3 mb-2",
          3: "text-base font-semibold mt-2 mb-1",
          4: "text-sm font-semibold mt-2 mb-1",
          5: "text-sm font-medium mt-1 mb-1",
          6: "text-xs font-medium mt-1 mb-1",
        }[level] || "";

      elements.push(
        React.createElement(
          HeaderTag,
          { key: i, className: headerClasses },
          renderInlineMarkdown(headerMatch[2])
        )
      );
      continue;
    }

    // Unordered list
    const ulMatch = line.match(/^[\s]*[-*+]\s+(.+)$/);
    if (ulMatch) {
      if (!inList || listType !== "ul") {
        flushList(i);
        inList = true;
        listType = "ul";
      }
      listItems.push(ulMatch[1]);
      continue;
    }

    // Ordered list
    const olMatch = line.match(/^[\s]*\d+\.\s+(.+)$/);
    if (olMatch) {
      if (!inList || listType !== "ol") {
        flushList(i);
        inList = true;
        listType = "ol";
      }
      listItems.push(olMatch[1]);
      continue;
    }

    // If we had a list and now we don't, flush it
    if (inList && !ulMatch && !olMatch && line.trim() !== "") {
      flushList(i);
    }

    // Regular content
    if (line.trim() === "") {
      flushList(i);
      elements.push(<br key={i} />);
    } else if (!inList) {
      elements.push(
        <p key={i} className="m-0">
          {renderInlineMarkdown(line)}
        </p>
      );
    }
  }

  // Flush any remaining list
  flushList(lines.length);

  // Handle unclosed code block
  if (inCodeBlock && codeBlockContent.length > 0) {
    elements.push(
      <CodeBlock key="unclosed" language={codeBlockLang}>
        {codeBlockContent.join("\n")}
      </CodeBlock>
    );
  }

  return <>{elements}</>;
}

function renderInlineMarkdown(text: string): React.ReactNode {
  const parts: React.ReactNode[] = [];
  let remaining = text;
  let keyIndex = 0;

  // Process inline code first
  while (remaining.length > 0) {
    // Inline code
    const codeMatch = remaining.match(/`([^`]+)`/);
    if (codeMatch?.index !== undefined) {
      if (codeMatch.index > 0) {
        parts.push(
          <span key={keyIndex++}>{processTextFormatting(remaining.slice(0, codeMatch.index))}</span>
        );
      }
      parts.push(
        <code key={keyIndex++} className="px-1.5 py-0.5 rounded bg-muted font-mono text-xs">
          {codeMatch[1]}
        </code>
      );
      remaining = remaining.slice(codeMatch.index + codeMatch[0].length);
      continue;
    }

    // No more special patterns
    parts.push(<span key={keyIndex++}>{processTextFormatting(remaining)}</span>);
    break;
  }

  return <>{parts}</>;
}

function processTextFormatting(text: string): React.ReactNode {
  // Bold
  let result = text.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  // Italic
  result = result.replace(/\*([^*]+)\*/g, "<em>$1</em>");
  // Use dangerouslySetInnerHTML for the formatted text
  // This is safe because we control the input patterns
  return <span dangerouslySetInnerHTML={{ __html: result }} />;
}

// Code block component
interface CodeBlockProps {
  children: string;
  language?: string;
}

function CodeBlock({ children, language }: CodeBlockProps) {
  const [copied, setCopied] = React.useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(children);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy code:", err);
    }
  };

  return (
    <div className="relative group/code my-2 rounded-lg overflow-hidden bg-muted/80 border">
      {language && (
        <div className="flex items-center justify-between px-3 py-1.5 border-b bg-muted/50">
          <span className="text-xs font-mono text-muted-foreground">{language}</span>
          <Button
            variant="ghost"
            size="icon-sm"
            className="size-6 opacity-0 group-hover/code:opacity-100 transition-opacity"
            onClick={handleCopy}
            aria-label={copied ? "Copied" : "Copy code"}
          >
            {copied ? <Check className="size-3 text-green-500" /> : <Copy className="size-3" />}
          </Button>
        </div>
      )}
      <pre className="p-3 overflow-x-auto">
        <code className="text-xs font-mono">{children}</code>
      </pre>
      {!language && (
        <Button
          variant="ghost"
          size="icon-sm"
          className="absolute top-2 right-2 size-6 opacity-0 group-hover/code:opacity-100 transition-opacity"
          onClick={handleCopy}
          aria-label={copied ? "Copied" : "Copy code"}
        >
          {copied ? <Check className="size-3 text-green-500" /> : <Copy className="size-3" />}
        </Button>
      )}
    </div>
  );
}

// Message group wrapper for consecutive messages from same role
interface ChatMessageGroupProps {
  children: React.ReactNode;
  className?: string;
}

function ChatMessageGroup({ children, className }: ChatMessageGroupProps) {
  const childArray = React.Children.toArray(children);

  return (
    <div className={cn("flex flex-col", className)}>
      {React.Children.map(childArray, (child, index) => {
        if (React.isValidElement<ChatMessageProps>(child)) {
          return React.cloneElement(child, {
            isGrouped: childArray.length > 1,
            isGroupStart: index === 0,
            isGroupEnd: index === childArray.length - 1,
          });
        }
        return child;
      })}
    </div>
  );
}

export { ChatMessage, ChatMessageGroup, CodeBlock };
export type { ChatMessageProps, MessageRole };
