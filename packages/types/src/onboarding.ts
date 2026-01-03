// types/onboarding.ts - Type definitions for the onboarding flow

export type OnboardingStep =
  | "welcome"
  | "pillars"
  | "blocklist"
  | "preferences"
  | "tutorial"
  | "complete";

export interface OnboardingState {
  currentStep: OnboardingStep;
  completedSteps: OnboardingStep[];
  isComplete: boolean;
  startedAt?: string;
  completedAt?: string;
}

export interface OnboardingData {
  // Welcome step
  userName: string;

  // Blocklist step
  selectedApps: string[];
  selectedWebsites: string[];

  // Preferences step
  defaultFocusDuration: number;
  defaultBreakDuration: number;
  enableNotifications: boolean;
  autoStartBreaks: boolean;

  // Tutorial step
  viewedTutorials: string[];
}

export interface IndistractablePillar {
  id: string;
  title: string;
  description: string;
  icon: string;
  color: string;
  examples: string[];
  actionItems: string[];
}

export const INDISTRACTABLE_PILLARS: IndistractablePillar[] = [
  {
    id: "internal-triggers",
    title: "Master Internal Triggers",
    description: "Understand the uncomfortable emotional states that prompt you to seek distraction. Learn to surf the urge instead of giving in.",
    icon: "brain",
    color: "from-blue-500 to-cyan-500",
    examples: [
      "Feeling bored during a task",
      "Anxiety about upcoming deadlines",
      "Restlessness when working alone",
      "Curiosity about notifications"
    ],
    actionItems: [
      "Use the journal to log your triggers",
      "Practice the 10-minute rule",
      "Reimagine the task to make it more engaging"
    ]
  },
  {
    id: "make-time-traction",
    title: "Make Time for Traction",
    description: "Schedule your day with intention. Turn your values into time. Without planning, you'll default to distraction.",
    icon: "calendar",
    color: "from-green-500 to-emerald-500",
    examples: [
      "Time-boxing important work",
      "Scheduling breaks and rejuvenation",
      "Blocking time for relationships",
      "Planning weekly review sessions"
    ],
    actionItems: [
      "Create a timeboxed calendar",
      "Use the FocusTimer for dedicated sessions",
      "Align your time with your values"
    ]
  },
  {
    id: "hack-external-triggers",
    title: "Hack Back External Triggers",
    description: "External triggers aren't always bad, but they should serve you, not control you. Reduce, remove, or restructure them.",
    icon: "bell-off",
    color: "from-orange-500 to-amber-500",
    examples: [
      "Social media notifications",
      "Email alerts",
      "Slack messages",
      "News websites"
    ],
    actionItems: [
      "Block distracting apps and websites",
      "Turn off non-essential notifications",
      "Use focus mode during deep work"
    ]
  },
  {
    id: "prevent-with-pacts",
    title: "Prevent Distraction with Pacts",
    description: "Make it harder to do things you don't want to do. Use effort pacts, price pacts, and identity pacts.",
    icon: "shield-check",
    color: "from-purple-500 to-pink-500",
    examples: [
      "Effort pact: Install website blockers",
      "Price pact: Bet money on completing tasks",
      "Identity pact: Tell others your commitments",
      "Use FocusFlow's blocking during sessions"
    ],
    actionItems: [
      "Set up app and website blocking",
      "Create accountability with team features",
      "Make distractions harder to access"
    ]
  }
];

export interface OnboardingSaveRequest {
  userName: string;
  selectedApps: string[];
  selectedWebsites: string[];
  defaultFocusDuration: number;
  defaultBreakDuration: number;
  enableNotifications: boolean;
  autoStartBreaks: boolean;
}

export interface OnboardingCompleteResponse {
  success: boolean;
  userId?: string;
  error?: string;
}

// Common app suggestions for blocklisting
export const SUGGESTED_APPS = [
  "Chrome",
  "Safari",
  "Firefox",
  "Slack",
  "Discord",
  "Messages",
  "WhatsApp",
  "Telegram",
  "Twitter",
  "Facebook",
  "Instagram",
  "TikTok",
  "YouTube",
  "Netflix",
  "Spotify",
  "Steam",
  "Epic Games"
];

// Common website suggestions for blocklisting
export const SUGGESTED_WEBSITES = [
  "twitter.com",
  "facebook.com",
  "instagram.com",
  "reddit.com",
  "youtube.com",
  "netflix.com",
  "tiktok.com",
  "twitch.tv",
  "linkedin.com",
  "news.ycombinator.com",
  "medium.com",
  "buzzfeed.com",
  "dailymail.co.uk",
  "cnn.com",
  "bbc.com"
];

// Tutorial topics
export interface TutorialTopic {
  id: string;
  title: string;
  description: string;
  icon: string;
  duration: string;
}

export const TUTORIAL_TOPICS: TutorialTopic[] = [
  {
    id: "focus-timer",
    title: "Focus Timer",
    description: "Start focused work sessions with customizable durations and automatic blocking",
    icon: "timer",
    duration: "2 min"
  },
  {
    id: "blocking",
    title: "App & Website Blocking",
    description: "Block distracting apps and websites during focus sessions",
    icon: "shield",
    duration: "2 min"
  },
  {
    id: "journal",
    title: "Trigger Journaling",
    description: "Log your internal triggers to understand and overcome them",
    icon: "book-open",
    duration: "2 min"
  },
  {
    id: "analytics",
    title: "Analytics & Insights",
    description: "Track your productivity patterns and see your progress over time",
    icon: "bar-chart",
    duration: "2 min"
  },
  {
    id: "calendar",
    title: "Calendar Integration",
    description: "Sync your calendar and get smart focus time suggestions (Pro)",
    icon: "calendar",
    duration: "2 min"
  },
  {
    id: "ai-coach",
    title: "AI Coach",
    description: "Get personalized advice based on the Indistractable framework (Pro)",
    icon: "bot",
    duration: "2 min"
  }
];
