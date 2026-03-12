// T085: Dark/Light Theme Toggle Tests

use baeus_ui::theme::*;

// --- Toggle from dark to light ---

#[test]
fn test_toggle_dark_to_light() {
    let mut manager = ThemeManager::new(ThemeMode::Dark);
    assert_eq!(manager.current().mode, ThemeMode::Dark);

    manager.toggle();
    assert_eq!(manager.current().mode, ThemeMode::Light);
}

// --- Toggle from light to dark ---

#[test]
fn test_toggle_light_to_dark() {
    let mut manager = ThemeManager::new(ThemeMode::Light);
    assert_eq!(manager.current().mode, ThemeMode::Light);

    manager.toggle();
    assert_eq!(manager.current().mode, ThemeMode::Dark);
}

// --- apply_toggle returns correct theme after toggle ---

#[test]
fn test_apply_toggle_returns_light_from_dark() {
    let mut manager = ThemeManager::new(ThemeMode::Dark);
    let theme = manager.apply_toggle();
    assert_eq!(theme.mode, ThemeMode::Light);
    assert_eq!(theme.colors.background, Color::rgb(255, 255, 255));
}

#[test]
fn test_apply_toggle_returns_dark_from_light() {
    let mut manager = ThemeManager::new(ThemeMode::Light);
    let theme = manager.apply_toggle();
    assert_eq!(theme.mode, ThemeMode::Dark);
    assert_eq!(theme.colors.background, Color::rgb(0x1e, 0x21, 0x24));
}

#[test]
fn test_apply_toggle_matches_current() {
    let mut manager = ThemeManager::new(ThemeMode::Dark);
    let theme = manager.apply_toggle();
    // The returned theme should match the current theme after toggle
    assert_eq!(&theme, manager.current());
}

// --- System mode toggle goes to Light ---

#[test]
fn test_toggle_from_system_goes_to_light() {
    let mut manager = ThemeManager::new(ThemeMode::System);
    manager.toggle();
    assert_eq!(manager.current().mode, ThemeMode::Light);
}

// --- System mode respects appearance ---

#[test]
fn test_system_mode_follows_dark_appearance() {
    let mut manager = ThemeManager::new(ThemeMode::System);
    manager.update_system_appearance(true);
    // System mode with dark appearance should use dark colors
    assert_eq!(manager.current().colors.background, Color::rgb(0x1e, 0x21, 0x24));
    assert_eq!(manager.current().mode, ThemeMode::System);
}

#[test]
fn test_system_mode_follows_light_appearance() {
    let mut manager = ThemeManager::new(ThemeMode::System);
    manager.update_system_appearance(false);
    // System mode with light appearance should use light colors
    assert_eq!(manager.current().colors.background, Color::rgb(255, 255, 255));
    assert_eq!(manager.current().mode, ThemeMode::System);
}

// --- Theme persistence: toggle changes mode, mode can be serialized ---

#[test]
fn test_theme_mode_serialization_after_toggle() {
    let mut manager = ThemeManager::new(ThemeMode::Dark);
    manager.toggle();
    let mode = manager.current().mode;
    // ThemeMode derives Serialize/Deserialize
    let json = serde_json::to_string(&mode).unwrap();
    let deserialized: ThemeMode = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, ThemeMode::Light);
}

#[test]
fn test_theme_mode_roundtrip_all_variants() {
    for mode in &[ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
        let json = serde_json::to_string(mode).unwrap();
        let deserialized: ThemeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(&deserialized, mode);
    }
}

// --- Color tokens change when theme toggles ---

#[test]
fn test_color_tokens_differ_between_light_and_dark() {
    let light = Theme::light();
    let dark = Theme::dark();
    // Backgrounds should differ
    assert_ne!(light.colors.background, dark.colors.background);
    // Text primary should differ
    assert_ne!(light.colors.text_primary, dark.colors.text_primary);
    // Sidebar backgrounds should differ
    assert_ne!(light.colors.sidebar_bg, dark.colors.sidebar_bg);
}

#[test]
fn test_toggle_changes_background_color() {
    let mut manager = ThemeManager::new(ThemeMode::Dark);
    let dark_bg = manager.current().colors.background;

    manager.toggle();
    let light_bg = manager.current().colors.background;

    assert_ne!(dark_bg, light_bg);
    assert_eq!(light_bg, Color::rgb(255, 255, 255));
    assert_eq!(dark_bg, Color::rgb(0x1e, 0x21, 0x24));
}

// --- All ThemeMode variants ---

#[test]
fn test_all_theme_mode_variants() {
    let modes = [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System];
    for mode in &modes {
        let manager = ThemeManager::new(*mode);
        assert_eq!(manager.current().mode, *mode);
    }
}

#[test]
fn test_theme_mode_default_is_system() {
    assert_eq!(ThemeMode::default(), ThemeMode::System);
}

// --- Toggle cycle: Dark -> Light -> Dark ---

#[test]
fn test_toggle_cycle_dark_light_dark() {
    let mut manager = ThemeManager::new(ThemeMode::Dark);
    assert_eq!(manager.current().mode, ThemeMode::Dark);

    manager.toggle();
    assert_eq!(manager.current().mode, ThemeMode::Light);

    manager.toggle();
    assert_eq!(manager.current().mode, ThemeMode::Dark);
}

#[test]
fn test_apply_toggle_cycle() {
    let mut manager = ThemeManager::new(ThemeMode::Light);

    let t1 = manager.apply_toggle();
    assert_eq!(t1.mode, ThemeMode::Dark);

    let t2 = manager.apply_toggle();
    assert_eq!(t2.mode, ThemeMode::Light);

    let t3 = manager.apply_toggle();
    assert_eq!(t3.mode, ThemeMode::Dark);
}

// --- ThemeToggleAction struct ---

#[test]
fn test_theme_toggle_action_clone_and_eq() {
    let action = ThemeToggleAction;
    let cloned = action.clone();
    assert_eq!(action, cloned);
}

#[test]
fn test_theme_toggle_action_debug() {
    let action = ThemeToggleAction;
    let debug = format!("{:?}", action);
    assert_eq!(debug, "ThemeToggleAction");
}

// --- set_mode preserves correct colors ---

#[test]
fn test_set_mode_light_has_white_background() {
    let mut manager = ThemeManager::new(ThemeMode::Dark);
    manager.set_mode(ThemeMode::Light);
    assert_eq!(manager.current().colors.background, Color::rgb(255, 255, 255));
    assert_eq!(manager.current().colors.header_bg, Color::rgb(255, 255, 255));
}

#[test]
fn test_set_mode_dark_has_dark_background() {
    let mut manager = ThemeManager::new(ThemeMode::Light);
    manager.set_mode(ThemeMode::Dark);
    assert_eq!(manager.current().colors.background, Color::rgb(0x1e, 0x21, 0x24));
    assert_eq!(manager.current().colors.header_bg, Color::rgb(0x26, 0x2b, 0x2f));
}
