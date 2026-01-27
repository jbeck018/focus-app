// lib.rs - Main library entry point

pub mod ai;
mod commands;
mod db;
mod blocking;
mod system;
mod error;
mod state;
mod oauth;
mod trailbase;

use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

pub use error::{Error, Result};
pub use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Logging plugin - writes to both console and rotating log files
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .max_file_size(5_000_000) // 5MB max per file
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        // Autostart plugin - MacosLauncher parameter is cross-platform safe
        // (ignored on Windows/Linux via conditional compilation)
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            let handle = app.handle();

            // Initialize encryption subsystem
            db::crypto::init_encryption()?;

            // Initialize application state
            let state = tauri::async_runtime::block_on(async {
                AppState::new(handle.clone()).await
            })?;

            // Setup system tray
            system::tray::setup_tray(app)?;

            // Initialize sync queue
            let sync_state = state.clone();
            tauri::async_runtime::block_on(async {
                let sync_queue = trailbase::sync::SyncQueue::new(sync_state.pool().clone());
                if let Err(e) = sync_queue.init().await {
                    tracing::warn!("Failed to initialize sync queue: {}", e);
                }
            });

            // Restore auth state from stored tokens
            let auth_state = state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = commands::auth::restore_auth_state(&auth_state).await {
                    tracing::warn!("Failed to restore auth state: {}", e);
                }
            });

            // Start background monitoring task
            let monitor_state = state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = blocking::process::start_monitoring_loop(monitor_state).await {
                    tracing::error!("Process monitoring error: {}", e);
                }
            });

            // Manage state
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Focus session commands
            commands::focus::start_focus_session,
            commands::focus::end_focus_session,
            commands::focus::get_active_session,
            commands::focus::get_session_history,
            commands::focus::toggle_session_pause,
            commands::focus::extend_session,
            commands::focus::get_todays_session_count,

            // Timer commands (backend-owned timer for cross-window sync)
            commands::timer::toggle_timer_pause,
            commands::timer::get_timer_state,

            // Blocking commands
            commands::blocking::add_blocked_app,
            commands::blocking::remove_blocked_app,
            commands::blocking::add_blocked_website,
            commands::blocking::remove_blocked_website,
            commands::blocking::get_blocked_items,
            commands::blocking::toggle_blocking,

            // DNS Fallback commands (frontend-based blocking)
            commands::blocking::get_blocked_domains,
            commands::blocking::check_domain_blocked,
            commands::blocking::check_url_blocked,
            commands::blocking::get_blocking_stats,

            // Capability detection and permission guidance
            commands::blocking::get_blocking_capabilities,
            commands::blocking::get_elevation_instructions,
            commands::blocking::check_hosts_file_permissions,

            // Comprehensive permission checking
            commands::permissions::check_permissions,
            commands::permissions::get_permission_instructions,

            // Advanced blocking commands
            commands::blocking_advanced::create_blocking_schedule,
            commands::blocking_advanced::get_blocking_schedules,
            commands::blocking_advanced::update_blocking_schedule,
            commands::blocking_advanced::delete_blocking_schedule,
            commands::blocking_advanced::get_blocking_categories,
            commands::blocking_advanced::create_blocking_category,
            commands::blocking_advanced::update_blocking_category,
            commands::blocking_advanced::toggle_blocking_category,
            commands::blocking_advanced::enable_strict_mode,
            commands::blocking_advanced::disable_strict_mode,
            commands::blocking_advanced::get_strict_mode_state,
            commands::blocking_advanced::activate_nuclear_option,
            commands::blocking_advanced::get_nuclear_option_state,
            commands::blocking_advanced::record_block_attempt,
            commands::blocking_advanced::get_block_statistics,
            commands::blocking_advanced::get_session_blocks,

            // Analytics commands
            commands::analytics::get_daily_stats,
            commands::analytics::get_weekly_stats,
            commands::analytics::get_date_range_stats,
            commands::analytics::get_productivity_score,

            // Auth commands
            commands::auth::login,
            commands::auth::register,
            commands::auth::logout,
            commands::auth::get_auth_state,
            commands::auth::refresh_token,
            commands::auth::set_trailbase_url,
            commands::auth::is_authenticated,
            commands::auth::get_current_user,
            commands::auth::dev_set_subscription_tier,
            commands::auth::start_google_oauth,
            commands::auth::complete_google_oauth,

            // Journal commands
            commands::journal::create_journal_entry,
            commands::journal::get_session_journal_entries,
            commands::journal::get_recent_journal_entries,
            commands::journal::get_trigger_insights,
            commands::journal::get_peak_distraction_times,

            // Sync commands
            commands::sync::export_data,
            commands::sync::import_data,

            // Calendar commands
            commands::calendar::get_calendar_connections,
            commands::calendar::get_oauth_config_status,
            commands::calendar::start_calendar_oauth,
            commands::calendar::complete_calendar_oauth,
            commands::calendar::disconnect_calendar,
            commands::calendar::get_calendar_events,
            commands::calendar::get_focus_suggestions,
            commands::calendar::get_meeting_load,

            // AI Coach commands
            commands::coach::get_coach_response,
            commands::coach::send_coach_message,
            commands::coach::get_or_create_active_conversation,
            commands::coach::get_daily_tip,
            commands::coach::get_session_advice,
            commands::coach::get_reflection_prompt,
            commands::coach::analyze_patterns,

            // Chat History and Memory commands
            commands::chat_history::create_conversation,
            commands::chat_history::list_conversations,
            commands::chat_history::get_conversation,
            commands::chat_history::delete_conversation,
            commands::chat_history::archive_conversation,
            commands::chat_history::add_message,
            commands::chat_history::get_recent_messages,
            commands::chat_history::get_messages_paginated,
            commands::chat_history::save_memory,
            commands::chat_history::get_memories,
            commands::chat_history::get_relevant_memories,
            commands::chat_history::delete_memory,
            commands::chat_history::auto_archive_old_conversations,
            commands::chat_history::cleanup_expired_memories,
            commands::chat_history::build_conversation_context,
            commands::chat_history::update_conversation_title,
            commands::chat_history::update_conversation_summary,
            commands::chat_history::clear_conversation_messages,
            commands::chat_history::search_messages,
            commands::chat_history::export_chat_history,
            commands::chat_history::link_conversation_to_session,
            commands::chat_history::get_session_conversations,
            commands::chat_history::get_conversation_count,

            // AI/LLM management commands
            commands::ai::get_available_models,
            commands::ai::load_model,
            commands::ai::unload_model,
            commands::ai::download_model,
            commands::ai::is_model_downloaded,
            commands::ai::delete_model,
            commands::ai::toggle_ai_coach,
            commands::ai::get_models_cache_size,
            commands::ai::clear_models_cache,

            // LLM status and health commands
            commands::llm::get_llm_status,
            commands::llm::refresh_llm_status,
            commands::llm::check_llm_connection,
            commands::llm::get_model_details,
            commands::llm::clear_llm_cache,

            // Multi-provider LLM commands
            commands::ai_providers::list_providers,
            commands::ai_providers::list_models,
            commands::ai_providers::set_active_provider,
            commands::ai_providers::get_active_provider,
            commands::ai_providers::test_provider_connection,
            commands::ai_providers::stream_chat,
            commands::ai_providers::complete_chat,
            commands::ai_providers::is_local_ai_available,

            // Credential management commands
            commands::credentials::save_api_key,
            commands::credentials::get_api_key,
            commands::credentials::delete_api_key,
            commands::credentials::has_api_key,
            commands::credentials::list_saved_providers,

            // Team commands (local storage with sync queue)
            commands::team::get_current_team,
            commands::team::create_team,
            commands::team::join_team,
            commands::team::leave_team,
            commands::team::get_team_members,
            commands::team::get_team_stats,
            commands::team::get_team_blocklist,
            commands::team::add_team_blocked_item,
            commands::team::remove_team_blocked_item,
            commands::team::get_team_privacy_settings,
            commands::team::update_team_privacy_settings,
            commands::team::sync_team_blocklist,

            // Team sync commands (TrailBase integration)
            commands::team_sync::connect_team,
            commands::team_sync::disconnect_team,
            commands::team_sync::get_team_members_sync,
            commands::team_sync::share_session,
            commands::team_sync::get_team_activity,
            commands::team_sync::sync_with_team,
            commands::team_sync::get_sync_status,
            commands::team_sync::retry_failed_syncs,
            commands::team_sync::clear_failed_syncs,

            // Window management commands
            commands::window::open_mini_timer,
            commands::window::close_mini_timer,
            commands::window::toggle_mini_timer,
            commands::window::set_mini_timer_position,
            commands::window::get_mini_timer_position,
            commands::window::focus_main_window,
            commands::window::emit_to_mini_timer,

            // Streak commands
            commands::streaks::get_current_streak,
            commands::streaks::get_streak_heatmap,
            commands::streaks::get_streak_stats,
            commands::streaks::get_streak_milestones,
            commands::streaks::get_available_freezes,
            commands::streaks::use_streak_freeze,
            commands::streaks::update_streak_history,
            commands::streaks::create_weekly_freeze,

            // Achievement commands
            commands::achievements::get_achievements,
            commands::achievements::get_achievement_stats,
            commands::achievements::get_recent_achievements,
            commands::achievements::check_achievements,

            // Onboarding commands
            commands::onboarding::complete_onboarding,
            commands::onboarding::is_onboarding_complete,
            commands::onboarding::get_onboarding_data,
            commands::onboarding::reset_onboarding,

            // Screen dimming commands
            commands::dimming::enable_screen_dimming,
            commands::dimming::disable_screen_dimming,
            commands::dimming::get_dimming_state,
            commands::dimming::set_dimming_opacity,

            // Notification control commands
            commands::notification_control::pause_system_notifications,
            commands::notification_control::resume_system_notifications,
            commands::notification_control::get_notification_control_state,
            commands::notification_control::check_notification_permission,

            // Tray icon state commands
            commands::tray::set_tray_state,
            commands::tray::get_tray_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
