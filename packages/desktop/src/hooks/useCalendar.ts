// hooks/useCalendar.ts - Calendar integration hooks for Tauri commands

import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  CalendarConnection,
  CalendarEvent,
  CalendarProvider,
  FocusBlockSuggestion,
  MeetingLoad,
  AuthorizationUrl,
  OAuthConfigStatus,
} from "@focusflow/types";

// Query keys
export const calendarQueryKeys = {
  connections: ["calendarConnections"] as const,
  events: (startDate: string, endDate: string) => ["calendarEvents", startDate, endDate] as const,
  suggestions: ["focusSuggestions"] as const,
  meetingLoad: ["meetingLoad"] as const,
  oauthConfigStatus: ["oauthConfigStatus"] as const,
};

// Get all calendar connections
export function useCalendarConnections() {
  return useQuery({
    queryKey: calendarQueryKeys.connections,
    queryFn: async () => {
      return invoke<CalendarConnection[]>("get_calendar_connections");
    },
  });
}

// Get OAuth configuration status
export function useOAuthConfigStatus() {
  return useQuery({
    queryKey: calendarQueryKeys.oauthConfigStatus,
    queryFn: async () => {
      return invoke<OAuthConfigStatus>("get_oauth_config_status");
    },
    staleTime: 1000 * 60 * 30, // 30 minutes - config rarely changes
  });
}

// Get calendar events for a date range
export function useCalendarEvents(startDate: string, endDate: string) {
  return useQuery({
    queryKey: calendarQueryKeys.events(startDate, endDate),
    queryFn: async () => {
      return invoke<CalendarEvent[]>("get_calendar_events", { startDate, endDate });
    },
    enabled: !!startDate && !!endDate,
  });
}

// Get focus block suggestions
export function useFocusSuggestions() {
  return useQuery({
    queryKey: calendarQueryKeys.suggestions,
    queryFn: async () => {
      return invoke<FocusBlockSuggestion[]>("get_focus_suggestions");
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

// Get meeting load statistics
export function useMeetingLoad() {
  return useQuery({
    queryKey: calendarQueryKeys.meetingLoad,
    queryFn: async () => {
      return invoke<MeetingLoad>("get_meeting_load");
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

// Start OAuth flow for a calendar provider
export function useStartCalendarOAuth() {
  return useMutation({
    mutationFn: async (provider: CalendarProvider) => {
      return invoke<AuthorizationUrl>("start_calendar_oauth", { provider });
    },
  });
}

// Complete OAuth flow
export function useCompleteCalendarOAuth() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      provider,
      code,
      receivedState,
    }: {
      provider: CalendarProvider;
      code: string;
      receivedState: string;
    }) => {
      return invoke<CalendarConnection>("complete_calendar_oauth", {
        provider,
        code,
        receivedState,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: calendarQueryKeys.connections });
      queryClient.invalidateQueries({ queryKey: ["calendarEvents"] });
      queryClient.invalidateQueries({ queryKey: calendarQueryKeys.suggestions });
      queryClient.invalidateQueries({ queryKey: calendarQueryKeys.meetingLoad });
    },
  });
}

// Disconnect a calendar provider
export function useDisconnectCalendar() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (provider: CalendarProvider) => {
      return invoke("disconnect_calendar", { provider });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: calendarQueryKeys.connections });
      queryClient.invalidateQueries({ queryKey: ["calendarEvents"] });
      queryClient.invalidateQueries({ queryKey: calendarQueryKeys.suggestions });
      queryClient.invalidateQueries({ queryKey: calendarQueryKeys.meetingLoad });
    },
  });
}
