// types/team.ts - Team collaboration types

/** Team information */
export interface Team {
  id: string;
  name: string;
  invite_code: string;
  member_count: number;
  created_at: string;
}

/** Member roles */
export type TeamRole = "owner" | "admin" | "member";

/** Team member with role */
export interface TeamMember {
  id: string;
  email: string;
  display_name: string | null;
  role: TeamRole;
  joined_at: string;
  sharing_enabled: boolean;
}

/** Aggregated team statistics */
export interface TeamStats {
  total_focus_hours_this_week: number;
  average_sessions_per_member: number;
  most_productive_day: string | null;
  top_blockers: string[];
  member_count: number;
}

/** Team blocklist item */
export interface TeamBlockedItem {
  id: number;
  item_type: "app" | "website";
  value: string;
  added_by: string;
  added_at: string;
}

/** Privacy settings for team sharing */
export interface TeamPrivacySettings {
  share_focus_time: boolean;
  share_session_count: boolean;
  share_streak: boolean;
  share_productivity_score: boolean;
}

/** Role display info */
export const ROLE_INFO: Record<TeamRole, { label: string; description: string }> = {
  owner: {
    label: "Owner",
    description: "Full control over team settings and members",
  },
  admin: {
    label: "Admin",
    description: "Can manage members and shared blocklist",
  },
  member: {
    label: "Member",
    description: "Can view team stats and use shared blocklist",
  },
};
