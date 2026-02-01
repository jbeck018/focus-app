// focus_time/parser.rs - Calendar event parsing for Focus Time detection
//
// This module provides functionality to:
// 1. Detect Focus Time events from calendar event titles
// 2. Parse allowed apps from event descriptions
// 3. Expand app categories to individual app lists

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Keywords that trigger Focus Time detection (case-insensitive)
pub const FOCUS_TIME_KEYWORDS: &[&str] = &[
    "focus time",
    "focus block",
    "deep work",
    "focus session",
    "no meetings",
    "heads down",
    "do not disturb",
    "dnd",
    "coding time",
    "writing time",
];

/// App categories that can be expanded to multiple apps
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppCategory {
    Coding,
    Communication,
    Writing,
    Design,
    Productivity,
    Browser,
    Terminal,
    Custom(String),
}

impl AppCategory {
    /// Parse category from string (with or without @ prefix)
    pub fn from_str(s: &str) -> Option<Self> {
        let name = s.trim().trim_start_matches('@').to_lowercase();
        match name.as_str() {
            "coding" | "code" | "dev" | "development" => Some(AppCategory::Coding),
            "communication" | "comm" | "chat" | "messaging" => Some(AppCategory::Communication),
            "writing" | "write" | "docs" | "documentation" => Some(AppCategory::Writing),
            "design" => Some(AppCategory::Design),
            "productivity" | "prod" => Some(AppCategory::Productivity),
            "browser" | "web" => Some(AppCategory::Browser),
            "terminal" | "shell" | "cli" => Some(AppCategory::Terminal),
            other if !other.is_empty() => Some(AppCategory::Custom(other.to_string())),
            _ => None,
        }
    }

    /// Get the apps that belong to this category
    pub fn expand(&self) -> Vec<String> {
        match self {
            AppCategory::Coding => vec![
                "Visual Studio Code".to_string(),
                "Code".to_string(),
                "code".to_string(),
                "VSCodium".to_string(),
                "Cursor".to_string(),
                "WebStorm".to_string(),
                "IntelliJ IDEA".to_string(),
                "PyCharm".to_string(),
                "CLion".to_string(),
                "GoLand".to_string(),
                "RustRover".to_string(),
                "Xcode".to_string(),
                "Android Studio".to_string(),
                "Sublime Text".to_string(),
                "Atom".to_string(),
                "Vim".to_string(),
                "nvim".to_string(),
                "Neovim".to_string(),
                "Emacs".to_string(),
                "Zed".to_string(),
            ],
            AppCategory::Communication => vec![
                "Slack".to_string(),
                "Discord".to_string(),
                "Microsoft Teams".to_string(),
                "Teams".to_string(),
                "Zoom".to_string(),
                "Telegram".to_string(),
                "WhatsApp".to_string(),
                "Messages".to_string(),
                "Mail".to_string(),
                "Outlook".to_string(),
                "Gmail".to_string(),
            ],
            AppCategory::Writing => vec![
                "Microsoft Word".to_string(),
                "Word".to_string(),
                "Google Docs".to_string(),
                "Pages".to_string(),
                "Notion".to_string(),
                "Obsidian".to_string(),
                "Bear".to_string(),
                "Ulysses".to_string(),
                "iA Writer".to_string(),
                "Typora".to_string(),
                "Notes".to_string(),
            ],
            AppCategory::Design => vec![
                "Figma".to_string(),
                "Sketch".to_string(),
                "Adobe Photoshop".to_string(),
                "Photoshop".to_string(),
                "Adobe Illustrator".to_string(),
                "Illustrator".to_string(),
                "Adobe XD".to_string(),
                "Canva".to_string(),
                "Affinity Designer".to_string(),
                "Affinity Photo".to_string(),
            ],
            AppCategory::Productivity => vec![
                "Notion".to_string(),
                "Todoist".to_string(),
                "Things".to_string(),
                "OmniFocus".to_string(),
                "TickTick".to_string(),
                "Asana".to_string(),
                "Trello".to_string(),
                "Linear".to_string(),
                "Jira".to_string(),
                "Calendar".to_string(),
            ],
            AppCategory::Browser => vec![
                "Google Chrome".to_string(),
                "Chrome".to_string(),
                "Safari".to_string(),
                "Firefox".to_string(),
                "Microsoft Edge".to_string(),
                "Edge".to_string(),
                "Arc".to_string(),
                "Brave Browser".to_string(),
                "Brave".to_string(),
                "Opera".to_string(),
            ],
            AppCategory::Terminal => vec![
                "Terminal".to_string(),
                "iTerm".to_string(),
                "iTerm2".to_string(),
                "Hyper".to_string(),
                "Alacritty".to_string(),
                "kitty".to_string(),
                "Warp".to_string(),
                "Windows Terminal".to_string(),
                "cmd".to_string(),
                "PowerShell".to_string(),
                "ConEmu".to_string(),
            ],
            AppCategory::Custom(name) => vec![name.clone()],
        }
    }
}

/// Parsed Focus Time configuration from a calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusTimeConfig {
    /// Whether this event should trigger Focus Time
    pub is_focus_time: bool,
    /// Apps that are allowed during this Focus Time (inverse blocklist)
    pub allowed_apps: Vec<String>,
    /// Categories that were used to generate the allowed apps
    pub allowed_categories: Vec<AppCategory>,
    /// Raw allowed apps specified directly (not from categories)
    pub raw_allowed_apps: Vec<String>,
    /// Source event title
    pub source_title: String,
}

impl Default for FocusTimeConfig {
    fn default() -> Self {
        Self {
            is_focus_time: false,
            allowed_apps: Vec::new(),
            allowed_categories: Vec::new(),
            raw_allowed_apps: Vec::new(),
            source_title: String::new(),
        }
    }
}

/// Parse a calendar event to detect Focus Time configuration
///
/// # Arguments
/// * `title` - The calendar event title
/// * `description` - Optional event description containing allowed apps
///
/// # Returns
/// A FocusTimeConfig with parsed settings
pub fn parse_focus_time_event(title: &str, description: Option<&str>) -> FocusTimeConfig {
    let mut config = FocusTimeConfig {
        source_title: title.to_string(),
        ..Default::default()
    };

    // Check if title contains focus time keywords
    let title_lower = title.to_lowercase();
    config.is_focus_time = FOCUS_TIME_KEYWORDS
        .iter()
        .any(|keyword| title_lower.contains(keyword));

    if !config.is_focus_time {
        return config;
    }

    // Parse allowed apps from description
    if let Some(desc) = description {
        let (categories, apps) = parse_allowed_apps_from_description(desc);
        config.allowed_categories = categories.clone();
        config.raw_allowed_apps = apps.clone();

        // Expand categories to apps
        let mut all_apps: HashSet<String> = HashSet::new();
        for category in &categories {
            for app in category.expand() {
                all_apps.insert(app);
            }
        }
        for app in &apps {
            all_apps.insert(app.clone());
        }

        config.allowed_apps = all_apps.into_iter().collect();
        config.allowed_apps.sort(); // For deterministic output
    }

    config
}

/// Parse allowed apps and categories from event description
///
/// Supported formats:
/// - `@coding` - Category reference
/// - `@coding, @terminal` - Multiple categories
/// - `vscode, terminal` - Direct app names
/// - `Allowed: @coding, notion` - Prefixed format
/// - Line-by-line format with "Apps:" header
///
/// # Arguments
/// * `description` - The event description text
///
/// # Returns
/// A tuple of (categories, direct_apps)
pub fn parse_allowed_apps_from_description(description: &str) -> (Vec<AppCategory>, Vec<String>) {
    let mut categories = Vec::new();
    let mut direct_apps = Vec::new();

    // Check if description is empty or whitespace only
    let desc = description.trim();
    if desc.is_empty() {
        return (categories, direct_apps);
    }

    // Try to find an "Allowed:" or "Apps:" line
    let content = if let Some(allowed_section) = extract_allowed_section(desc) {
        allowed_section
    } else {
        // Use the whole description
        desc.to_string()
    };

    // Split by common delimiters and process each item
    for item in content
        .split(|c: char| c == ',' || c == '\n' || c == ';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        // Skip common headers/labels
        if item.to_lowercase().starts_with("allowed:")
            || item.to_lowercase().starts_with("apps:")
            || item.to_lowercase().starts_with("focus:")
        {
            continue;
        }

        // Check if it's a category (starts with @)
        if item.starts_with('@') {
            if let Some(category) = AppCategory::from_str(item) {
                categories.push(category);
            }
        } else {
            // It's a direct app name
            direct_apps.push(item.to_string());
        }
    }

    (categories, direct_apps)
}

/// Extract the allowed apps section from a description
fn extract_allowed_section(description: &str) -> Option<String> {
    let desc_lower = description.to_lowercase();

    // Look for "Allowed:" or "Apps:" prefix
    for prefix in ["allowed:", "apps:", "allow:", "permitted:"] {
        if let Some(pos) = desc_lower.find(prefix) {
            let start = pos + prefix.len();
            // Find the end of this section (next blank line or end of string)
            let remaining = &description[start..];
            let end = remaining
                .find("\n\n")
                .unwrap_or(remaining.len());
            return Some(remaining[..end].trim().to_string());
        }
    }

    None
}

/// Normalize an app name for comparison
///
/// This handles:
/// - Case insensitivity
/// - .exe extension removal
/// - Common variations
pub fn normalize_app_name(name: &str) -> String {
    let mut normalized = name.trim().to_lowercase();

    // Remove .exe extension
    if normalized.ends_with(".exe") {
        normalized = normalized[..normalized.len() - 4].to_string();
    }

    // Remove .app extension (macOS)
    if normalized.ends_with(".app") {
        normalized = normalized[..normalized.len() - 4].to_string();
    }

    normalized
}

/// Check if a process name matches any allowed app (for inverse blocking)
///
/// # Arguments
/// * `process_name` - The running process name
/// * `allowed_apps` - List of allowed app names
///
/// # Returns
/// true if the process is in the allowed list
pub fn is_app_allowed(process_name: &str, allowed_apps: &[String]) -> bool {
    let normalized_process = normalize_app_name(process_name);

    allowed_apps.iter().any(|allowed| {
        let normalized_allowed = normalize_app_name(allowed);

        // Exact match
        if normalized_process == normalized_allowed {
            return true;
        }

        // Fuzzy match: check if either contains the other
        // This helps with variations like "Code" vs "Visual Studio Code"
        if normalized_process.contains(&normalized_allowed)
            || normalized_allowed.contains(&normalized_process)
        {
            return true;
        }

        false
    })
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_focus_time_detection_basic() {
        let config = parse_focus_time_event("Focus Time", None);
        assert!(config.is_focus_time);

        let config = parse_focus_time_event("Deep Work Session", None);
        assert!(config.is_focus_time);

        let config = parse_focus_time_event("Team Meeting", None);
        assert!(!config.is_focus_time);
    }

    #[test]
    fn test_focus_time_detection_case_insensitive() {
        assert!(parse_focus_time_event("FOCUS TIME", None).is_focus_time);
        assert!(parse_focus_time_event("Focus time", None).is_focus_time);
        assert!(parse_focus_time_event("focus TIME", None).is_focus_time);
        assert!(parse_focus_time_event("DeEp WoRk", None).is_focus_time);
    }

    #[test]
    fn test_focus_time_keywords_variations() {
        let test_cases = vec![
            ("Focus Block", true),
            ("Deep Work", true),
            ("Focus Session", true),
            ("No Meetings", true),
            ("Heads Down Time", true),
            ("Do Not Disturb", true),
            ("DND", true),
            ("Coding Time", true),
            ("Writing Time", true),
            ("Regular Meeting", false),
            ("Sprint Planning", false),
            ("Standup", false),
        ];

        for (title, expected) in test_cases {
            let config = parse_focus_time_event(title, None);
            assert_eq!(
                config.is_focus_time, expected,
                "Failed for title: {}",
                title
            );
        }
    }

    #[test]
    fn test_category_parsing() {
        assert_eq!(
            AppCategory::from_str("@coding"),
            Some(AppCategory::Coding)
        );
        assert_eq!(
            AppCategory::from_str("coding"),
            Some(AppCategory::Coding)
        );
        assert_eq!(
            AppCategory::from_str("@dev"),
            Some(AppCategory::Coding)
        );
        assert_eq!(
            AppCategory::from_str("@communication"),
            Some(AppCategory::Communication)
        );
        assert_eq!(
            AppCategory::from_str("@terminal"),
            Some(AppCategory::Terminal)
        );
    }

    #[test]
    fn test_category_expansion() {
        let coding_apps = AppCategory::Coding.expand();
        assert!(coding_apps.contains(&"Visual Studio Code".to_string()));
        assert!(coding_apps.contains(&"Xcode".to_string()));
        assert!(coding_apps.contains(&"Vim".to_string()));

        let terminal_apps = AppCategory::Terminal.expand();
        assert!(terminal_apps.contains(&"Terminal".to_string()));
        assert!(terminal_apps.contains(&"iTerm2".to_string()));
    }

    #[test]
    fn test_parse_allowed_apps_categories() {
        let (categories, apps) = parse_allowed_apps_from_description("@coding, @terminal");
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&AppCategory::Coding));
        assert!(categories.contains(&AppCategory::Terminal));
        assert!(apps.is_empty());
    }

    #[test]
    fn test_parse_allowed_apps_mixed() {
        let (categories, apps) =
            parse_allowed_apps_from_description("@coding, Notion, Obsidian");
        assert_eq!(categories.len(), 1);
        assert!(categories.contains(&AppCategory::Coding));
        assert_eq!(apps.len(), 2);
        assert!(apps.contains(&"Notion".to_string()));
        assert!(apps.contains(&"Obsidian".to_string()));
    }

    #[test]
    fn test_parse_allowed_apps_with_prefix() {
        let (categories, apps) =
            parse_allowed_apps_from_description("Allowed: @coding, Notion");
        assert!(categories.contains(&AppCategory::Coding));
        assert!(apps.contains(&"Notion".to_string()));
    }

    #[test]
    fn test_parse_allowed_apps_multiline() {
        let description = r#"
        Apps:
        @coding
        @terminal
        Notion
        "#;
        let (categories, apps) = parse_allowed_apps_from_description(description);
        assert!(categories.contains(&AppCategory::Coding));
        assert!(categories.contains(&AppCategory::Terminal));
        assert!(apps.contains(&"Notion".to_string()));
    }

    #[test]
    fn test_full_focus_time_parsing() {
        let config = parse_focus_time_event(
            "Deep Work - Coding",
            Some("Allowed: @coding, @terminal, Notion"),
        );

        assert!(config.is_focus_time);
        assert!(!config.allowed_apps.is_empty());
        assert!(config.allowed_apps.contains(&"Visual Studio Code".to_string()));
        assert!(config.allowed_apps.contains(&"Terminal".to_string()));
        assert!(config.allowed_apps.contains(&"Notion".to_string()));
    }

    #[test]
    fn test_empty_description() {
        let config = parse_focus_time_event("Focus Time", Some(""));
        assert!(config.is_focus_time);
        assert!(config.allowed_apps.is_empty());

        let config = parse_focus_time_event("Focus Time", Some("   \n  "));
        assert!(config.is_focus_time);
        assert!(config.allowed_apps.is_empty());
    }

    #[test]
    fn test_normalize_app_name() {
        assert_eq!(normalize_app_name("Chrome"), "chrome");
        assert_eq!(normalize_app_name("chrome.exe"), "chrome");
        assert_eq!(normalize_app_name("Chrome.exe"), "chrome");
        assert_eq!(normalize_app_name("Safari.app"), "safari");
        assert_eq!(normalize_app_name("  Visual Studio Code  "), "visual studio code");
    }

    #[test]
    fn test_is_app_allowed_exact() {
        let allowed = vec!["chrome".to_string(), "vscode".to_string()];
        assert!(is_app_allowed("chrome", &allowed));
        assert!(is_app_allowed("Chrome", &allowed));
        assert!(is_app_allowed("chrome.exe", &allowed));
        assert!(is_app_allowed("vscode", &allowed));
        assert!(!is_app_allowed("firefox", &allowed));
    }

    #[test]
    fn test_is_app_allowed_fuzzy() {
        let allowed = vec!["Code".to_string(), "Terminal".to_string()];
        assert!(is_app_allowed("Visual Studio Code", &allowed));
        assert!(is_app_allowed("code", &allowed));
        assert!(is_app_allowed("Code - Insiders", &allowed));
        assert!(is_app_allowed("Terminal", &allowed));
        assert!(!is_app_allowed("Slack", &allowed));
    }

    #[test]
    fn test_malformed_description_formats() {
        // Various malformed inputs should not crash
        let (categories, apps) = parse_allowed_apps_from_description("@@@");
        assert!(categories.is_empty());
        assert!(apps.is_empty() || apps.iter().all(|a| a.is_empty() || a == "@@@"));

        let (categories, apps) = parse_allowed_apps_from_description(",,,");
        assert!(categories.is_empty());
        assert!(apps.is_empty());

        let (categories, apps) = parse_allowed_apps_from_description("Allowed:");
        assert!(categories.is_empty());
        assert!(apps.is_empty());
    }
}
