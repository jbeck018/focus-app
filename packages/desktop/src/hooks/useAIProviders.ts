// hooks/useAIProviders.ts - AI Provider management with React Query

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

/**
 * Provider types supported by the application
 */
export type ProviderType = "local" | "openai" | "anthropic" | "google" | "openrouter";

/**
 * Provider configuration - matches Rust backend enum structure
 */
export type ProviderConfig =
  | {
      provider: "openai";
      api_key: string;
      model: string;
      base_url?: string;
      organization?: string;
    }
  | {
      provider: "anthropic";
      api_key: string;
      model: string;
    }
  | {
      provider: "google";
      api_key: string;
      model: string;
    }
  | {
      provider: "openrouter";
      api_key: string;
      model: string;
      site_url?: string;
      app_name?: string;
    }
  | {
      provider: "local";
      model_path: string;
    };

/**
 * Provider info with connection status
 */
export interface ProviderInfo {
  id: ProviderType;
  name: string;
  description: string;
  requiresApiKey: boolean;
  status: "connected" | "disconnected" | "error";
  errorMessage?: string;
  currentModel?: string;
}

/**
 * Model information for a provider
 */
export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  sizeMb?: number;
  recommended?: boolean;
}

/**
 * Model download progress event
 */
export interface DownloadProgress {
  percent: number;
  downloadedMb: number;
  totalMb: number;
  modelName: string;
}

/**
 * Model download status
 */
export interface DownloadStatus {
  isDownloading: boolean;
  modelName?: string;
  progress?: DownloadProgress;
  error?: string;
}

// Query keys for React Query
export const aiProviderQueryKeys = {
  providers: ["ai", "providers"] as const,
  activeProvider: ["ai", "activeProvider"] as const,
  availableModels: (provider: ProviderType) => ["ai", "models", provider] as const,
  downloadStatus: ["ai", "downloadStatus"] as const,
  apiKey: (provider: ProviderType) => ["ai", "apiKey", provider] as const,
};

/**
 * Hook to get list of available providers
 *
 * @returns Query object with providers list
 *
 * @example
 * ```tsx
 * const { data: providers, isLoading } = useProviders();
 * ```
 */
export function useProviders() {
  return useQuery({
    queryKey: aiProviderQueryKeys.providers,
    queryFn: async () => {
      return invoke<ProviderInfo[]>("list_providers");
    },
    staleTime: 60000, // Cache for 1 minute
  });
}

/**
 * Hook to get the currently active provider
 *
 * @returns Query object with active provider config
 *
 * @example
 * ```tsx
 * const { data: activeProvider, isLoading } = useActiveProvider();
 * if (activeProvider) {
 *   console.log(`Using ${activeProvider.provider} with ${activeProvider.modelId}`);
 * }
 * ```
 */
export function useActiveProvider() {
  return useQuery({
    queryKey: aiProviderQueryKeys.activeProvider,
    queryFn: async () => {
      return invoke<ProviderConfig | null>("get_active_provider");
    },
    staleTime: 30000, // Cache for 30 seconds
    refetchInterval: 60000, // Refetch every minute
  });
}

/**
 * Hook to get available models for a provider
 *
 * @param provider - Provider type
 * @param options - Query options
 *
 * @example
 * ```tsx
 * const { data: models, isLoading } = useAvailableModels("openai");
 * ```
 */
export function useAvailableModels(
  provider: ProviderType | null,
  options?: { enabled?: boolean }
) {
  const { enabled = true } = options || {};

  return useQuery({
    queryKey: aiProviderQueryKeys.availableModels(provider!),
    queryFn: async () => {
      return invoke<ModelInfo[]>("list_models", { provider });
    },
    enabled: enabled && provider !== null,
    staleTime: 300000, // Cache for 5 minutes
  });
}

/**
 * Hook to set the active provider
 *
 * @returns Mutation object to set provider
 *
 * @example
 * ```tsx
 * const setProvider = useSetProvider();
 *
 * const handleSelectProvider = (config: ProviderConfig) => {
 *   setProvider.mutate(config, {
 *     onSuccess: () => toast.success("Provider updated"),
 *     onError: (error) => toast.error(error.message),
 *   });
 * };
 * ```
 */
export function useSetProvider() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (config: ProviderConfig) => {
      return invoke<void>("set_active_provider", { config });
    },
    onSuccess: () => {
      // Invalidate active provider and LLM status
      queryClient.invalidateQueries({ queryKey: aiProviderQueryKeys.activeProvider });
      queryClient.invalidateQueries({ queryKey: ["llm", "status"] });
    },
  });
}

/**
 * Hook to test provider connection
 *
 * @returns Mutation object to test connection
 *
 * @example
 * ```tsx
 * const testConnection = useTestConnection();
 *
 * const handleTest = () => {
 *   testConnection.mutate(
 *     { provider: "openai", apiKey: "sk-...", modelId: "gpt-4o" },
 *     {
 *       onSuccess: () => toast.success("Connection successful!"),
 *       onError: (error) => toast.error(`Failed: ${error.message}`),
 *     }
 *   );
 * };
 * ```
 */
export function useTestConnection() {
  return useMutation({
    mutationFn: async (config: ProviderConfig) => {
      return invoke<{ success: boolean; error?: string }>(
        "test_provider_connection",
        { config }
      );
    },
  });
}

/**
 * Hook to save API key for a provider
 *
 * @returns Mutation object to save API key
 *
 * @example
 * ```tsx
 * const saveApiKey = useSaveApiKey();
 *
 * const handleSave = (provider: ProviderType, key: string) => {
 *   saveApiKey.mutate({ provider, apiKey: key });
 * };
 * ```
 */
export function useSaveApiKey() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ provider, apiKey }: { provider: ProviderType; apiKey: string }) => {
      return invoke<void>("save_api_key", { provider, apiKey });
    },
    onSuccess: (_, { provider }) => {
      // Invalidate API key cache for this provider
      queryClient.invalidateQueries({ queryKey: aiProviderQueryKeys.apiKey(provider) });
      queryClient.invalidateQueries({ queryKey: aiProviderQueryKeys.providers });
    },
  });
}

/**
 * Hook to get stored API key for a provider (returns masked version)
 *
 * @param provider - Provider type
 *
 * @example
 * ```tsx
 * const { data: apiKey } = useGetApiKey("openai");
 * // Returns: "sk-...abc123" (masked)
 * ```
 */
export function useGetApiKey(provider: ProviderType | null) {
  return useQuery({
    queryKey: aiProviderQueryKeys.apiKey(provider!),
    queryFn: async () => {
      return invoke<string | null>("get_api_key", { provider });
    },
    enabled: provider !== null,
    staleTime: 300000, // Cache for 5 minutes
  });
}

/**
 * Hook to delete API key for a provider
 *
 * @returns Mutation object to delete API key
 *
 * @example
 * ```tsx
 * const deleteApiKey = useDeleteApiKey();
 *
 * const handleDelete = (provider: ProviderType) => {
 *   deleteApiKey.mutate(provider);
 * };
 * ```
 */
export function useDeleteApiKey() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (provider: ProviderType) => {
      return invoke<void>("delete_api_key", { provider });
    },
    onSuccess: (_, provider) => {
      queryClient.invalidateQueries({ queryKey: aiProviderQueryKeys.apiKey(provider) });
      queryClient.invalidateQueries({ queryKey: aiProviderQueryKeys.providers });
      queryClient.invalidateQueries({ queryKey: aiProviderQueryKeys.activeProvider });
    },
  });
}

/**
 * Hook to download a local model
 *
 * @returns Mutation object to download model
 *
 * @example
 * ```tsx
 * const downloadModel = useDownloadModel();
 *
 * const handleDownload = (modelName: string) => {
 *   downloadModel.mutate(modelName, {
 *     onSuccess: () => toast.success("Download started"),
 *     onError: (error) => toast.error(error.message),
 *   });
 * };
 * ```
 */
export function useDownloadModel() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (modelName: string) => {
      return invoke<void>("download_model", { modelName });
    },
    onSuccess: () => {
      // Invalidate download status to trigger refetch
      queryClient.invalidateQueries({ queryKey: aiProviderQueryKeys.downloadStatus });
    },
  });
}

/**
 * Hook to get current model download status
 *
 * @returns Query object with download status
 *
 * @example
 * ```tsx
 * const { data: status, isLoading } = useModelDownloadStatus();
 * if (status?.isDownloading) {
 *   console.log(`Downloading ${status.modelName}: ${status.progress?.percent}%`);
 * }
 * ```
 */
export function useModelDownloadStatus() {
  return useQuery({
    queryKey: aiProviderQueryKeys.downloadStatus,
    queryFn: async () => {
      return invoke<DownloadStatus>("get_model_download_status");
    },
    staleTime: 1000, // Cache for 1 second (actively downloading)
    refetchInterval: 2000, // Poll every 2 seconds during download
  });
}

/**
 * Hook to listen to model download progress events
 *
 * @param onProgress - Callback for progress updates
 * @param onComplete - Callback for download completion
 * @param onError - Callback for download errors
 *
 * @example
 * ```tsx
 * useModelDownloadProgress(
 *   (progress) => console.log(`${progress.percent}% complete`),
 *   (modelName) => toast.success(`Downloaded ${modelName}`),
 *   (error) => toast.error(`Download failed: ${error}`)
 * );
 * ```
 */
export function useModelDownloadProgress(
  onProgress?: (progress: DownloadProgress) => void,
  onComplete?: (modelName: string) => void,
  onError?: (error: string, modelName?: string) => void
) {
  const queryClient = useQueryClient();

  useEffect(() => {
    const progressUnlisten = listen<DownloadProgress>(
      "model-download-progress",
      (event) => {
        // Update query cache with latest progress
        queryClient.setQueryData(aiProviderQueryKeys.downloadStatus, {
          isDownloading: true,
          modelName: event.payload.modelName,
          progress: event.payload,
          error: undefined,
        });

        onProgress?.(event.payload);
      }
    );

    const completeUnlisten = listen<{ model_name: string }>(
      "model-download-complete",
      (event) => {
        // Update query cache to reflect completion
        queryClient.setQueryData(aiProviderQueryKeys.downloadStatus, {
          isDownloading: false,
          error: undefined,
        });

        // Invalidate models list to show newly downloaded model
        queryClient.invalidateQueries({ queryKey: aiProviderQueryKeys.availableModels("local") });
        queryClient.invalidateQueries({ queryKey: ["llm", "status"] });

        onComplete?.(event.payload.model_name);
      }
    );

    const errorUnlisten = listen<{ error: string; model?: string }>(
      "model-download-error",
      (event) => {
        const currentStatus = queryClient.getQueryData<DownloadStatus>(
          aiProviderQueryKeys.downloadStatus
        );

        // Update query cache to reflect error
        queryClient.setQueryData(aiProviderQueryKeys.downloadStatus, {
          isDownloading: false,
          modelName: event.payload.model || currentStatus?.modelName,
          error: event.payload.error,
        });

        onError?.(event.payload.error, event.payload.model);
      }
    );

    return () => {
      progressUnlisten.then((fn) => fn());
      completeUnlisten.then((fn) => fn());
      errorUnlisten.then((fn) => fn());
    };
  }, [queryClient, onProgress, onComplete, onError]);
}

/**
 * Combined hook for comprehensive AI provider management
 *
 * @returns Object with all provider utilities
 *
 * @example
 * ```tsx
 * function AISettings() {
 *   const {
 *     providers,
 *     activeProvider,
 *     setProvider,
 *     testConnection,
 *     isLoading,
 *   } = useAIProviderManager();
 *
 *   return (
 *     <div>
 *       {providers?.map((provider) => (
 *         <ProviderCard
 *           key={provider.id}
 *           provider={provider}
 *           isActive={activeProvider?.provider === provider.id}
 *           onSelect={(config) => setProvider.mutate(config)}
 *         />
 *       ))}
 *     </div>
 *   );
 * }
 * ```
 */
export function useAIProviderManager() {
  const providers = useProviders();
  const activeProvider = useActiveProvider();
  const setProvider = useSetProvider();
  const testConnection = useTestConnection();
  const saveApiKey = useSaveApiKey();
  const deleteApiKey = useDeleteApiKey();

  return {
    // Queries
    providers: providers.data,
    activeProvider: activeProvider.data,
    isLoading: providers.isLoading || activeProvider.isLoading,
    error: providers.error || activeProvider.error,

    // Mutations
    setProvider,
    testConnection,
    saveApiKey,
    deleteApiKey,

    // Refetch functions
    refetchProviders: providers.refetch,
    refetchActiveProvider: activeProvider.refetch,
  };
}
