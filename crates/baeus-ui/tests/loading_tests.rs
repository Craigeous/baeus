// T090: Loading states integration tests

use baeus_ui::components::loading::*;
use baeus_ui::theme::Theme;

// ===========================================================================
// LoadingVariant
// ===========================================================================

#[test]
fn test_loading_variant_spinner() {
    let variant = LoadingVariant::Spinner;
    assert_eq!(variant, LoadingVariant::Spinner);
}

#[test]
fn test_loading_variant_skeleton() {
    let variant = LoadingVariant::Skeleton;
    assert_eq!(variant, LoadingVariant::Skeleton);
}

#[test]
fn test_loading_variant_dots() {
    let variant = LoadingVariant::Dots;
    assert_eq!(variant, LoadingVariant::Dots);
}

#[test]
fn test_loading_variant_bar() {
    let variant = LoadingVariant::Bar;
    assert_eq!(variant, LoadingVariant::Bar);
}

#[test]
fn test_loading_variant_all_distinct() {
    let variants = [
        LoadingVariant::Spinner,
        LoadingVariant::Skeleton,
        LoadingVariant::Dots,
        LoadingVariant::Bar,
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn test_loading_variant_clone() {
    let variant = LoadingVariant::Spinner;
    let cloned = variant.clone();
    assert_eq!(variant, cloned);
}

// ===========================================================================
// LoadingState::spinner()
// ===========================================================================

#[test]
fn test_spinner_with_message() {
    let state = LoadingState::spinner(Some("Loading pods...".to_string()));
    assert_eq!(state.variant, LoadingVariant::Spinner);
    assert_eq!(state.message.as_deref(), Some("Loading pods..."));
    assert!(state.progress.is_none());
}

#[test]
fn test_spinner_without_message() {
    let state = LoadingState::spinner(None);
    assert_eq!(state.variant, LoadingVariant::Spinner);
    assert!(state.message.is_none());
    assert!(state.progress.is_none());
}

// ===========================================================================
// LoadingState::skeleton()
// ===========================================================================

#[test]
fn test_skeleton_creates_state() {
    let state = LoadingState::skeleton(5, 3);
    assert_eq!(state.variant, LoadingVariant::Skeleton);
    assert!(state.message.is_none());
    assert!(state.progress.is_none());
}

// ===========================================================================
// LoadingState::progress_bar()
// ===========================================================================

#[test]
fn test_progress_bar_with_progress() {
    let state = LoadingState::progress_bar(0.5, Some("Uploading...".to_string()));
    assert_eq!(state.variant, LoadingVariant::Bar);
    assert_eq!(state.message.as_deref(), Some("Uploading..."));
    assert!((state.progress.unwrap() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_progress_bar_clamps_below_zero() {
    let state = LoadingState::progress_bar(-0.5, None);
    assert!((state.progress.unwrap() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_progress_bar_clamps_above_one() {
    let state = LoadingState::progress_bar(1.5, None);
    assert!((state.progress.unwrap() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_progress_bar_zero() {
    let state = LoadingState::progress_bar(0.0, None);
    assert!((state.progress.unwrap() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_progress_bar_one() {
    let state = LoadingState::progress_bar(1.0, None);
    assert!((state.progress.unwrap() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_progress_bar_without_message() {
    let state = LoadingState::progress_bar(0.75, None);
    assert!(state.message.is_none());
    assert!((state.progress.unwrap() - 0.75).abs() < 0.001);
}

// ===========================================================================
// LoadingState::dots()
// ===========================================================================

#[test]
fn test_dots_with_message() {
    let state = LoadingState::dots(Some("Connecting".to_string()));
    assert_eq!(state.variant, LoadingVariant::Dots);
    assert_eq!(state.message.as_deref(), Some("Connecting"));
    assert!(state.progress.is_none());
}

#[test]
fn test_dots_without_message() {
    let state = LoadingState::dots(None);
    assert_eq!(state.variant, LoadingVariant::Dots);
    assert!(state.message.is_none());
}

// ===========================================================================
// SkeletonConfig
// ===========================================================================

#[test]
fn test_skeleton_config_new() {
    let config = SkeletonConfig::new(5, 3, 24.0);
    assert_eq!(config.rows, 5);
    assert_eq!(config.columns, 3);
    assert!((config.row_height - 24.0).abs() < f32::EPSILON);
}

#[test]
fn test_skeleton_config_table_default() {
    let config = SkeletonConfig::table_default();
    assert_eq!(config.rows, 10);
    assert_eq!(config.columns, 4);
    assert!((config.row_height - 32.0).abs() < f32::EPSILON);
}

#[test]
fn test_skeleton_config_custom_values() {
    let config = SkeletonConfig::new(20, 6, 48.0);
    assert_eq!(config.rows, 20);
    assert_eq!(config.columns, 6);
    assert!((config.row_height - 48.0).abs() < f32::EPSILON);
}

#[test]
fn test_skeleton_config_clone() {
    let config = SkeletonConfig::table_default();
    let cloned = config.clone();
    assert_eq!(config, cloned);
}

#[test]
fn test_skeleton_config_debug() {
    let config = SkeletonConfig::new(3, 2, 16.0);
    let debug = format!("{:?}", config);
    assert!(debug.contains("SkeletonConfig"));
    assert!(debug.contains("3"));
}

// ===========================================================================
// LoadingViewComponent
// ===========================================================================

#[test]
fn test_loading_view_component_new() {
    let state = LoadingState::spinner(None);
    let view = LoadingViewComponent::new(state.clone(), Theme::dark());
    assert_eq!(view.state, state);
    assert!(view.skeleton_config.is_none());
}

#[test]
fn test_loading_view_component_with_skeleton_config() {
    let state = LoadingState::skeleton(5, 3);
    let config = SkeletonConfig::new(5, 3, 32.0);
    let view = LoadingViewComponent::new(state, Theme::dark()).with_skeleton_config(config.clone());
    assert_eq!(view.skeleton_config, Some(config));
}

#[test]
fn test_loading_view_component_light_theme() {
    let state = LoadingState::dots(Some("Fetching".to_string()));
    let view = LoadingViewComponent::new(state, Theme::light());
    assert_eq!(view.theme.colors.background, Theme::light().colors.background);
}

#[test]
fn test_loading_state_equality() {
    let a = LoadingState::spinner(Some("Loading".to_string()));
    let b = LoadingState::spinner(Some("Loading".to_string()));
    assert_eq!(a, b);

    let c = LoadingState::spinner(Some("Other".to_string()));
    assert_ne!(a, c);
}
