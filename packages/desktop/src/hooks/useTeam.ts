// hooks/useTeam.ts - Team feature hooks for Tauri commands

import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  Team,
  TeamMember,
  TeamStats,
  TeamBlockedItem,
  TeamPrivacySettings,
} from "@focusflow/types";

// Query keys
export const teamQueryKeys = {
  currentTeam: ["team", "current"] as const,
  members: ["team", "members"] as const,
  stats: ["team", "stats"] as const,
  blocklist: ["team", "blocklist"] as const,
  privacy: ["team", "privacy"] as const,
};

// Get current team
export function useCurrentTeam() {
  return useQuery({
    queryKey: teamQueryKeys.currentTeam,
    queryFn: async () => {
      return invoke<Team | null>("get_current_team");
    },
  });
}

// Get team members
export function useTeamMembers() {
  const { data: team } = useCurrentTeam();

  return useQuery({
    queryKey: teamQueryKeys.members,
    queryFn: async () => {
      return invoke<TeamMember[]>("get_team_members");
    },
    enabled: !!team,
  });
}

// Get team statistics
export function useTeamStats() {
  const { data: team } = useCurrentTeam();

  return useQuery({
    queryKey: teamQueryKeys.stats,
    queryFn: async () => {
      return invoke<TeamStats>("get_team_stats");
    },
    enabled: !!team,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

// Get team blocklist
export function useTeamBlocklist() {
  const { data: team } = useCurrentTeam();

  return useQuery({
    queryKey: teamQueryKeys.blocklist,
    queryFn: async () => {
      return invoke<TeamBlockedItem[]>("get_team_blocklist");
    },
    enabled: !!team,
  });
}

// Get privacy settings
export function useTeamPrivacySettings() {
  return useQuery({
    queryKey: teamQueryKeys.privacy,
    queryFn: async () => {
      return invoke<TeamPrivacySettings>("get_team_privacy_settings");
    },
  });
}

// Create team mutation
export function useCreateTeam() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (name: string) => {
      return invoke<Team>("create_team", { name });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: teamQueryKeys.currentTeam });
    },
  });
}

// Join team mutation
export function useJoinTeam() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (inviteCode: string) => {
      return invoke<Team>("join_team", { inviteCode });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: teamQueryKeys.currentTeam });
    },
  });
}

// Leave team mutation
export function useLeaveTeam() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      return invoke<void>("leave_team");
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["team"] });
    },
  });
}

// Add team blocked item mutation
export function useAddTeamBlockedItem() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ itemType, value }: { itemType: string; value: string }) => {
      return invoke<TeamBlockedItem>("add_team_blocked_item", { itemType, value });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: teamQueryKeys.blocklist });
    },
  });
}

// Remove team blocked item mutation
export function useRemoveTeamBlockedItem() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (itemId: number) => {
      return invoke<void>("remove_team_blocked_item", { itemId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: teamQueryKeys.blocklist });
    },
  });
}

// Update privacy settings mutation
export function useUpdateTeamPrivacySettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (settings: TeamPrivacySettings) => {
      return invoke<void>("update_team_privacy_settings", { settings });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: teamQueryKeys.privacy });
    },
  });
}

// Sync team blocklist to local
export function useSyncTeamBlocklist() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      return invoke<number>("sync_team_blocklist");
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["blockedItems"] });
    },
  });
}
