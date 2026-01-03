/**
 * Type definitions for Website/App Blocking
 *
 * Uses template literal types and branded types for URL/app validation
 */

// Branded types for type safety
export type RuleId = string & { readonly __brand: "RuleId" };
export type Domain = string & { readonly __brand: "Domain" };
export type AppName = string & { readonly __brand: "AppName" };
export type CronExpression = string & { readonly __brand: "CronExpression" };

// Constructors with validation
export const RuleId = (id: string): RuleId => id as RuleId;

export const Domain = (domain: string): Domain => {
  // Basic domain validation
  const domainRegex =
    /^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]$/i;
  if (!domainRegex.test(domain)) {
    throw new Error(`Invalid domain: ${domain}`);
  }
  return domain as Domain;
};

export const AppName = (name: string): AppName => {
  if (name.trim().length === 0) {
    throw new Error("App name cannot be empty");
  }
  return name as AppName;
};

// Block rule type with const assertion
export const RuleType = {
  WEBSITE: "website",
  APP: "app",
  CATEGORY: "category",
} as const;

export type RuleType = (typeof RuleType)[keyof typeof RuleType];

// Schedule type for when blocking is active
export const ScheduleType = {
  ALWAYS: "always",
  FOCUS_ONLY: "focus_only",
  SCHEDULED: "scheduled",
} as const;

export type ScheduleType = (typeof ScheduleType)[keyof typeof ScheduleType];

// Strictness levels using template literal types for better autocomplete
export type StrictnessLevel = "soft" | "medium" | "hard";

// Conditional type for rule targets based on rule type
export type RuleTarget<T extends RuleType> = T extends typeof RuleType.WEBSITE
  ? Domain
  : T extends typeof RuleType.APP
    ? AppName
    : T extends typeof RuleType.CATEGORY
      ? string
      : never;

// Base block rule interface
interface BaseBlockRule {
  readonly id: RuleId;
  readonly enabled: boolean;
  readonly strictness: StrictnessLevel;
  readonly createdAt: number;
}

// Discriminated union for different rule types
export type BlockRule =
  | (BaseBlockRule & {
      readonly ruleType: typeof RuleType.WEBSITE;
      readonly target: Domain;
      readonly scheduleType: ScheduleType;
      readonly scheduleCron?: CronExpression;
    })
  | (BaseBlockRule & {
      readonly ruleType: typeof RuleType.APP;
      readonly target: AppName;
      readonly scheduleType: ScheduleType;
      readonly scheduleCron?: CronExpression;
    })
  | (BaseBlockRule & {
      readonly ruleType: typeof RuleType.CATEGORY;
      readonly target: string;
      readonly scheduleType: ScheduleType;
      readonly scheduleCron?: CronExpression;
    });

// Type guard for rule types
export const isWebsiteRule = (
  rule: BlockRule
): rule is Extract<BlockRule, { ruleType: "website" }> => {
  return rule.ruleType === RuleType.WEBSITE;
};

export const isAppRule = (rule: BlockRule): rule is Extract<BlockRule, { ruleType: "app" }> => {
  return rule.ruleType === RuleType.APP;
};

// Create rule DTOs with type-safe builders
export type CreateWebsiteRuleDTO = {
  readonly ruleType: typeof RuleType.WEBSITE;
  readonly target: Domain;
  readonly scheduleType: ScheduleType;
  readonly scheduleCron?: CronExpression;
  readonly strictness?: StrictnessLevel;
};

export type CreateAppRuleDTO = {
  readonly ruleType: typeof RuleType.APP;
  readonly target: AppName;
  readonly scheduleType: ScheduleType;
  readonly scheduleCron?: CronExpression;
  readonly strictness?: StrictnessLevel;
};

export type CreateCategoryRuleDTO = {
  readonly ruleType: typeof RuleType.CATEGORY;
  readonly target: string;
  readonly scheduleType: ScheduleType;
  readonly scheduleCron?: CronExpression;
  readonly strictness?: StrictnessLevel;
};

export type CreateBlockRuleDTO = CreateWebsiteRuleDTO | CreateAppRuleDTO | CreateCategoryRuleDTO;

// Block event tracking
export interface BlockEvent {
  readonly id: string;
  readonly ruleId: RuleId;
  readonly blockedAt: number;
  readonly target: string;
  readonly wasBypassed: boolean;
  readonly createdAt: number;
}

// Block attempt with metadata
export interface BlockAttempt {
  readonly target: string;
  readonly ruleId: RuleId;
  readonly timestamp: number;
  readonly userAgent?: string;
  readonly processName?: string;
}

// Bypass request with time-locked code
export interface BypassRequest {
  readonly ruleId: RuleId;
  readonly requestedAt: number;
  readonly bypassCode?: string;
  readonly reason?: string;
}

// Result type for blocking operations
export type BlockingResult<T> =
  | { success: true; data: T }
  | { success: false; error: BlockingError };

export type BlockingError =
  | { type: "validation"; field: string; message: string }
  | { type: "not_found"; ruleId: RuleId }
  | { type: "already_exists"; target: string }
  | { type: "permission_denied"; message: string }
  | { type: "system_error"; message: string };

// Predefined category templates
export const BlockCategory = {
  SOCIAL_MEDIA: "social_media",
  NEWS: "news",
  ENTERTAINMENT: "entertainment",
  SHOPPING: "shopping",
  GAMING: "gaming",
  ADULT: "adult",
} as const;

export type BlockCategory = (typeof BlockCategory)[keyof typeof BlockCategory];

// Category to domains mapping (type-safe)
export type CategoryDomains = {
  readonly [K in BlockCategory]: readonly Domain[];
};

// Export predefined category mappings
export const CATEGORY_DOMAINS: CategoryDomains = {
  [BlockCategory.SOCIAL_MEDIA]: [
    Domain("facebook.com"),
    Domain("twitter.com"),
    Domain("instagram.com"),
    Domain("tiktok.com"),
    Domain("linkedin.com"),
  ],
  [BlockCategory.NEWS]: [
    Domain("cnn.com"),
    Domain("bbc.com"),
    Domain("nytimes.com"),
    Domain("reddit.com"),
  ],
  [BlockCategory.ENTERTAINMENT]: [
    Domain("youtube.com"),
    Domain("netflix.com"),
    Domain("twitch.tv"),
  ],
  [BlockCategory.SHOPPING]: [Domain("amazon.com"), Domain("ebay.com")],
  [BlockCategory.GAMING]: [Domain("steam.com"), Domain("epicgames.com")],
  [BlockCategory.ADULT]: [],
} as const;

// Block rule statistics
export interface RuleStats {
  readonly ruleId: RuleId;
  readonly totalBlocks: number;
  readonly bypasses: number;
  readonly lastTriggered: number | null;
  readonly avgBlocksPerDay: number;
}
