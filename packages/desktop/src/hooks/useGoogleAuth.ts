// hooks/useGoogleAuth.ts - Google OAuth sign-in hook

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { listen } from "@tauri-apps/api/event";
import { useAuthStore } from "@/stores/authStore";

interface GoogleOAuthResponse {
  authUrl: string;
  state: string;
}

interface AuthResponse {
  accessToken: string;
  refreshToken: string;
  user: {
    id: string;
    email: string;
    createdAt: string;
    subscriptionTier: string;
  };
}

export function useGoogleAuth() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const setUser = useAuthStore((s) => s.setUser);
  const setAuthenticated = useAuthStore((s) => s.setAuthenticated);
  const setError_ = useAuthStore((s) => s.setError);

  // Listen for deep link callback
  useEffect(() => {
    const handleDeepLink = async (event: { payload: { url: string } }) => {
      const url = event.payload.url || (event.payload as unknown as string);

      if (typeof url === "string" && url.startsWith("focusflow://oauth/auth-callback")) {
        try {
          setIsLoading(true);
          setError(null);

          // Parse the callback URL
          const urlObj = new URL(url);
          const code = urlObj.searchParams.get("code");
          const state = urlObj.searchParams.get("state");
          const errorParam = urlObj.searchParams.get("error");

          if (errorParam) {
            throw new Error(`OAuth error: ${errorParam}`);
          }

          if (!code || !state) {
            throw new Error("Missing authorization code or state");
          }

          // Complete the OAuth flow
          const response = await invoke<AuthResponse>("complete_google_oauth", {
            code,
            receivedState: state,
          });

          // Transform response to match UserInfo interface (snake_case)
          const userInfo = {
            id: response.user.id,
            email: response.user.email,
            created_at: response.user.createdAt,
            subscription_tier: response.user.subscriptionTier as "free" | "pro" | "team",
          };

          // Update auth store (tokens are stored in backend)
          setUser(userInfo);
          setAuthenticated(true);

          console.log("Google OAuth completed successfully");
        } catch (e) {
          const errorMessage = e instanceof Error ? e.message : "OAuth failed";
          setError(errorMessage);
          setError_(errorMessage);
          console.error("Google OAuth error:", e);
        } finally {
          setIsLoading(false);
        }
      }
    };

    // Wrap async handler to satisfy void return type
    const unlisten = listen<{ url: string }>("deep-link", (event) => {
      void handleDeepLink(event);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [setUser, setAuthenticated, setError_]);

  const startGoogleAuth = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);

      // Get the auth URL from backend
      const response = await invoke<GoogleOAuthResponse>("start_google_oauth");

      // Open the auth URL in the system browser
      await open(response.authUrl);

      // User will be redirected back to focusflow://oauth/auth-callback
      // which will trigger the deep-link listener above
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : "Failed to start Google sign-in";
      setError(errorMessage);
      setError_(errorMessage);
      setIsLoading(false);
      console.error("Failed to start Google auth:", e);
    }
  }, [setError_]);

  return {
    startGoogleAuth,
    isLoading,
    error,
  };
}
