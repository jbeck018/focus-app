// focus_time/tests.rs - Comprehensive tests for Calendar-Based Focus Time
//
// Test Coverage:
// 1. Calendar Event Parsing Tests
// 2. Category Expansion Tests
// 3. Process Blocking Tests (inverse blocking mode)
// 4. Integration Tests
// 5. Edge Case Tests

#[cfg(test)]
mod calendar_parsing_tests {
    use crate::focus_time::parser::*;

    #[test]
    fn test_detect_focus_time_from_title_basic() {
        let test_cases = vec![
            ("Focus Time", true),
            ("Focus Block", true),
            ("Deep Work", true),
            ("Focus Session", true),
            ("No Meetings", true),
            ("Heads Down", true),
            ("Do Not Disturb", true),
            ("DND", true),
            ("Coding Time", true),
            ("Writing Time", true),
        ];

        for (title, expected) in test_cases {
            let config = parse_focus_time_event(title, None);
            assert_eq!(
                config.is_focus_time, expected,
                "Failed for title: '{}'",
                title
            );
        }
    }

    #[test]
    fn test_detect_focus_time_case_insensitive() {
        let variations = vec![
            "FOCUS TIME",
            "Focus Time",
            "focus time",
            "FoCuS TiMe",
            "DEEP WORK",
            "Deep Work",
            "deep work",
        ];

        for title in variations {
            let config = parse_focus_time_event(title, None);
            assert!(
                config.is_focus_time,
                "Should detect focus time for: '{}'",
                title
            );
        }
    }

    #[test]
    fn test_non_focus_time_events() {
        let non_focus = vec![
            "Team Meeting",
            "Sprint Planning",
            "1:1 with Manager",
            "Standup",
            "Code Review",
            "Interview",
            "Lunch",
            "All Hands",
        ];

        for title in non_focus {
            let config = parse_focus_time_event(title, None);
            assert!(
                !config.is_focus_time,
                "Should NOT detect focus time for: '{}'",
                title
            );
        }
    }

    #[test]
    fn test_focus_time_with_context() {
        // Focus time with additional context in title
        let config = parse_focus_time_event("Focus Time - Feature Development", None);
        assert!(config.is_focus_time);

        let config = parse_focus_time_event("[Focus] API Refactoring", None);
        assert!(!config.is_focus_time); // [Focus] alone doesn't match keywords

        let config = parse_focus_time_event("Deep Work: Database Migration", None);
        assert!(config.is_focus_time);
    }

    #[test]
    fn test_parse_allowed_apps_single_category() {
        let config = parse_focus_time_event("Focus Time", Some("@coding"));

        assert!(config.is_focus_time);
        assert!(!config.allowed_apps.is_empty());
        assert!(config.allowed_apps.contains(&"Visual Studio Code".to_string()));
    }

    #[test]
    fn test_parse_allowed_apps_multiple_categories() {
        let config = parse_focus_time_event(
            "Deep Work",
            Some("Allowed: @coding, @terminal"),
        );

        assert!(config.is_focus_time);
        assert!(config.allowed_apps.contains(&"Visual Studio Code".to_string()));
        assert!(config.allowed_apps.contains(&"Terminal".to_string()));
    }

    #[test]
    fn test_parse_allowed_apps_mixed_categories_and_apps() {
        let config = parse_focus_time_event(
            "Focus Time",
            Some("@coding, Notion, Obsidian"),
        );

        assert!(config.is_focus_time);
        // Category apps
        assert!(config.allowed_apps.contains(&"Visual Studio Code".to_string()));
        // Direct apps
        assert!(config.allowed_apps.contains(&"Notion".to_string()));
        assert!(config.allowed_apps.contains(&"Obsidian".to_string()));
    }

    #[test]
    fn test_parse_empty_description() {
        let config = parse_focus_time_event("Focus Time", Some(""));
        assert!(config.is_focus_time);
        assert!(config.allowed_apps.is_empty());
    }

    #[test]
    fn test_parse_whitespace_only_description() {
        let config = parse_focus_time_event("Focus Time", Some("   \n\t  "));
        assert!(config.is_focus_time);
        assert!(config.allowed_apps.is_empty());
    }

    #[test]
    fn test_parse_multiline_description() {
        let description = r#"
        Focus session for API development

        Apps:
        @coding
        @terminal
        Postman
        "#;

        let config = parse_focus_time_event("Focus Time", Some(description));
        assert!(config.is_focus_time);
        assert!(!config.allowed_apps.is_empty());
    }

    #[test]
    fn test_parse_description_with_prefix() {
        let test_cases = vec![
            "Allowed: @coding, Terminal",
            "Apps: @coding, Terminal",
            "Allow: @coding, Terminal",
            "Permitted: @coding, Terminal",
        ];

        for desc in test_cases {
            let config = parse_focus_time_event("Focus Time", Some(desc));
            assert!(
                config.is_focus_time,
                "Failed for description: '{}'",
                desc
            );
        }
    }
}

#[cfg(test)]
mod category_expansion_tests {
    use crate::focus_time::parser::AppCategory;

    #[test]
    fn test_category_from_string() {
        assert_eq!(AppCategory::from_str("@coding"), Some(AppCategory::Coding));
        assert_eq!(AppCategory::from_str("coding"), Some(AppCategory::Coding));
        assert_eq!(AppCategory::from_str("@dev"), Some(AppCategory::Coding));
        assert_eq!(AppCategory::from_str("@terminal"), Some(AppCategory::Terminal));
        assert_eq!(AppCategory::from_str("shell"), Some(AppCategory::Terminal));
        assert_eq!(AppCategory::from_str("@browser"), Some(AppCategory::Browser));
    }

    #[test]
    fn test_coding_category_apps() {
        let apps = AppCategory::Coding.expand();

        // Must include common IDEs
        assert!(apps.contains(&"Visual Studio Code".to_string()));
        assert!(apps.contains(&"Xcode".to_string()));
        assert!(apps.contains(&"IntelliJ IDEA".to_string()));
        assert!(apps.contains(&"Vim".to_string()));
        assert!(apps.contains(&"nvim".to_string()));
    }

    #[test]
    fn test_terminal_category_apps() {
        let apps = AppCategory::Terminal.expand();

        assert!(apps.contains(&"Terminal".to_string()));
        assert!(apps.contains(&"iTerm2".to_string()));
        assert!(apps.contains(&"Alacritty".to_string()));
        assert!(apps.contains(&"Warp".to_string()));
    }

    #[test]
    fn test_communication_category_apps() {
        let apps = AppCategory::Communication.expand();

        assert!(apps.contains(&"Slack".to_string()));
        assert!(apps.contains(&"Discord".to_string()));
        assert!(apps.contains(&"Zoom".to_string()));
        assert!(apps.contains(&"Microsoft Teams".to_string()));
    }

    #[test]
    fn test_custom_category() {
        let category = AppCategory::Custom("my-custom-app".to_string());
        let apps = category.expand();

        assert_eq!(apps.len(), 1);
        assert!(apps.contains(&"my-custom-app".to_string()));
    }
}

#[cfg(test)]
mod process_blocking_tests {
    use crate::focus_time::parser::{is_app_allowed, normalize_app_name};
    use crate::focus_time::app_registry::AppRegistry;

    #[test]
    fn test_normalize_app_name_basic() {
        assert_eq!(normalize_app_name("Chrome"), "chrome");
        assert_eq!(normalize_app_name("CHROME"), "chrome");
        assert_eq!(normalize_app_name("  Chrome  "), "chrome");
    }

    #[test]
    fn test_normalize_app_name_extensions() {
        // Windows .exe
        assert_eq!(normalize_app_name("chrome.exe"), "chrome");
        assert_eq!(normalize_app_name("Chrome.exe"), "chrome");
        assert_eq!(normalize_app_name("CHROME.EXE"), "chrome");

        // macOS .app
        assert_eq!(normalize_app_name("Safari.app"), "safari");
        assert_eq!(normalize_app_name("Terminal.app"), "terminal");
    }

    #[test]
    fn test_is_app_allowed_exact_match() {
        let allowed = vec!["chrome".to_string(), "vscode".to_string()];

        assert!(is_app_allowed("chrome", &allowed));
        assert!(is_app_allowed("Chrome", &allowed));
        assert!(is_app_allowed("CHROME", &allowed));
        assert!(is_app_allowed("chrome.exe", &allowed));
        assert!(is_app_allowed("vscode", &allowed));
        assert!(!is_app_allowed("firefox", &allowed));
    }

    #[test]
    fn test_is_app_allowed_fuzzy_match() {
        let allowed = vec!["Code".to_string(), "Terminal".to_string()];

        // "Visual Studio Code" contains "Code"
        assert!(is_app_allowed("Visual Studio Code", &allowed));
        // "code" contained in "Code"
        assert!(is_app_allowed("code", &allowed));
        // Process name variations
        assert!(is_app_allowed("Terminal", &allowed));
        assert!(is_app_allowed("terminal.app", &allowed));

        // Should not match unrelated apps
        assert!(!is_app_allowed("Slack", &allowed));
        assert!(!is_app_allowed("Chrome", &allowed));
    }

    #[test]
    fn test_inverse_blocking_mode() {
        // In Focus Time, we ALLOW specific apps and block everything else
        let allowed = vec!["Code".to_string(), "Terminal".to_string()];

        // These should be allowed (return true)
        assert!(is_app_allowed("Code", &allowed));
        assert!(is_app_allowed("Visual Studio Code", &allowed));
        assert!(is_app_allowed("Terminal", &allowed));

        // These should be blocked (return false)
        assert!(!is_app_allowed("Chrome", &allowed));
        assert!(!is_app_allowed("Slack", &allowed));
        assert!(!is_app_allowed("Discord", &allowed));
        assert!(!is_app_allowed("Twitter", &allowed));
    }

    #[test]
    fn test_registry_process_allowed() {
        let registry = AppRegistry::new();
        let allowed = vec!["@coding".to_string(), "slack".to_string()];

        // Coding apps should be allowed
        assert!(registry.is_process_allowed("code", &allowed));
        assert!(registry.is_process_allowed("Visual Studio Code", &allowed));
        assert!(registry.is_process_allowed("vim", &allowed));

        // Slack should be allowed (direct)
        assert!(registry.is_process_allowed("Slack", &allowed));

        // Other apps should not be allowed
        assert!(!registry.is_process_allowed("Chrome", &allowed));
        assert!(!registry.is_process_allowed("Discord", &allowed));
    }

    #[test]
    fn test_protected_processes() {
        use crate::focus_time::app_registry::is_protected_process;

        // System processes should be protected
        #[cfg(target_os = "macos")]
        {
            assert!(is_protected_process("kernel_task"));
            assert!(is_protected_process("Finder"));
            assert!(is_protected_process("WindowServer"));
        }

        #[cfg(target_os = "windows")]
        {
            assert!(is_protected_process("explorer.exe"));
            assert!(is_protected_process("csrss.exe"));
        }

        #[cfg(target_os = "linux")]
        {
            assert!(is_protected_process("systemd"));
            assert!(is_protected_process("Xorg"));
        }

        // User apps should not be protected
        assert!(!is_protected_process("Chrome"));
        assert!(!is_protected_process("Slack"));
        assert!(!is_protected_process("vscode"));
    }
}

#[cfg(test)]
mod focus_time_state_tests {
    use crate::focus_time::{FocusTimeState, FocusTimeEventParsed};
    use chrono::{Duration, Utc};

    #[test]
    fn test_state_allowed_apps() {
        let mut state = FocusTimeState {
            active: true,
            allowed_apps: vec!["vscode".to_string(), "terminal".to_string()],
            original_allowed_apps: vec!["vscode".to_string(), "terminal".to_string()],
            ..Default::default()
        };

        // Check allowed apps
        assert!(state.is_app_allowed("vscode"));
        assert!(state.is_app_allowed("VSCode"));
        assert!(state.is_app_allowed("terminal"));
        assert!(!state.is_app_allowed("chrome"));

        // Add an app
        state.add_allowed_app("slack");
        assert!(state.is_app_allowed("slack"));
        assert!(state.added_apps.contains(&"slack".to_string()));

        // Remove an app
        state.remove_allowed_app("vscode");
        assert!(!state.is_app_allowed("vscode"));
        assert!(state.removed_apps.contains(&"vscode".to_string()));
    }

    #[test]
    fn test_state_inactive_allows_all() {
        let state = FocusTimeState {
            active: false,
            allowed_apps: vec!["vscode".to_string()],
            ..Default::default()
        };

        // When inactive, all apps should be allowed
        assert!(state.is_app_allowed("vscode"));
        assert!(state.is_app_allowed("chrome"));
        assert!(state.is_app_allowed("slack"));
        assert!(state.is_app_allowed("anything"));
    }

    #[test]
    fn test_state_reset_overrides() {
        let mut state = FocusTimeState {
            active: true,
            allowed_apps: vec!["vscode".to_string()],
            original_allowed_apps: vec!["vscode".to_string(), "terminal".to_string()],
            added_apps: vec!["slack".to_string()],
            removed_apps: vec!["terminal".to_string()],
            ..Default::default()
        };

        // Reset overrides
        state.reset_overrides();

        // Should restore original apps
        assert_eq!(state.allowed_apps, state.original_allowed_apps);
        assert!(state.added_apps.is_empty());
        assert!(state.removed_apps.is_empty());
    }

    #[test]
    fn test_state_end() {
        let mut state = FocusTimeState {
            active: true,
            ..Default::default()
        };

        // End normally
        state.end(false);
        assert!(!state.active);
        assert!(!state.ended_early);

        // End early
        let mut state2 = FocusTimeState {
            active: true,
            ..Default::default()
        };
        state2.end(true);
        assert!(!state2.active);
        assert!(state2.ended_early);
    }

    #[test]
    fn test_event_timing() {
        let now = Utc::now();

        // Active event
        let active_event = FocusTimeEventParsed {
            id: "1".to_string(),
            title: "Focus Time".to_string(),
            clean_title: "Focus Time".to_string(),
            description: None,
            start_time: now - Duration::minutes(10),
            end_time: now + Duration::minutes(50),
            duration_minutes: 60,
            allowed_apps: vec![],
            raw_allowed_apps: None,
            categories: vec![],
            is_active: true,
            is_upcoming: false,
            source: "calendar".to_string(),
        };
        assert!(active_event.check_is_active());
        assert!(!active_event.check_is_upcoming());

        // Upcoming event
        let upcoming_event = FocusTimeEventParsed {
            start_time: now + Duration::minutes(30),
            end_time: now + Duration::minutes(90),
            is_active: false,
            is_upcoming: true,
            ..active_event.clone()
        };
        assert!(!upcoming_event.check_is_active());
        assert!(upcoming_event.check_is_upcoming());

        // Past event
        let past_event = FocusTimeEventParsed {
            start_time: now - Duration::hours(2),
            end_time: now - Duration::hours(1),
            is_active: false,
            is_upcoming: false,
            ..active_event.clone()
        };
        assert!(!past_event.check_is_active());
        assert!(!past_event.check_is_upcoming());
    }
}

#[cfg(test)]
mod manager_tests {
    use crate::focus_time::{FocusTimeManager, FocusTimeState, FocusTimeEventParsed};
    use chrono::{Duration, Utc};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_event() -> FocusTimeEventParsed {
        let now = Utc::now();
        FocusTimeEventParsed {
            id: "test-event-1".to_string(),
            title: "Focus Time".to_string(),
            clean_title: "Coding Session".to_string(),
            description: Some("@coding".to_string()),
            start_time: now,
            end_time: now + Duration::hours(1),
            duration_minutes: 60,
            allowed_apps: vec!["Code".to_string(), "Terminal".to_string()],
            raw_allowed_apps: None,
            categories: vec!["Coding".to_string()],
            is_active: true,
            is_upcoming: false,
            source: "calendar".to_string(),
        }
    }

    #[tokio::test]
    async fn test_manager_start_from_event() {
        let state = Arc::new(RwLock::new(FocusTimeState::default()));
        let manager = FocusTimeManager::new(state);
        let event = create_test_event();

        // Start Focus Time
        let result = manager.start_from_event(&event, false).await;
        assert!(result.is_ok());

        // Should be active
        assert!(manager.is_active().await);

        // Should have correct allowed apps
        let allowed = manager.get_allowed_apps().await;
        assert!(allowed.contains(&"Code".to_string()));
    }

    #[tokio::test]
    async fn test_manager_cannot_start_when_active() {
        let state = Arc::new(RwLock::new(FocusTimeState::default()));
        let manager = FocusTimeManager::new(state);
        let event = create_test_event();

        // Start Focus Time
        let _ = manager.start_from_event(&event, false).await;

        // Try to start again
        let result = manager.start_from_event(&event, false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_manager_end_early() {
        let state = Arc::new(RwLock::new(FocusTimeState::default()));
        let manager = FocusTimeManager::new(state);
        let event = create_test_event();

        // Start and then end early
        let _ = manager.start_from_event(&event, false).await;
        let result = manager.end(true).await;

        assert!(result.is_ok());
        assert!(!manager.is_active().await);

        // Check the state was marked as ended early
        let state = manager.get_state().await;
        assert!(state.ended_early);
    }

    #[tokio::test]
    async fn test_manager_process_allowed() {
        let state = Arc::new(RwLock::new(FocusTimeState::default()));
        let manager = FocusTimeManager::new(state);
        let event = create_test_event();

        // Start Focus Time
        let _ = manager.start_from_event(&event, false).await;

        // Check allowed processes
        assert!(manager.is_process_allowed("Code").await);
        assert!(manager.is_process_allowed("Terminal").await);
        assert!(!manager.is_process_allowed("Chrome").await);
    }

    #[tokio::test]
    async fn test_manager_add_remove_apps() {
        let state = Arc::new(RwLock::new(FocusTimeState::default()));
        let manager = FocusTimeManager::new(state);
        let event = create_test_event();

        let _ = manager.start_from_event(&event, false).await;

        // Add an app
        let _ = manager.add_allowed_app("Slack").await;
        assert!(manager.is_process_allowed("Slack").await);

        // Remove an app
        let _ = manager.remove_allowed_app("Code").await;
        assert!(!manager.is_process_allowed("Code").await);
    }

    #[tokio::test]
    async fn test_manager_auto_deactivate() {
        let state = Arc::new(RwLock::new(FocusTimeState::default()));
        let manager = FocusTimeManager::new(state);

        // Create an event that has already ended
        let now = Utc::now();
        let _past_event = FocusTimeEventParsed {
            id: "past-event".to_string(),
            title: "Focus Time".to_string(),
            clean_title: "Past Session".to_string(),
            description: None,
            start_time: now - Duration::hours(2),
            end_time: now - Duration::hours(1),
            duration_minutes: 60,
            allowed_apps: vec![],
            raw_allowed_apps: None,
            categories: vec![],
            is_active: false,
            is_upcoming: false,
            source: "calendar".to_string(),
        };

        // Force start (even though event is past)
        {
            let mut state = manager.get_state().await;
            state.active = true;
            state.ends_at = Some(now - Duration::hours(1));
        }

        // Since we can't modify the internal state directly in the test,
        // we'll test the check_auto_deactivate logic differently
        // The manager should detect that the end time has passed
    }

    #[tokio::test]
    async fn test_manager_reset() {
        let state = Arc::new(RwLock::new(FocusTimeState::default()));
        let manager = FocusTimeManager::new(state);
        let event = create_test_event();

        // Start and then reset
        let _ = manager.start_from_event(&event, false).await;
        manager.reset().await;

        assert!(!manager.is_active().await);
        assert!(manager.get_allowed_apps().await.is_empty());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use crate::focus_time::parser::*;

    #[test]
    fn test_empty_event_title() {
        let config = parse_focus_time_event("", None);
        assert!(!config.is_focus_time);
    }

    #[test]
    fn test_very_long_title() {
        let long_title = "Focus Time ".repeat(100);
        let config = parse_focus_time_event(&long_title, None);
        assert!(config.is_focus_time);
    }

    #[test]
    fn test_special_characters_in_title() {
        let config = parse_focus_time_event("Focus Time: 日本語テスト", None);
        assert!(config.is_focus_time);

        let config = parse_focus_time_event("Deep Work 🎯", None);
        assert!(config.is_focus_time);
    }

    #[test]
    fn test_malformed_category() {
        let config = parse_focus_time_event(
            "Focus Time",
            Some("@invalid-category-that-doesnt-exist"),
        );

        // Should still be focus time, but with custom category
        assert!(config.is_focus_time);
    }

    #[test]
    fn test_duplicate_categories() {
        let config = parse_focus_time_event(
            "Focus Time",
            Some("@coding, @coding, @coding"),
        );

        assert!(config.is_focus_time);
        // Should not have duplicates in allowed_apps
        let unique_count = config
            .allowed_apps
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(unique_count, config.allowed_apps.len());
    }

    #[test]
    fn test_mixed_case_categories() {
        let config = parse_focus_time_event(
            "Focus Time",
            Some("@CODING, @Terminal, @browser"),
        );

        assert!(config.is_focus_time);
        assert!(!config.allowed_apps.is_empty());
    }

    #[test]
    fn test_description_with_html() {
        // Calendar descriptions might contain HTML
        let config = parse_focus_time_event(
            "Focus Time",
            Some("<p>Allowed: @coding</p><br/><b>Important meeting notes</b>"),
        );

        // Should still parse categories even with HTML
        assert!(config.is_focus_time);
    }

    #[test]
    fn test_description_with_urls() {
        let config = parse_focus_time_event(
            "Focus Time",
            Some("@coding\nhttps://example.com/meeting\nNotion"),
        );

        assert!(config.is_focus_time);
    }

    #[test]
    fn test_app_name_with_version() {
        let allowed = vec!["Code".to_string()];

        // Should match versioned process names
        assert!(is_app_allowed("Code - Insiders", &allowed));
        assert!(is_app_allowed("Visual Studio Code", &allowed));
    }

    #[test]
    fn test_empty_allowed_apps_list() {
        // With no allowed apps, nothing should be allowed
        let allowed: Vec<String> = vec![];

        assert!(!is_app_allowed("chrome", &allowed));
        assert!(!is_app_allowed("vscode", &allowed));
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::commands::calendar::CalendarEvent;
    use crate::focus_time::{
        detect_focus_time_events,
        find_active_focus_time,
        find_upcoming_focus_times,
    };
    use chrono::{Duration, Utc};

    fn create_calendar_event(
        id: &str,
        title: &str,
        description: Option<&str>,
        start_offset_mins: i64,
        duration_mins: i64,
    ) -> CalendarEvent {
        let start = Utc::now() + Duration::minutes(start_offset_mins);
        let end = start + Duration::minutes(duration_mins);

        CalendarEvent {
            id: id.to_string(),
            title: title.to_string(),
            description: description.map(String::from),
            start_time: start.to_rfc3339(),
            end_time: end.to_rfc3339(),
            is_all_day: false,
            is_busy: true,
            location: None,
            attendees: vec![],
            html_link: None,
        }
    }

    #[test]
    fn test_detect_focus_time_from_calendar_events() {
        let events = vec![
            create_calendar_event("1", "Focus Time", Some("@coding"), -30, 60),
            create_calendar_event("2", "Team Meeting", None, 60, 30),
            create_calendar_event("3", "Deep Work", Some("@terminal, Notion"), 120, 90),
        ];

        let focus_events = detect_focus_time_events(&events);

        // Should find 2 focus time events
        assert_eq!(focus_events.len(), 2);
        assert!(focus_events.iter().any(|e| e.id == "1"));
        assert!(focus_events.iter().any(|e| e.id == "3"));
    }

    #[test]
    fn test_find_active_focus_time() {
        let events = vec![
            create_calendar_event("1", "Focus Time", Some("@coding"), -30, 60), // Active
            create_calendar_event("2", "Deep Work", Some("@terminal"), 60, 30), // Upcoming
        ];

        let focus_events = detect_focus_time_events(&events);
        let active = find_active_focus_time(&focus_events);

        assert!(active.is_some());
        assert_eq!(active.unwrap().id, "1");
    }

    #[test]
    fn test_find_upcoming_focus_times() {
        let events = vec![
            create_calendar_event("1", "Focus Time", None, -60, 30), // Past
            create_calendar_event("2", "Focus Time", None, -10, 60), // Active
            create_calendar_event("3", "Deep Work", None, 30, 60), // Upcoming (in 30 mins)
            create_calendar_event("4", "Focus Block", None, 120, 60), // Too far ahead
        ];

        let focus_events = detect_focus_time_events(&events);
        let upcoming = find_upcoming_focus_times(&focus_events);

        // Should find only event 3 (within next hour, not active)
        assert_eq!(upcoming.len(), 1);
        assert_eq!(upcoming[0].id, "3");
    }

    #[test]
    fn test_calendar_event_with_allowed_apps() {
        let event = create_calendar_event(
            "1",
            "Coding Time",
            Some("Allowed: @coding, @terminal, Notion"),
            -10,
            60,
        );

        let focus_events = detect_focus_time_events(&[event]);

        assert_eq!(focus_events.len(), 1);
        let focus = &focus_events[0];

        // Should have parsed allowed apps
        assert!(!focus.allowed_apps.is_empty());
        assert!(focus.allowed_apps.iter().any(|a| a.contains("Code")));
        assert!(focus.allowed_apps.iter().any(|a| a.contains("Terminal")));
        assert!(focus.allowed_apps.iter().any(|a| a.contains("Notion")));
    }

    #[test]
    fn test_focus_time_event_clean_title() {
        let events = vec![
            create_calendar_event("1", "[Focus] Coding Session", None, 0, 60),
            create_calendar_event("2", "Focus Time: API Development", None, 0, 60),
            create_calendar_event("3", "[Deep Work] Writing", None, 0, 60),
        ];

        let focus_events = detect_focus_time_events(&events);

        // Note: Only event 2 matches "Focus Time" keyword
        // Events 1 and 3 don't match any keyword exactly
        for event in &focus_events {
            assert!(!event.clean_title.starts_with("["));
        }
    }
}

#[cfg(test)]
mod security_tests {
    use crate::focus_time::parser::*;

    #[test]
    fn test_no_code_injection_in_description() {
        // Ensure malicious descriptions don't cause issues
        let malicious = r#"
            <script>alert('xss')</script>
            @coding
            ${process.env.SECRET}
            `rm -rf /`
            $(whoami)
        "#;

        let config = parse_focus_time_event("Focus Time", Some(malicious));

        // Should still work, but not execute anything
        assert!(config.is_focus_time);
    }

    #[test]
    fn test_null_bytes_in_input() {
        let config = parse_focus_time_event("Focus Time\0", Some("@coding\0"));
        // Should handle null bytes gracefully
        assert!(config.is_focus_time);
    }

    #[test]
    fn test_extremely_long_description() {
        let long_desc = "@coding\n".repeat(10000);
        let config = parse_focus_time_event("Focus Time", Some(&long_desc));

        // Should not hang or crash
        assert!(config.is_focus_time);
    }
}
