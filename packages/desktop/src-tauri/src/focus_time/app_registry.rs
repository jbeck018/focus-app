// focus_time/app_registry.rs - App name to process name mapping
//
// This module provides:
// 1. Mapping of friendly app names to actual process names
// 2. macOS bundle identifier mappings
// 3. Predefined app categories for Focus Time
// 4. Cross-platform process name normalization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// App registry for mapping friendly names to process names
#[derive(Debug, Clone)]
pub struct AppRegistry {
    /// Friendly name -> process name(s)
    app_to_process: HashMap<String, Vec<String>>,
    /// Process name -> friendly name
    process_to_app: HashMap<String, String>,
    /// Category -> list of apps
    categories: HashMap<String, Vec<String>>,
    /// macOS bundle identifier -> process name
    #[cfg(target_os = "macos")]
    bundle_to_process: HashMap<String, String>,
}

impl Default for AppRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AppRegistry {
    /// Create a new AppRegistry with default mappings
    pub fn new() -> Self {
        let mut registry = Self {
            app_to_process: HashMap::new(),
            process_to_app: HashMap::new(),
            categories: HashMap::new(),
            #[cfg(target_os = "macos")]
            bundle_to_process: HashMap::new(),
        };

        registry.init_default_mappings();
        registry.init_categories();

        registry
    }

    /// Initialize default app to process mappings
    fn init_default_mappings(&mut self) {
        // Code Editors
        self.add_mapping("vscode", &["code", "Code", "Visual Studio Code", "code.exe"]);
        self.add_mapping("visual studio code", &["code", "Code", "code.exe"]);
        self.add_mapping("cursor", &["cursor", "Cursor", "cursor.exe"]);
        self.add_mapping("sublime", &["sublime_text", "Sublime Text", "sublime_text.exe"]);
        self.add_mapping("atom", &["atom", "Atom", "atom.exe"]);
        self.add_mapping("vim", &["vim", "nvim", "gvim", "mvim"]);
        self.add_mapping("neovim", &["nvim", "neovim"]);
        self.add_mapping("emacs", &["emacs", "Emacs", "emacs.exe"]);
        self.add_mapping("zed", &["zed", "Zed"]);

        // JetBrains IDEs
        self.add_mapping("intellij", &["idea", "IntelliJ IDEA", "idea64.exe"]);
        self.add_mapping("webstorm", &["webstorm", "WebStorm", "webstorm64.exe"]);
        self.add_mapping("pycharm", &["pycharm", "PyCharm", "pycharm64.exe"]);
        self.add_mapping("rustrover", &["rustrover", "RustRover"]);
        self.add_mapping("goland", &["goland", "GoLand", "goland64.exe"]);
        self.add_mapping("clion", &["clion", "CLion", "clion64.exe"]);
        self.add_mapping("datagrip", &["datagrip", "DataGrip", "datagrip64.exe"]);

        // Apple Development
        self.add_mapping("xcode", &["Xcode", "xcode"]);
        self.add_mapping("android studio", &["studio", "Android Studio", "studio64.exe"]);

        // Terminals
        self.add_mapping("terminal", &["Terminal", "terminal", "com.apple.Terminal"]);
        self.add_mapping("iterm", &["iTerm2", "iTerm", "iterm2"]);
        self.add_mapping("iterm2", &["iTerm2", "iTerm", "iterm2"]);
        self.add_mapping("hyper", &["Hyper", "hyper", "hyper.exe"]);
        self.add_mapping("alacritty", &["alacritty", "Alacritty", "alacritty.exe"]);
        self.add_mapping("kitty", &["kitty", "Kitty"]);
        self.add_mapping("warp", &["Warp", "warp"]);
        self.add_mapping("windows terminal", &["WindowsTerminal", "wt.exe"]);
        self.add_mapping("powershell", &["powershell", "pwsh", "powershell.exe"]);
        self.add_mapping("cmd", &["cmd", "cmd.exe"]);

        // Browsers
        self.add_mapping("chrome", &["Google Chrome", "chrome", "chrome.exe"]);
        self.add_mapping("google chrome", &["Google Chrome", "chrome", "chrome.exe"]);
        self.add_mapping("firefox", &["firefox", "Firefox", "firefox.exe"]);
        self.add_mapping("safari", &["Safari", "safari"]);
        self.add_mapping("edge", &["Microsoft Edge", "msedge", "msedge.exe"]);
        self.add_mapping("brave", &["Brave Browser", "brave", "brave.exe"]);
        self.add_mapping("arc", &["Arc", "arc"]);
        self.add_mapping("opera", &["opera", "Opera", "opera.exe"]);

        // Communication
        self.add_mapping("slack", &["Slack", "slack", "slack.exe"]);
        self.add_mapping("discord", &["Discord", "discord", "discord.exe"]);
        self.add_mapping("teams", &["Microsoft Teams", "Teams", "ms-teams.exe"]);
        self.add_mapping("zoom", &["zoom.us", "Zoom", "zoom.exe"]);
        self.add_mapping("telegram", &["Telegram", "telegram-desktop", "telegram.exe"]);
        self.add_mapping("whatsapp", &["WhatsApp", "whatsapp"]);
        self.add_mapping("messages", &["Messages", "imessage"]);
        self.add_mapping("mail", &["Mail", "mail"]);
        self.add_mapping("outlook", &["Microsoft Outlook", "Outlook", "outlook.exe"]);

        // Productivity
        self.add_mapping("notion", &["Notion", "notion", "notion.exe"]);
        self.add_mapping("obsidian", &["Obsidian", "obsidian", "obsidian.exe"]);
        self.add_mapping("todoist", &["Todoist", "todoist"]);
        self.add_mapping("things", &["Things3", "Things"]);
        self.add_mapping("linear", &["Linear", "linear"]);
        self.add_mapping("trello", &["Trello", "trello"]);
        self.add_mapping("asana", &["Asana", "asana"]);
        self.add_mapping("notes", &["Notes", "notes"]);

        // Writing
        self.add_mapping("word", &["Microsoft Word", "Word", "WINWORD.EXE"]);
        self.add_mapping("pages", &["Pages", "pages"]);
        self.add_mapping("bear", &["Bear", "bear"]);
        self.add_mapping("ulysses", &["Ulysses", "ulysses"]);
        self.add_mapping("ia writer", &["iA Writer", "ia-writer"]);
        self.add_mapping("typora", &["Typora", "typora", "typora.exe"]);

        // Design
        self.add_mapping("figma", &["Figma", "figma", "figma.exe"]);
        self.add_mapping("sketch", &["Sketch", "sketch"]);
        self.add_mapping("photoshop", &["Adobe Photoshop", "Photoshop", "photoshop.exe"]);
        self.add_mapping("illustrator", &["Adobe Illustrator", "Illustrator", "illustrator.exe"]);
        self.add_mapping("xd", &["Adobe XD", "xd"]);
        self.add_mapping("canva", &["Canva", "canva"]);
        self.add_mapping("affinity designer", &["Affinity Designer", "affinity-designer"]);

        // Music / Focus
        self.add_mapping("spotify", &["Spotify", "spotify", "spotify.exe"]);
        self.add_mapping("apple music", &["Music", "music"]);
        self.add_mapping("music", &["Music", "music"]);

        // System/Utilities (always allowed by default)
        self.add_mapping("finder", &["Finder", "finder"]);
        self.add_mapping("explorer", &["explorer", "explorer.exe"]);
        self.add_mapping("system preferences", &["System Preferences", "System Settings"]);
        self.add_mapping("settings", &["System Settings", "Settings"]);
        self.add_mapping("activity monitor", &["Activity Monitor"]);
        self.add_mapping("task manager", &["taskmgr", "taskmgr.exe"]);

        // Initialize macOS bundle identifiers
        #[cfg(target_os = "macos")]
        self.init_bundle_mappings();
    }

    /// Add a mapping from friendly name to process names
    fn add_mapping(&mut self, friendly: &str, processes: &[&str]) {
        let friendly_lower = friendly.to_lowercase();
        let process_list: Vec<String> = processes.iter().map(|s| s.to_string()).collect();

        self.app_to_process.insert(friendly_lower.clone(), process_list.clone());

        for process in &process_list {
            let process_lower = process.to_lowercase();
            if !self.process_to_app.contains_key(&process_lower) {
                self.process_to_app.insert(process_lower, friendly.to_string());
            }
        }
    }

    /// Initialize macOS bundle identifier mappings
    #[cfg(target_os = "macos")]
    fn init_bundle_mappings(&mut self) {
        let bundles = [
            // Code Editors & IDEs
            ("com.microsoft.VSCode", "code"),
            ("com.microsoft.VSCodeInsiders", "code"),
            ("com.todesktop.230313mzl4w4u92", "Cursor"),
            ("com.sublimetext.4", "Sublime Text"),
            ("com.sublimetext.3", "Sublime Text"),
            ("io.atom.Atom", "Atom"),
            ("dev.zed.Zed", "Zed"),
            ("org.vim.MacVim", "MacVim"),
            // JetBrains IDEs
            ("com.jetbrains.intellij", "IntelliJ IDEA"),
            ("com.jetbrains.intellij.ce", "IntelliJ IDEA CE"),
            ("com.jetbrains.WebStorm", "WebStorm"),
            ("com.jetbrains.pycharm", "PyCharm"),
            ("com.jetbrains.pycharm.ce", "PyCharm CE"),
            ("com.jetbrains.CLion", "CLion"),
            ("com.jetbrains.goland", "GoLand"),
            ("com.jetbrains.rustrover", "RustRover"),
            ("com.jetbrains.DataGrip", "DataGrip"),
            ("com.jetbrains.rider", "Rider"),
            // Apple Development
            ("com.apple.dt.Xcode", "Xcode"),
            ("com.google.android.studio", "Android Studio"),
            // Terminals
            ("com.apple.Terminal", "Terminal"),
            ("com.googlecode.iterm2", "iTerm2"),
            ("dev.warp.Warp-Stable", "Warp"),
            ("io.alacritty", "Alacritty"),
            ("net.kovidgoyal.kitty", "kitty"),
            ("co.zeit.hyper", "Hyper"),
            // Browsers
            ("com.google.Chrome", "Google Chrome"),
            ("com.google.Chrome.canary", "Google Chrome Canary"),
            ("com.apple.Safari", "Safari"),
            ("org.mozilla.firefox", "firefox"),
            ("org.mozilla.firefoxdeveloperedition", "Firefox Developer Edition"),
            ("com.microsoft.edgemac", "Microsoft Edge"),
            ("com.brave.Browser", "Brave Browser"),
            ("company.thebrowser.Browser", "Arc"),
            ("com.operasoftware.Opera", "Opera"),
            ("com.vivaldi.Vivaldi", "Vivaldi"),
            // Communication
            ("com.tinyspeck.slackmacgap", "Slack"),
            ("com.hnc.Discord", "Discord"),
            ("com.microsoft.teams", "Microsoft Teams"),
            ("com.microsoft.teams2", "Microsoft Teams"),
            ("us.zoom.xos", "zoom.us"),
            ("org.telegram.desktop", "Telegram"),
            ("net.whatsapp.WhatsApp", "WhatsApp"),
            ("com.apple.MobileSMS", "Messages"),
            ("com.apple.mail", "Mail"),
            ("com.microsoft.Outlook", "Microsoft Outlook"),
            // Productivity & Notes
            ("notion.id", "Notion"),
            ("md.obsidian", "Obsidian"),
            ("com.todoist.mac.Todoist", "Todoist"),
            ("com.culturedcode.ThingsMac", "Things3"),
            ("com.linear", "Linear"),
            ("com.apple.Notes", "Notes"),
            ("com.apple.reminders", "Reminders"),
            ("net.shinyfrog.bear", "Bear"),
            ("com.ulyssesapp.mac", "Ulysses"),
            ("com.ragingmenace.SoulverMac", "Soulver"),
            ("com.flexibits.fantastical2.mac", "Fantastical"),
            // Writing
            ("com.microsoft.Word", "Microsoft Word"),
            ("com.apple.iWork.Pages", "Pages"),
            ("abnerworks.Typora", "Typora"),
            ("pro.writer.mac", "iA Writer"),
            // Design
            ("com.figma.Desktop", "Figma"),
            ("com.bohemiancoding.sketch3", "Sketch"),
            ("com.adobe.Photoshop", "Adobe Photoshop"),
            ("com.adobe.Illustrator", "Adobe Illustrator"),
            ("com.adobe.xd", "Adobe XD"),
            ("com.adobe.InDesign", "Adobe InDesign"),
            ("com.serif.affinity-designer", "Affinity Designer"),
            ("com.serif.affinity-photo", "Affinity Photo"),
            // Music & Media
            ("com.spotify.client", "Spotify"),
            ("com.apple.Music", "Music"),
            ("com.apple.podcasts", "Podcasts"),
            // System & Utilities
            ("com.apple.finder", "Finder"),
            ("com.apple.systempreferences", "System Preferences"),
            ("com.apple.systemsettings", "System Settings"),
            ("com.apple.ActivityMonitor", "Activity Monitor"),
            ("com.apple.Console", "Console"),
            ("com.apple.Preview", "Preview"),
            ("com.apple.TextEdit", "TextEdit"),
            // Development Tools
            ("com.postmanlabs.mac", "Postman"),
            ("com.insomnia.app", "Insomnia"),
            ("com.docker.docker", "Docker Desktop"),
            ("com.github.GitHubClient", "GitHub Desktop"),
            ("com.sublimemerge", "Sublime Merge"),
            ("com.todesktop.ForkApp", "Fork"),
        ];

        for (bundle_id, process_name) in bundles {
            self.bundle_to_process.insert(bundle_id.to_string(), process_name.to_string());
        }
    }

    /// Initialize app categories
    fn init_categories(&mut self) {
        self.categories.insert(
            "@coding".to_string(),
            vec![
                "vscode", "visual studio code", "cursor", "sublime", "vim", "neovim", "emacs",
                "zed", "intellij", "webstorm", "pycharm", "rustrover", "goland", "clion",
                "xcode", "android studio",
            ].into_iter().map(String::from).collect(),
        );

        self.categories.insert(
            "@terminal".to_string(),
            vec![
                "terminal", "iterm", "iterm2", "hyper", "alacritty", "kitty", "warp",
                "windows terminal", "powershell", "cmd",
            ].into_iter().map(String::from).collect(),
        );

        self.categories.insert(
            "@browser".to_string(),
            vec![
                "chrome", "google chrome", "firefox", "safari", "edge", "brave", "arc", "opera",
            ].into_iter().map(String::from).collect(),
        );

        self.categories.insert(
            "@communication".to_string(),
            vec![
                "slack", "discord", "teams", "zoom", "telegram", "whatsapp", "messages",
                "mail", "outlook",
            ].into_iter().map(String::from).collect(),
        );

        self.categories.insert(
            "@writing".to_string(),
            vec![
                "word", "pages", "notion", "obsidian", "bear", "ulysses", "ia writer",
                "typora", "notes",
            ].into_iter().map(String::from).collect(),
        );

        self.categories.insert(
            "@design".to_string(),
            vec![
                "figma", "sketch", "photoshop", "illustrator", "xd", "canva",
                "affinity designer",
            ].into_iter().map(String::from).collect(),
        );

        self.categories.insert(
            "@productivity".to_string(),
            vec![
                "notion", "obsidian", "todoist", "things", "linear", "trello", "asana", "notes",
            ].into_iter().map(String::from).collect(),
        );

        self.categories.insert(
            "@music".to_string(),
            vec!["spotify", "apple music", "music"].into_iter().map(String::from).collect(),
        );
    }

    /// Get process names for a friendly app name
    pub fn get_process_names(&self, friendly_name: &str) -> Option<Vec<String>> {
        let key = friendly_name.to_lowercase();
        self.app_to_process.get(&key).cloned()
    }

    /// Get the primary process name for a friendly app name
    pub fn get_process_name(&self, friendly_name: &str) -> Option<String> {
        let key = friendly_name.to_lowercase();
        self.app_to_process.get(&key).and_then(|v| v.first().cloned())
    }

    /// Get friendly app name from a process name
    pub fn get_friendly_name(&self, process_name: &str) -> Option<String> {
        let key = process_name.to_lowercase();
        self.process_to_app.get(&key).cloned()
    }

    /// Get apps in a category (e.g., "@coding")
    pub fn get_category_apps(&self, category: &str) -> Option<Vec<String>> {
        let key = if category.starts_with('@') {
            category.to_lowercase()
        } else {
            format!("@{}", category.to_lowercase())
        };
        self.categories.get(&key).cloned()
    }

    /// Expand a category to all its process names
    pub fn expand_category(&self, category: &str) -> Vec<String> {
        let mut processes = Vec::new();

        if let Some(apps) = self.get_category_apps(category) {
            for app in apps {
                if let Some(procs) = self.get_process_names(&app) {
                    processes.extend(procs);
                }
            }
        }

        // Deduplicate
        processes.sort();
        processes.dedup();
        processes
    }

    /// Expand a list of allowed items (apps and categories) to process names
    pub fn expand_allowed_list(&self, items: &[String]) -> Vec<String> {
        let mut processes = Vec::new();

        for item in items {
            if item.starts_with('@') {
                // It's a category
                processes.extend(self.expand_category(item));
            } else if let Some(procs) = self.get_process_names(item) {
                // It's a known app
                processes.extend(procs);
            } else {
                // Unknown app - add as-is (user might have specified exact process name)
                processes.push(item.clone());
            }
        }

        // Deduplicate
        processes.sort();
        processes.dedup();
        processes
    }

    /// Get process name from macOS bundle identifier
    #[cfg(target_os = "macos")]
    pub fn get_process_from_bundle(&self, bundle_id: &str) -> Option<String> {
        self.bundle_to_process.get(bundle_id).cloned()
    }

    /// Check if a process name matches any in the allowed list
    pub fn is_process_allowed(&self, process_name: &str, allowed_items: &[String]) -> bool {
        let process_lower = process_name.to_lowercase();
        let process_normalized = normalize_process_name(&process_lower);

        // Expand all allowed items to process names
        let allowed_processes = self.expand_allowed_list(allowed_items);

        for allowed in &allowed_processes {
            let allowed_normalized = normalize_process_name(&allowed.to_lowercase());

            // Exact match
            if process_normalized == allowed_normalized {
                return true;
            }

            // Substring match (handles "Visual Studio Code" containing "code")
            if process_normalized.contains(&allowed_normalized)
                || allowed_normalized.contains(&process_normalized)
            {
                return true;
            }
        }

        false
    }

    /// Get all available categories
    pub fn get_all_categories(&self) -> Vec<String> {
        self.categories.keys().cloned().collect()
    }
}

/// Normalize a process name for comparison
fn normalize_process_name(name: &str) -> String {
    let mut normalized = name.trim().to_lowercase();

    // Remove common extensions
    for ext in [".exe", ".app", ".bat", ".cmd"] {
        if normalized.ends_with(ext) {
            normalized = normalized[..normalized.len() - ext.len()].to_string();
            break;
        }
    }

    normalized
}

/// Critical system processes that should never be terminated
/// Platform-specific protection lists to prevent accidental system damage
#[cfg(target_os = "windows")]
const PROTECTED_PROCESSES: &[&str] = &[
    // Core Windows system processes
    "system",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "svchost.exe",
    "dwm.exe",
    "explorer.exe",
    "taskmgr.exe",
    "conhost.exe",
    "audiodg.exe",
    "fontdrvhost.exe",
    "spoolsv.exe",
    "runtimebroker.exe",
    "sihost.exe",
    "taskhostw.exe",
    "registry",
    "memory compression",
    // Windows security
    "securityhealthservice.exe",
    "msmpeng.exe",
    "nissrv.exe",
    "smartscreen.exe",
    // Windows shell
    "shellexperiencehost.exe",
    "searchui.exe",
    "searchhost.exe",
    "startmenuexperiencehost.exe",
    // Windows services
    "ctfmon.exe",
    "dllhost.exe",
    "wmiprvse.exe",
    "wudfhost.exe",
    "dashost.exe",
    "searchindexer.exe",
    // Input/display
    "textinputhost.exe",
    "inputapp.exe",
    "microsoft.photos.exe",
    // System idle
    "system idle process",
    // Critical driver hosts
    "wudfhost.exe",
    "sppsvc.exe",
];

#[cfg(target_os = "macos")]
const PROTECTED_PROCESSES: &[&str] = &[
    // Kernel and core system
    "kernel_task",
    "launchd",
    "launchd_sim",
    "WindowServer",
    "loginwindow",
    "SystemUIServer",
    "Dock",
    "Finder",
    // User session
    "cfprefsd",
    "pboard",
    "sharedfilelistd",
    "usernoted",
    "useractivityd",
    "contextstored",
    // System daemons
    "sysmond",
    "diskarbitrationd",
    "configd",
    "notifyd",
    "opendirectoryd",
    "powerd",
    "fseventsd",
    "blued",
    "locationd",
    "identityservicesd",
    "imagent",
    "securityd",
    "trustd",
    "coreduetd",
    "symptomsd",
    // Spotlight
    "mds",
    "mds_stores",
    "mdworker",
    "mdworker_shared",
    "corespotlightd",
    // Audio/Video
    "coreaudiod",
    "audioclocksyncd",
    "coreanimationd",
    // Input
    "hidd",
    "touchbard",
    // Core services
    "coreservicesd",
    "UserEventAgent",
    "lsd",
    "servicemanagementd",
    // Security
    "nesessionmanager",
    "tccd",
    "authd",
    "keybagd",
    // Terminals (allow these for usability)
    "Activity Monitor",
    "Console",
    "Terminal",
    "iTerm2",
    "Warp",
    // Window management
    "Accessibility",
    "universalaccessd",
];

#[cfg(target_os = "linux")]
const PROTECTED_PROCESSES: &[&str] = &[
    // Init systems
    "systemd",
    "init",
    "upstart",
    // Kernel threads
    "kthreadd",
    "ksoftirqd",
    "kworker",
    "rcu_sched",
    "rcu_bh",
    "migration",
    "watchdog",
    "kdevtmpfs",
    "kauditd",
    "khungtaskd",
    // D-Bus
    "dbus-daemon",
    "dbus-broker",
    // Display servers
    "X",
    "Xorg",
    "Xwayland",
    "gnome-shell",
    "kwin_x11",
    "kwin_wayland",
    "mutter",
    "sway",
    // Session managers
    "xfce4-session",
    "gnome-session",
    "gnome-session-binary",
    "kde-session",
    "plasmashell",
    "lxsession",
    // Display managers
    "lightdm",
    "gdm",
    "gdm3",
    "sddm",
    "xdm",
    // Systemd components
    "systemd-logind",
    "systemd-journald",
    "systemd-udevd",
    "systemd-resolved",
    "systemd-timesyncd",
    // Audio
    "pulseaudio",
    "pipewire",
    "pipewire-pulse",
    "wireplumber",
    // Networking
    "NetworkManager",
    "wpa_supplicant",
    "dhclient",
    "avahi-daemon",
    // Security
    "polkitd",
    "accounts-daemon",
    // Core desktop
    "gsd-keyboard",
    "gsd-media-keys",
    "gsd-power",
    "gsd-wacom",
    "nautilus",
    "dolphin",
    "thunar",
];

/// Default fallback for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
const PROTECTED_PROCESSES: &[&str] = &[];

/// Check if a process is protected from termination
///
/// Returns true if the process is a critical system process that should never be killed.
/// This prevents accidental system instability or crashes.
pub fn is_protected_process(process_name: &str) -> bool {
    let normalized = normalize_process_name(process_name);

    PROTECTED_PROCESSES.iter().any(|&protected| {
        let normalized_protected = normalize_process_name(protected);
        normalized == normalized_protected
    })
}

/// App entry for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppEntry {
    /// Friendly name for display
    pub name: String,
    /// Icon identifier (can be used for UI)
    pub icon: Option<String>,
    /// Category this app belongs to
    pub category: Option<String>,
    /// Process names this app maps to
    pub processes: Vec<String>,
}

/// Get a list of common apps for the UI
pub fn get_common_apps() -> Vec<AppEntry> {
    vec![
        // Code Editors & IDEs
        AppEntry {
            name: "VS Code".to_string(),
            icon: Some("vscode".to_string()),
            category: Some("coding".to_string()),
            processes: vec!["code".to_string(), "Code".to_string(), "Visual Studio Code".to_string()],
        },
        AppEntry {
            name: "Cursor".to_string(),
            icon: Some("cursor".to_string()),
            category: Some("coding".to_string()),
            processes: vec!["Cursor".to_string(), "cursor".to_string()],
        },
        AppEntry {
            name: "Xcode".to_string(),
            icon: Some("xcode".to_string()),
            category: Some("coding".to_string()),
            processes: vec!["Xcode".to_string()],
        },
        AppEntry {
            name: "IntelliJ IDEA".to_string(),
            icon: Some("intellij".to_string()),
            category: Some("coding".to_string()),
            processes: vec!["IntelliJ IDEA".to_string(), "idea".to_string()],
        },
        AppEntry {
            name: "PyCharm".to_string(),
            icon: Some("pycharm".to_string()),
            category: Some("coding".to_string()),
            processes: vec!["PyCharm".to_string(), "pycharm".to_string()],
        },
        AppEntry {
            name: "WebStorm".to_string(),
            icon: Some("webstorm".to_string()),
            category: Some("coding".to_string()),
            processes: vec!["WebStorm".to_string(), "webstorm".to_string()],
        },
        AppEntry {
            name: "Sublime Text".to_string(),
            icon: Some("sublime".to_string()),
            category: Some("coding".to_string()),
            processes: vec!["Sublime Text".to_string(), "sublime_text".to_string()],
        },
        AppEntry {
            name: "Zed".to_string(),
            icon: Some("zed".to_string()),
            category: Some("coding".to_string()),
            processes: vec!["Zed".to_string(), "zed".to_string()],
        },
        // Terminals
        AppEntry {
            name: "Terminal".to_string(),
            icon: Some("terminal".to_string()),
            category: Some("terminal".to_string()),
            processes: vec!["Terminal".to_string()],
        },
        AppEntry {
            name: "iTerm2".to_string(),
            icon: Some("iterm".to_string()),
            category: Some("terminal".to_string()),
            processes: vec!["iTerm2".to_string(), "iTerm".to_string()],
        },
        AppEntry {
            name: "Warp".to_string(),
            icon: Some("warp".to_string()),
            category: Some("terminal".to_string()),
            processes: vec!["Warp".to_string()],
        },
        AppEntry {
            name: "Alacritty".to_string(),
            icon: Some("alacritty".to_string()),
            category: Some("terminal".to_string()),
            processes: vec!["Alacritty".to_string(), "alacritty".to_string()],
        },
        // Browsers
        AppEntry {
            name: "Chrome".to_string(),
            icon: Some("chrome".to_string()),
            category: Some("browser".to_string()),
            processes: vec!["Google Chrome".to_string(), "chrome".to_string()],
        },
        AppEntry {
            name: "Safari".to_string(),
            icon: Some("safari".to_string()),
            category: Some("browser".to_string()),
            processes: vec!["Safari".to_string()],
        },
        AppEntry {
            name: "Firefox".to_string(),
            icon: Some("firefox".to_string()),
            category: Some("browser".to_string()),
            processes: vec!["Firefox".to_string(), "firefox".to_string()],
        },
        AppEntry {
            name: "Arc".to_string(),
            icon: Some("arc".to_string()),
            category: Some("browser".to_string()),
            processes: vec!["Arc".to_string()],
        },
        AppEntry {
            name: "Edge".to_string(),
            icon: Some("edge".to_string()),
            category: Some("browser".to_string()),
            processes: vec!["Microsoft Edge".to_string(), "msedge".to_string()],
        },
        AppEntry {
            name: "Brave".to_string(),
            icon: Some("brave".to_string()),
            category: Some("browser".to_string()),
            processes: vec!["Brave Browser".to_string(), "brave".to_string()],
        },
        // Communication
        AppEntry {
            name: "Slack".to_string(),
            icon: Some("slack".to_string()),
            category: Some("communication".to_string()),
            processes: vec!["Slack".to_string()],
        },
        AppEntry {
            name: "Discord".to_string(),
            icon: Some("discord".to_string()),
            category: Some("communication".to_string()),
            processes: vec!["Discord".to_string()],
        },
        AppEntry {
            name: "Zoom".to_string(),
            icon: Some("zoom".to_string()),
            category: Some("communication".to_string()),
            processes: vec!["zoom.us".to_string(), "Zoom".to_string()],
        },
        AppEntry {
            name: "Microsoft Teams".to_string(),
            icon: Some("teams".to_string()),
            category: Some("communication".to_string()),
            processes: vec!["Microsoft Teams".to_string(), "Teams".to_string()],
        },
        AppEntry {
            name: "Telegram".to_string(),
            icon: Some("telegram".to_string()),
            category: Some("communication".to_string()),
            processes: vec!["Telegram".to_string(), "telegram-desktop".to_string()],
        },
        // Productivity
        AppEntry {
            name: "Notion".to_string(),
            icon: Some("notion".to_string()),
            category: Some("productivity".to_string()),
            processes: vec!["Notion".to_string()],
        },
        AppEntry {
            name: "Obsidian".to_string(),
            icon: Some("obsidian".to_string()),
            category: Some("productivity".to_string()),
            processes: vec!["Obsidian".to_string()],
        },
        AppEntry {
            name: "Todoist".to_string(),
            icon: Some("todoist".to_string()),
            category: Some("productivity".to_string()),
            processes: vec!["Todoist".to_string()],
        },
        AppEntry {
            name: "Linear".to_string(),
            icon: Some("linear".to_string()),
            category: Some("productivity".to_string()),
            processes: vec!["Linear".to_string()],
        },
        AppEntry {
            name: "Things".to_string(),
            icon: Some("things".to_string()),
            category: Some("productivity".to_string()),
            processes: vec!["Things3".to_string(), "Things".to_string()],
        },
        AppEntry {
            name: "Notes".to_string(),
            icon: Some("notes".to_string()),
            category: Some("productivity".to_string()),
            processes: vec!["Notes".to_string()],
        },
        // Writing
        AppEntry {
            name: "Word".to_string(),
            icon: Some("word".to_string()),
            category: Some("writing".to_string()),
            processes: vec!["Microsoft Word".to_string(), "Word".to_string()],
        },
        AppEntry {
            name: "Pages".to_string(),
            icon: Some("pages".to_string()),
            category: Some("writing".to_string()),
            processes: vec!["Pages".to_string()],
        },
        AppEntry {
            name: "Bear".to_string(),
            icon: Some("bear".to_string()),
            category: Some("writing".to_string()),
            processes: vec!["Bear".to_string()],
        },
        AppEntry {
            name: "Ulysses".to_string(),
            icon: Some("ulysses".to_string()),
            category: Some("writing".to_string()),
            processes: vec!["Ulysses".to_string()],
        },
        // Design
        AppEntry {
            name: "Figma".to_string(),
            icon: Some("figma".to_string()),
            category: Some("design".to_string()),
            processes: vec!["Figma".to_string()],
        },
        AppEntry {
            name: "Sketch".to_string(),
            icon: Some("sketch".to_string()),
            category: Some("design".to_string()),
            processes: vec!["Sketch".to_string()],
        },
        AppEntry {
            name: "Photoshop".to_string(),
            icon: Some("photoshop".to_string()),
            category: Some("design".to_string()),
            processes: vec!["Adobe Photoshop".to_string(), "Photoshop".to_string()],
        },
        AppEntry {
            name: "Illustrator".to_string(),
            icon: Some("illustrator".to_string()),
            category: Some("design".to_string()),
            processes: vec!["Adobe Illustrator".to_string(), "Illustrator".to_string()],
        },
        AppEntry {
            name: "Canva".to_string(),
            icon: Some("canva".to_string()),
            category: Some("design".to_string()),
            processes: vec!["Canva".to_string()],
        },
        // Music
        AppEntry {
            name: "Spotify".to_string(),
            icon: Some("spotify".to_string()),
            category: Some("music".to_string()),
            processes: vec!["Spotify".to_string()],
        },
        AppEntry {
            name: "Apple Music".to_string(),
            icon: Some("music".to_string()),
            category: Some("music".to_string()),
            processes: vec!["Music".to_string()],
        },
    ]
}

/// Get all available app categories with their apps
pub fn get_app_categories() -> Vec<CategoryInfo> {
    vec![
        CategoryInfo {
            id: "@coding".to_string(),
            name: "Coding".to_string(),
            description: "Code editors and IDEs".to_string(),
            example_apps: vec!["VS Code".to_string(), "Xcode".to_string(), "IntelliJ".to_string()],
        },
        CategoryInfo {
            id: "@terminal".to_string(),
            name: "Terminal".to_string(),
            description: "Terminal emulators and shells".to_string(),
            example_apps: vec!["Terminal".to_string(), "iTerm2".to_string(), "Warp".to_string()],
        },
        CategoryInfo {
            id: "@browser".to_string(),
            name: "Browser".to_string(),
            description: "Web browsers".to_string(),
            example_apps: vec!["Chrome".to_string(), "Safari".to_string(), "Firefox".to_string()],
        },
        CategoryInfo {
            id: "@communication".to_string(),
            name: "Communication".to_string(),
            description: "Chat and email apps".to_string(),
            example_apps: vec!["Slack".to_string(), "Discord".to_string(), "Zoom".to_string()],
        },
        CategoryInfo {
            id: "@writing".to_string(),
            name: "Writing".to_string(),
            description: "Writing and note-taking apps".to_string(),
            example_apps: vec!["Notion".to_string(), "Obsidian".to_string(), "Word".to_string()],
        },
        CategoryInfo {
            id: "@design".to_string(),
            name: "Design".to_string(),
            description: "Design and creative tools".to_string(),
            example_apps: vec!["Figma".to_string(), "Sketch".to_string(), "Photoshop".to_string()],
        },
        CategoryInfo {
            id: "@productivity".to_string(),
            name: "Productivity".to_string(),
            description: "Task management and productivity apps".to_string(),
            example_apps: vec!["Notion".to_string(), "Todoist".to_string(), "Linear".to_string()],
        },
        CategoryInfo {
            id: "@music".to_string(),
            name: "Music".to_string(),
            description: "Music streaming apps".to_string(),
            example_apps: vec!["Spotify".to_string(), "Apple Music".to_string()],
        },
    ]
}

/// Category information for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInfo {
    /// Category identifier (e.g., "@coding")
    pub id: String,
    /// Display name
    pub name: String,
    /// Description of the category
    pub description: String,
    /// Example apps in this category
    pub example_apps: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_registry_basic() {
        let registry = AppRegistry::new();

        // Test process name lookup
        let procs = registry.get_process_names("vscode").unwrap();
        assert!(procs.contains(&"code".to_string()));

        // Test friendly name lookup
        let friendly = registry.get_friendly_name("code").unwrap();
        assert_eq!(friendly.to_lowercase(), "vscode");
    }

    #[test]
    fn test_category_expansion() {
        let registry = AppRegistry::new();

        let coding_processes = registry.expand_category("@coding");
        assert!(!coding_processes.is_empty());
        assert!(coding_processes.iter().any(|p| p.to_lowercase().contains("code")));
    }

    #[test]
    fn test_allowed_list_expansion() {
        let registry = AppRegistry::new();

        let allowed = vec!["@terminal".to_string(), "notion".to_string()];
        let processes = registry.expand_allowed_list(&allowed);

        assert!(processes.iter().any(|p| p.to_lowercase().contains("terminal")));
        assert!(processes.iter().any(|p| p.to_lowercase().contains("notion")));
    }

    #[test]
    fn test_is_process_allowed() {
        let registry = AppRegistry::new();

        let allowed = vec!["@coding".to_string(), "slack".to_string()];

        assert!(registry.is_process_allowed("code", &allowed));
        assert!(registry.is_process_allowed("Visual Studio Code", &allowed));
        assert!(registry.is_process_allowed("Slack", &allowed));
        assert!(!registry.is_process_allowed("Chrome", &allowed));
    }

    #[test]
    fn test_normalize_process_name() {
        assert_eq!(normalize_process_name("chrome.exe"), "chrome");
        assert_eq!(normalize_process_name("Safari.app"), "safari");
        assert_eq!(normalize_process_name("Code"), "code");
        assert_eq!(normalize_process_name("  Terminal  "), "terminal");
    }
}
