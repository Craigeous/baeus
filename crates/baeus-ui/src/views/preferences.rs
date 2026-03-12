//! Preferences panel — settings view.
//!
//! Defines `PreferencesSection` (sidebar categories) and `PreferencesState`
//! (the mutable form state that maps to persisted `UserPreferences`).

use std::collections::HashMap;

use crate::theme::ThemeMode;

// -----------------------------------------------------------------------
// Section enum
// -----------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreferencesSection {
    #[default]
    App,
    Kubernetes,
    Terminal,
    About,
}

impl PreferencesSection {
    pub fn all() -> &'static [Self] {
        &[Self::App, Self::Kubernetes, Self::Terminal, Self::About]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::App => "App",
            Self::Kubernetes => "Kubernetes",
            Self::Terminal => "Terminal",
            Self::About => "About",
        }
    }
}

// -----------------------------------------------------------------------
// PreferencesState — in-memory form state mirroring UserPreferences fields
// -----------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PreferencesState {
    pub theme_mode: ThemeMode,
    pub font_size: f32,
    pub log_line_limit: u32,
    pub default_namespace: Option<String>,
    pub kubeconfig_scan_dirs: Vec<String>,
    pub terminal_shell_path: Option<String>,
    pub sidebar_collapsed: bool,
    pub default_aws_profile: Option<String>,
    pub cluster_aws_profiles: HashMap<String, String>,
    /// Saved EKS connections to restore on app restart.
    pub saved_eks_connections: Vec<SavedEksConnectionInfo>,
}

/// Persisted metadata for an EKS cluster (no secrets).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedEksConnectionInfo {
    pub cluster_name: String,
    pub cluster_arn: String,
    pub endpoint: String,
    pub region: String,
    pub certificate_authority_data: Option<String>,
    pub auth_method: String, // "Sso", "AccessKey", "AssumeRole"
    pub sso_start_url: Option<String>,
    pub sso_region: Option<String>,
    pub role_arn: Option<String>,
}

impl Default for PreferencesState {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::default(),
            font_size: 13.0,
            log_line_limit: 10000,
            default_namespace: None,
            kubeconfig_scan_dirs: Vec::new(),
            terminal_shell_path: None,
            sidebar_collapsed: false,
            default_aws_profile: None,
            cluster_aws_profiles: HashMap::new(),
            saved_eks_connections: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preferences_section_all() {
        let sections = PreferencesSection::all();
        assert_eq!(sections.len(), 4);
        assert_eq!(sections[0], PreferencesSection::App);
        assert_eq!(sections[3], PreferencesSection::About);
    }

    #[test]
    fn test_preferences_section_labels() {
        assert_eq!(PreferencesSection::App.label(), "App");
        assert_eq!(PreferencesSection::Kubernetes.label(), "Kubernetes");
        assert_eq!(PreferencesSection::Terminal.label(), "Terminal");
        assert_eq!(PreferencesSection::About.label(), "About");
    }

    #[test]
    fn test_preferences_section_default() {
        assert_eq!(PreferencesSection::default(), PreferencesSection::App);
    }

    #[test]
    fn test_preferences_state_default() {
        let state = PreferencesState::default();
        assert_eq!(state.theme_mode, ThemeMode::System);
        assert_eq!(state.font_size, 13.0);
        assert_eq!(state.log_line_limit, 10000);
        assert!(state.default_namespace.is_none());
        assert!(state.kubeconfig_scan_dirs.is_empty());
        assert!(state.terminal_shell_path.is_none());
        assert!(!state.sidebar_collapsed);
    }
}
