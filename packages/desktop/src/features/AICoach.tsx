// features/AICoach.tsx - AI Coach chat interface with modern chat components

import { useState, useRef, useEffect, useCallback } from "react";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useCoachChat } from "@/hooks/useCoach";
import { useLlmStatusManager } from "@/hooks/useLlmStatus";
import { useConversations, useConversation } from "@/hooks/useChatHistory";
import type { ChatMessage as ChatMessageType } from "@focusflow/types";
import { Bot, Loader2, Settings, Cloud, Lock, AlertCircle, Download } from "lucide-react";

// Import new chat components
import {
  ChatContainer,
  ChatMessage,
  ChatInput,
  ChatSuggestions,
  ChatThinking,
  type ChatContainerRef,
  type Suggestion,
} from "@/components/chat";

// Import chat history components
import { ChatHistoryPanel } from "@/components/chat/ChatHistoryPanel";
import { ChatHistoryToggle } from "@/components/chat/ChatHistoryToggle";

// Import AI Settings Dialog
import { AISettingsDialog } from "@/components/ai-settings/AISettingsDialog";

// Quick suggestion prompts that appear above the input
const QUICK_SUGGESTIONS: Suggestion[] = [
  { id: "quick-tip", text: "Give me a quick tip", icon: "sparkles" },
  { id: "patterns", text: "Show my patterns", icon: "trending" },
  { id: "focus-on", text: "What should I focus on?", icon: "target" },
];

export function AICoach() {
  const [messages, setMessages] = useState<ChatMessageType[]>([]);
  const [input, setInput] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const [activeConversationId, setActiveConversationId] = useState<string | null>(null);
  const [isViewingHistory, setIsViewingHistory] = useState(false);
  const chatContainerRef = useRef<ChatContainerRef>(null);

  const chat = useCoachChat();
  const conversationsQuery = useConversations({ limit: 50, daysBack: 30 });
  const conversationQuery = useConversation(activeConversationId);

  // Get LLM status for provider information
  const {
    status: llmStatus,
    isLoading: llmLoading,
    isAvailable,
    isEnabled,
    modelName,
    provider,
    errorMessage,
    refetch: refetchStatus,
  } = useLlmStatusManager({
    refetchInterval: 10000, // Check every 10 seconds
  });

  // Determine provider state
  const isLocalProvider = provider === "local-llama";
  const isCloudProvider =
    provider === "openai" ||
    provider === "anthropic" ||
    provider === "google" ||
    provider === "openrouter";
  const hasProvider = provider !== "none";
  const isModelLoading =
    llmStatus?.model_status === "downloading" || llmStatus?.model_status === "loading";

  // For local providers: Check if model file is available (even if not loaded into memory yet)
  // For cloud providers: Check if provider is configured with API key
  // The model will be loaded into memory on first inference, so we don't require model_loaded=true
  const isReady = isAvailable && isEnabled;

  // Check if model needs to be downloaded (ONLY for local provider when file doesn't exist)
  // This should ONLY be true when available=false (model file not downloaded)
  const needsModelDownload =
    isLocalProvider &&
    !isAvailable &&
    (errorMessage.toLowerCase().includes("not downloaded") ||
      llmStatus?.model_status === "not_downloaded");

  // Load conversation when selected from history
  // Use derived state or conditional rendering instead of setState in effect
  const loadedMessages = conversationQuery.data?.messages;

  useEffect(() => {
    if (loadedMessages) {
      // Schedule state updates in a microtask to avoid cascading renders
      Promise.resolve().then(() => {
        setMessages(loadedMessages);
        setIsViewingHistory(true);
      });
    }
  }, [loadedMessages]);

  // Only initialize welcome message once on mount if needed
  useEffect(() => {
    if (messages.length === 0 && !activeConversationId) {
      const welcomeMessage: ChatMessageType = {
        role: "assistant",
        content:
          "Hi! I'm your focus coach. I can help you plan sessions, understand your distraction patterns, and build better focus habits. What would you like to work on?",
        timestamp: new Date().toISOString(),
      };
      setMessages([welcomeMessage]);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only run on mount

  // Handle starting a new chat
  const handleNewChat = useCallback(() => {
    setActiveConversationId(null);
    setIsViewingHistory(false);
    setMessages([
      {
        role: "assistant",
        content:
          "Hi! I'm your focus coach. I can help you plan sessions, understand your distraction patterns, and build better focus habits. What would you like to work on?",
        timestamp: new Date().toISOString(),
      },
    ]);
    setHistoryOpen(false);
  }, []);

  // Handle selecting a conversation from history
  const handleConversationSelect = useCallback((id: string) => {
    setActiveConversationId(id);
    setHistoryOpen(false);
  }, []);

  // Handle continuing from a historical conversation
  const handleContinueConversation = useCallback(() => {
    setIsViewingHistory(false);
  }, []);

  const handleSend = useCallback(async () => {
    if (!input.trim() || chat.isPending) return;

    // Check if provider is ready before sending
    if (!isReady && !hasProvider) {
      console.error("No AI provider configured");
      return;
    }

    if (isModelLoading) {
      console.error("Model is still loading");
      return;
    }

    const userMessage: ChatMessageType = {
      role: "user",
      content: input.trim(),
      timestamp: new Date().toISOString(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput("");

    try {
      const response = await chat.mutateAsync({ message: userMessage.content });
      const assistantMessage: ChatMessageType = {
        role: "assistant",
        content: response.message,
        timestamp: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, assistantMessage]);
    } catch (error) {
      const errorMessage: ChatMessageType = {
        role: "assistant",
        content: "Sorry, I had trouble processing that. Please try again.",
        timestamp: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, errorMessage]);
      console.error("Chat error:", error);
    }
  }, [input, chat, isReady, hasProvider, isModelLoading]);

  // Handle quick suggestion clicks - directly send the message
  const handleSuggestionClick = useCallback(
    async (suggestion: string) => {
      if (chat.isPending || !isReady) return;

      const userMessage: ChatMessageType = {
        role: "user",
        content: suggestion,
        timestamp: new Date().toISOString(),
      };

      setMessages((prev) => [...prev, userMessage]);

      try {
        const response = await chat.mutateAsync({ message: suggestion });
        const assistantMessage: ChatMessageType = {
          role: "assistant",
          content: response.message,
          timestamp: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, assistantMessage]);
      } catch (error) {
        const errorMessage: ChatMessageType = {
          role: "assistant",
          content: "Sorry, I had trouble processing that. Please try again.",
          timestamp: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, errorMessage]);
        console.error("Chat error:", error);
      }
    },
    [chat, isReady]
  );

  // Show quick suggestions when chat is not active
  const showQuickSuggestions = messages.length <= 1 && !chat.isPending;

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-start justify-between mb-4 flex-shrink-0">
        <div className="flex items-center gap-2">
          {/* Chat History Toggle */}
          <ChatHistoryToggle
            onClick={() => setHistoryOpen(true)}
            conversationCount={conversationsQuery.data?.total}
            isActive={historyOpen}
          />

          <div>
            <h2 className="text-xl font-semibold flex items-center gap-2">
              <Bot className="h-5 w-5" />
              AI Focus Coach
            </h2>
            <p className="text-sm text-muted-foreground">
              {isLocalProvider
                ? "Your personal productivity assistant, running 100% locally"
                : isCloudProvider
                  ? "Your personal productivity assistant, powered by cloud AI"
                  : "Configure an AI provider to get started"}
            </p>
          </div>
        </div>

        {/* Model Indicator & Settings */}
        <div className="flex items-center gap-2">
          {/* Model Badge */}
          {hasProvider && (
            <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-muted border">
              {isLocalProvider ? (
                <Lock className="h-3.5 w-3.5 text-green-500" />
              ) : isCloudProvider ? (
                <Cloud className="h-3.5 w-3.5 text-blue-500" />
              ) : null}
              <span className="text-xs font-medium">
                {llmLoading ? "Checking..." : modelName || "No model"}
              </span>
              {isModelLoading && <Loader2 className="h-3 w-3 animate-spin text-muted-foreground" />}
            </div>
          )}

          {/* Settings Button */}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setSettingsOpen(true)}
            className="h-8 w-8"
            aria-label="AI Settings"
          >
            <Settings className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Full-Screen Chat Area */}
      <Card className="relative flex flex-col flex-1 overflow-hidden min-h-0">
        {/* Messages Area */}
        <ChatContainer ref={chatContainerRef} className="flex-1">
          {messages.map((msg, idx) => (
            <ChatMessage
              key={`${msg.timestamp}-${idx}`}
              content={msg.content}
              role={msg.role}
              timestamp={msg.timestamp}
              showCopy={msg.role === "assistant"}
            />
          ))}

          {/* Thinking indicator */}
          {chat.isPending && <ChatThinking variant="dots" />}
        </ChatContainer>

        {/* Input Area */}
        <div className="border-t flex-shrink-0">
          {/* Viewing History Banner */}
          {isViewingHistory && (
            <div className="px-4 py-3 bg-muted/50 border-b flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="h-2 w-2 rounded-full bg-amber-500 animate-pulse" />
                <span className="text-sm text-muted-foreground">
                  Viewing conversation from{" "}
                  {conversationQuery.data?.createdAt
                    ? new Date(conversationQuery.data.createdAt).toLocaleDateString()
                    : "history"}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <Button variant="outline" size="sm" onClick={handleContinueConversation}>
                  Continue Conversation
                </Button>
                <Button variant="ghost" size="sm" onClick={handleNewChat}>
                  New Chat
                </Button>
              </div>
            </div>
          )}

          {/* Quick Suggestion Buttons */}
          {showQuickSuggestions && !isViewingHistory && (
            <div className="px-4 pt-3 pb-2">
              <ChatSuggestions
                suggestions={QUICK_SUGGESTIONS}
                onSelect={handleSuggestionClick}
                variant="inline"
                disabled={!isReady}
              />
            </div>
          )}

          {/* Input */}
          <ChatInput
            value={input}
            onChange={setInput}
            onSend={handleSend}
            placeholder={
              isViewingHistory
                ? "Continue this conversation or start a new one..."
                : isReady
                  ? "Ask me anything about focus..."
                  : "Configure AI to start chatting..."
            }
            disabled={chat.isPending || !isReady || isViewingHistory}
            isLoading={chat.isPending}
          />
        </div>

        {/* Overlay States */}
        {/* Loading State */}
        {llmLoading && !llmStatus && (
          <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex flex-col items-center justify-center z-10">
            <Loader2 className="animate-spin h-8 w-8 mb-4 text-primary" />
            <p className="text-sm font-medium">Checking AI status...</p>
          </div>
        )}

        {/* Model Downloading State */}
        {isModelLoading && hasProvider && (
          <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex flex-col items-center justify-center z-10">
            <Loader2 className="animate-spin h-8 w-8 mb-4 text-primary" />
            <p className="text-sm font-medium mb-2">
              {llmStatus.model_status === "downloading"
                ? "Downloading model..."
                : "Loading model..."}
            </p>
            {/* Indeterminate progress - actual progress tracking would require backend events */}
            <div className="w-64 mt-2 h-2 bg-muted rounded-full overflow-hidden">
              <div className="h-full bg-primary animate-pulse w-full opacity-70" />
            </div>
            <p className="text-xs text-muted-foreground mt-2">This may take a few minutes</p>
          </div>
        )}

        {/* Model Needs Download State */}
        {!llmLoading && needsModelDownload && !isModelLoading && (
          <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex flex-col items-center justify-center z-10">
            <Download className="h-12 w-12 mb-4 text-amber-500" />
            <p className="text-sm font-medium mb-1">Model Download Required</p>
            <p className="text-xs text-muted-foreground mb-4 max-w-xs text-center">
              The local AI model needs to be downloaded before you can use the coach. This is a
              one-time download of approximately 2-3 GB.
            </p>
            <Button onClick={() => setSettingsOpen(true)}>
              <Download className="h-4 w-4 mr-2" />
              Download Model
            </Button>
          </div>
        )}

        {/* No Provider State */}
        {!llmLoading && !hasProvider && (
          <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex flex-col items-center justify-center z-10">
            <Bot className="h-12 w-12 mb-4 text-muted-foreground" />
            <p className="text-sm font-medium mb-1">No AI provider configured</p>
            <p className="text-xs text-muted-foreground mb-4 max-w-xs text-center">
              Configure a local or cloud AI provider to start coaching
            </p>
            <Button onClick={() => setSettingsOpen(true)}>Configure AI</Button>
          </div>
        )}

        {/* Error State (but not when model needs download - that has its own overlay) */}
        {!llmLoading &&
          hasProvider &&
          llmStatus?.error &&
          !isModelLoading &&
          !needsModelDownload && (
            <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex flex-col items-center justify-center z-10">
              <AlertCircle className="h-12 w-12 mb-4 text-destructive" />
              <p className="text-sm font-medium mb-1">AI Provider Error</p>
              <p className="text-xs text-muted-foreground mb-4 max-w-xs text-center">
                {errorMessage}
              </p>
              <div className="flex gap-2">
                <Button variant="outline" onClick={() => refetchStatus()}>
                  Retry
                </Button>
                <Button onClick={() => setSettingsOpen(true)}>Configure AI</Button>
              </div>
            </div>
          )}
      </Card>

      {/* Chat History Panel */}
      <ChatHistoryPanel
        open={historyOpen}
        onOpenChange={setHistoryOpen}
        activeConversationId={activeConversationId}
        onConversationSelect={handleConversationSelect}
        onNewChat={handleNewChat}
      />

      {/* AI Settings Dialog */}
      <AISettingsDialog open={settingsOpen} onOpenChange={setSettingsOpen} />
    </div>
  );
}
