// hooks/useAuth.ts - Authentication hooks for Tauri commands

import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { AuthResponse, AuthState, UserInfo } from "@focusflow/types";
import { useAuthStore } from "@/stores/authStore";

// Query keys
export const authQueryKeys = {
  authState: ["authState"] as const,
  currentUser: ["currentUser"] as const,
  isAuthenticated: ["isAuthenticated"] as const,
};

// Get current auth state
export function useAuthState() {
  const setUser = useAuthStore((s) => s.setUser);
  const setAuthenticated = useAuthStore((s) => s.setAuthenticated);

  return useQuery({
    queryKey: authQueryKeys.authState,
    queryFn: async () => {
      const state = await invoke<AuthState>("get_auth_state");
      // Sync with store
      setAuthenticated(state.is_authenticated);
      if (state.user) {
        setUser(state.user);
      }
      return state;
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

// Get current user
export function useCurrentUser() {
  return useQuery({
    queryKey: authQueryKeys.currentUser,
    queryFn: async () => {
      return invoke<UserInfo | null>("get_current_user");
    },
  });
}

// Login mutation
export function useLogin() {
  const queryClient = useQueryClient();
  const setUser = useAuthStore((s) => s.setUser);
  const setLoading = useAuthStore((s) => s.setLoading);
  const setError = useAuthStore((s) => s.setError);

  return useMutation({
    mutationFn: async ({ email, password }: { email: string; password: string }) => {
      setLoading(true);
      setError(null);
      return invoke<AuthResponse>("login", { email, password });
    },
    onSuccess: (data) => {
      setUser(data.user);
      setLoading(false);
      // Invalidate auth queries
      queryClient.invalidateQueries({ queryKey: authQueryKeys.authState });
      queryClient.invalidateQueries({ queryKey: authQueryKeys.currentUser });
    },
    onError: (error: Error) => {
      setError(error.message);
      setLoading(false);
    },
  });
}

// Register mutation
export function useRegister() {
  const queryClient = useQueryClient();
  const setUser = useAuthStore((s) => s.setUser);
  const setLoading = useAuthStore((s) => s.setLoading);
  const setError = useAuthStore((s) => s.setError);

  return useMutation({
    mutationFn: async ({ email, password }: { email: string; password: string }) => {
      setLoading(true);
      setError(null);
      return invoke<AuthResponse>("register", { email, password });
    },
    onSuccess: (data) => {
      setUser(data.user);
      setLoading(false);
      queryClient.invalidateQueries({ queryKey: authQueryKeys.authState });
      queryClient.invalidateQueries({ queryKey: authQueryKeys.currentUser });
    },
    onError: (error: Error) => {
      setError(error.message);
      setLoading(false);
    },
  });
}

// Logout mutation
export function useLogout() {
  const queryClient = useQueryClient();
  const logout = useAuthStore((s) => s.logout);

  return useMutation({
    mutationFn: async () => {
      return invoke("logout");
    },
    onSuccess: () => {
      logout();
      queryClient.invalidateQueries({ queryKey: authQueryKeys.authState });
      queryClient.invalidateQueries({ queryKey: authQueryKeys.currentUser });
    },
  });
}

// Refresh token mutation
export function useRefreshToken() {
  const queryClient = useQueryClient();
  const setUser = useAuthStore((s) => s.setUser);

  return useMutation({
    mutationFn: async () => {
      return invoke<AuthResponse>("refresh_token");
    },
    onSuccess: (data) => {
      setUser(data.user);
      queryClient.invalidateQueries({ queryKey: authQueryKeys.authState });
    },
    onError: () => {
      // Token refresh failed, user needs to login again
      useAuthStore.getState().logout();
    },
  });
}

// Check if authenticated
export function useIsAuthenticated() {
  return useQuery({
    queryKey: authQueryKeys.isAuthenticated,
    queryFn: async () => {
      return invoke<boolean>("is_authenticated");
    },
    staleTime: 1000 * 30, // 30 seconds
  });
}

// Set TrailBase URL
export function useSetTrailbaseUrl() {
  return useMutation({
    mutationFn: async (url: string) => {
      return invoke("set_trailbase_url", { url });
    },
  });
}
