// hooks/useChatHistory.ts - Chat history hooks for conversation management

import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  Conversation,
  ConversationDetail,
  ConversationListResponse,
  ListConversationsRequest,
  CreateConversationRequest,
} from "@focusflow/types";

// Query keys
export const chatHistoryQueryKeys = {
  all: ["chatHistory"] as const,
  conversations: (params?: ListConversationsRequest) =>
    ["chatHistory", "conversations", params] as const,
  conversation: (id: string) => ["chatHistory", "conversation", id] as const,
};

/**
 * Hook to list conversations with pagination
 * @param params - Optional parameters for filtering and pagination
 */
export function useConversations(params?: ListConversationsRequest) {
  return useQuery({
    queryKey: chatHistoryQueryKeys.conversations(params),
    queryFn: async () => {
      return invoke<ConversationListResponse>("list_conversations", {
        request: {
          limit: params?.limit ?? 20,
          offset: params?.offset ?? 0,
          daysBack: params?.daysBack ?? 30,
        },
      });
    },
    staleTime: 1000 * 60 * 2, // 2 minutes
  });
}

/**
 * Hook to get a single conversation with all messages
 * @param id - Conversation ID
 */
export function useConversation(id: string | null) {
  return useQuery({
    queryKey: chatHistoryQueryKeys.conversation(id ?? ""),
    queryFn: async () => {
      if (!id) return null;
      return invoke<ConversationDetail>("get_conversation", {
        conversationId: id,
      });
    },
    enabled: !!id,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

/**
 * Hook to create a new conversation
 */
export function useCreateConversation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request?: CreateConversationRequest) => {
      return invoke<Conversation>("create_conversation", {
        request: request ?? {},
      });
    },
    onSuccess: () => {
      // Invalidate conversation list to refetch
      queryClient.invalidateQueries({
        queryKey: chatHistoryQueryKeys.all,
      });
    },
  });
}

/**
 * Hook to delete a conversation
 */
export function useDeleteConversation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (conversationId: string) => {
      return invoke<void>("delete_conversation", { conversationId });
    },
    onSuccess: (_data, conversationId) => {
      // Remove from cache
      queryClient.removeQueries({
        queryKey: chatHistoryQueryKeys.conversation(conversationId),
      });

      // Invalidate conversation list to refetch
      queryClient.invalidateQueries({
        queryKey: chatHistoryQueryKeys.conversations(),
      });
    },
  });
}

/**
 * Hook to update conversation title
 */
export function useUpdateConversationTitle() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      conversationId,
      title,
    }: {
      conversationId: string;
      title: string;
    }) => {
      return invoke<Conversation>("update_conversation_title", {
        conversationId,
        title,
      });
    },
    onSuccess: (data) => {
      // Update the conversation in cache
      queryClient.setQueryData(
        chatHistoryQueryKeys.conversation(data.id),
        (old: ConversationDetail | undefined) => {
          if (!old) return old;
          return { ...old, title: data.title, updatedAt: data.updatedAt };
        }
      );

      // Invalidate conversation list to show updated title
      queryClient.invalidateQueries({
        queryKey: chatHistoryQueryKeys.conversations(),
      });
    },
  });
}

/**
 * Hook to add a message to a conversation
 */
export function useAddMessageToConversation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      conversationId,
      message,
      role,
    }: {
      conversationId: string;
      message: string;
      role: "user" | "assistant";
    }) => {
      return invoke<ConversationDetail>("add_message_to_conversation", {
        conversationId,
        message,
        role,
      });
    },
    onSuccess: (data) => {
      // Update the conversation in cache
      queryClient.setQueryData(
        chatHistoryQueryKeys.conversation(data.id),
        data
      );

      // Invalidate conversation list to update metadata
      queryClient.invalidateQueries({
        queryKey: chatHistoryQueryKeys.conversations(),
      });
    },
  });
}
