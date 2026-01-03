// hooks/useLlmStatus.ts - LLM status and health monitoring hooks

import { invoke } from "@tauri-apps/api/core";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

/**
 * LLM status information from the backend
 */
export interface LlmStatus {
  /** Whether LLM features are available */
  available: boolean;
  /** Provider type: "local-llama", "none", etc. */
  provider: string;
  /** Currently loaded model name */
  model?: string;
  /** Model status (loaded, downloading, etc.) */
  model_status?: string;
  /** Connection/health error if any */
  error?: string;
  /** Whether the model is currently loaded in memory */
  model_loaded: boolean;
  /** Feature flag status */
  feature_enabled: boolean;
}

/**
 * Detailed model information
 */
export interface ModelDetails {
  name: string;
  description: string;
  size_mb: number;
  status: string;
  is_loaded: boolean;
  supports_streaming: boolean;
}

// Query keys for React Query
export const llmQueryKeys = {
  status: ["llm", "status"] as const,
  modelDetails: ["llm", "modelDetails"] as const,
};

/**
 * Hook to get LLM status with automatic polling and caching
 *
 * @param options - Query options
 * @param options.enabled - Whether to enable the query (default: true)
 * @param options.refetchInterval - How often to refetch in ms (default: 60000 = 1 minute)
 * @param options.staleTime - How long data is considered fresh in ms (default: 30000 = 30 seconds)
 *
 * @returns Object containing:
 * - status: LlmStatus object or undefined if loading
 * - isLoading: Whether the query is loading
 * - error: Error object if query failed
 * - refetch: Function to manually refetch the status
 * - isAvailable: Convenience boolean for status?.available && !status?.error
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { status, isLoading, isAvailable, refetch } = useLlmStatus();
 *
 *   if (isLoading) return <div>Loading...</div>;
 *
 *   if (!isAvailable) {
 *     return (
 *       <div>
 *         <p>AI Offline: {status?.error}</p>
 *         <button onClick={() => refetch()}>Retry</button>
 *       </div>
 *     );
 *   }
 *
 *   return <div>AI ready with {status?.model}</div>;
 * }
 * ```
 */
export function useLlmStatus(options?: {
  enabled?: boolean;
  refetchInterval?: number;
  staleTime?: number;
}) {
  const {
    enabled = true,
    refetchInterval = 60000, // Check every minute
    staleTime = 30000, // Cache for 30 seconds
  } = options || {};

  const query = useQuery({
    queryKey: llmQueryKeys.status,
    queryFn: async () => {
      return invoke<LlmStatus>("get_llm_status");
    },
    staleTime,
    refetchInterval,
    enabled,
    retry: 2, // Retry failed requests twice
  });

  return {
    ...query,
    status: query.data,
    isAvailable: Boolean(query.data?.available && !query.data?.error),
  };
}

/**
 * Hook for simple boolean LLM connection check
 *
 * This is lighter-weight than useLlmStatus and just returns true/false.
 * Use this when you only need to know if LLM is working, not the details.
 *
 * @returns Object containing:
 * - isConnected: Boolean indicating if LLM is connected and healthy
 * - isLoading: Whether the query is loading
 * - error: Error object if query failed
 * - refetch: Function to manually refetch
 *
 * @example
 * ```tsx
 * function AIFeatureToggle() {
 *   const { isConnected } = useLlmConnection();
 *   return <Switch disabled={!isConnected} />;
 * }
 * ```
 */
export function useLlmConnection(options?: {
  enabled?: boolean;
  refetchInterval?: number;
}) {
  const {
    enabled = true,
    refetchInterval = 60000,
  } = options || {};

  const query = useQuery({
    queryKey: ["llm", "connection"],
    queryFn: async () => {
      return invoke<boolean>("check_llm_connection");
    },
    staleTime: 30000,
    refetchInterval,
    enabled,
  });

  return {
    ...query,
    isConnected: query.data ?? false,
  };
}

/**
 * Hook to force refresh LLM status (bypasses cache)
 *
 * Use this after making changes that affect LLM status,
 * like loading/unloading models or changing configuration.
 *
 * @returns Mutation object with refresh function
 *
 * @example
 * ```tsx
 * function ModelLoader() {
 *   const { mutate: refreshStatus } = useRefreshLlmStatus();
 *   const loadModel = useLoadModel();
 *
 *   const handleLoad = async () => {
 *     await loadModel.mutateAsync();
 *     refreshStatus(); // Force refresh after loading
 *   };
 *
 *   return <button onClick={handleLoad}>Load Model</button>;
 * }
 * ```
 */
export function useRefreshLlmStatus() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      return invoke<LlmStatus>("refresh_llm_status");
    },
    onSuccess: (data) => {
      // Update the cache with fresh data
      queryClient.setQueryData(llmQueryKeys.status, data);
    },
  });
}

/**
 * Hook to get detailed model information
 *
 * @returns Query object with model details
 *
 * @example
 * ```tsx
 * function ModelInfo() {
 *   const { data: details, isLoading } = useModelDetails();
 *
 *   if (isLoading) return <div>Loading...</div>;
 *   if (!details) return null;
 *
 *   return (
 *     <div>
 *       <h3>{details.name}</h3>
 *       <p>{details.description}</p>
 *       <p>Size: {details.size_mb}MB</p>
 *       <p>Status: {details.status}</p>
 *     </div>
 *   );
 * }
 * ```
 */
export function useModelDetails(options?: {
  enabled?: boolean;
}) {
  const { enabled = true } = options || {};

  return useQuery({
    queryKey: llmQueryKeys.modelDetails,
    queryFn: async () => {
      return invoke<ModelDetails>("get_model_details");
    },
    staleTime: 60000, // Cache for 1 minute
    enabled,
  });
}

/**
 * Hook to clear the LLM status cache
 *
 * Useful for debugging or forcing immediate re-checks.
 *
 * @returns Mutation object
 */
export function useClearLlmCache() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      await invoke("clear_llm_cache");
    },
    onSuccess: () => {
      // Invalidate all LLM queries to force refetch
      queryClient.invalidateQueries({ queryKey: ["llm"] });
    },
  });
}

/**
 * Combined hook for LLM status with helpful utilities
 *
 * This hook provides everything you need for managing LLM status in one place.
 *
 * @returns Object with all LLM status utilities
 *
 * @example
 * ```tsx
 * function AIStatusBadge() {
 *   const {
 *     status,
 *     isLoading,
 *     isAvailable,
 *     isEnabled,
 *     modelName,
 *     errorMessage,
 *     refresh,
 *   } = useLlmStatusManager();
 *
 *   if (isLoading) {
 *     return <Badge variant="secondary">Checking AI...</Badge>;
 *   }
 *
 *   if (!isEnabled) {
 *     return <Badge variant="outline">AI Disabled</Badge>;
 *   }
 *
 *   if (!isAvailable) {
 *     return (
 *       <Badge variant="destructive" onClick={refresh}>
 *         AI Offline: {errorMessage}
 *       </Badge>
 *     );
 *   }
 *
 *   return <Badge variant="success">AI Ready ({modelName})</Badge>;
 * }
 * ```
 */
export function useLlmStatusManager(options?: {
  enabled?: boolean;
  refetchInterval?: number;
}) {
  const { status, isLoading, error, refetch, isAvailable } = useLlmStatus(options);
  const { mutate: refresh } = useRefreshLlmStatus();

  return {
    // Raw data
    status,
    isLoading,
    error,

    // Convenience booleans
    isAvailable,
    isEnabled: status?.feature_enabled ?? false,
    isModelLoaded: status?.model_loaded ?? false,

    // Convenience strings
    modelName: status?.model ?? "None",
    provider: status?.provider ?? "none",
    errorMessage: status?.error ?? error?.message ?? "Unknown error",

    // Actions
    refetch,
    refresh,
  };
}
