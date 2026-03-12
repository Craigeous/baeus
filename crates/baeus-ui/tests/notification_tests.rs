// T089: Notification/Toast system integration tests

use baeus_ui::components::notification::*;
use baeus_ui::theme::Theme;

// ===========================================================================
// NotificationLevel
// ===========================================================================

#[test]
fn test_notification_level_label_success() {
    assert_eq!(NotificationLevel::Success.label(), "Success");
}

#[test]
fn test_notification_level_label_error() {
    assert_eq!(NotificationLevel::Error.label(), "Error");
}

#[test]
fn test_notification_level_label_warning() {
    assert_eq!(NotificationLevel::Warning.label(), "Warning");
}

#[test]
fn test_notification_level_label_info() {
    assert_eq!(NotificationLevel::Info.label(), "Info");
}

#[test]
fn test_notification_level_icon_success() {
    assert_eq!(NotificationLevel::Success.icon(), "checkmark");
}

#[test]
fn test_notification_level_icon_error() {
    assert_eq!(NotificationLevel::Error.icon(), "x-circle");
}

#[test]
fn test_notification_level_icon_warning() {
    assert_eq!(NotificationLevel::Warning.icon(), "warning-triangle");
}

#[test]
fn test_notification_level_icon_info() {
    assert_eq!(NotificationLevel::Info.icon(), "info-circle");
}

#[test]
fn test_notification_level_labels_all_unique() {
    let levels = [
        NotificationLevel::Success,
        NotificationLevel::Error,
        NotificationLevel::Warning,
        NotificationLevel::Info,
    ];
    let labels: Vec<&str> = levels.iter().map(|l| l.label()).collect();
    for (i, a) in labels.iter().enumerate() {
        for (j, b) in labels.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn test_notification_level_icons_all_unique() {
    let levels = [
        NotificationLevel::Success,
        NotificationLevel::Error,
        NotificationLevel::Warning,
        NotificationLevel::Info,
    ];
    let icons: Vec<&str> = levels.iter().map(|l| l.icon()).collect();
    for (i, a) in icons.iter().enumerate() {
        for (j, b) in icons.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn test_notification_level_serialization_roundtrip() {
    for level in [
        NotificationLevel::Success,
        NotificationLevel::Error,
        NotificationLevel::Warning,
        NotificationLevel::Info,
    ] {
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: NotificationLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }
}

// ===========================================================================
// NotificationState::new()
// ===========================================================================

#[test]
fn test_new_state_empty() {
    let state = NotificationState::new();
    assert!(state.notifications.is_empty());
    assert_eq!(state.count(), 0);
}

#[test]
fn test_new_state_max_visible() {
    let state = NotificationState::new();
    assert_eq!(state.max_visible, 5);
}

// ===========================================================================
// NotificationState::push()
// ===========================================================================

#[test]
fn test_push_adds_notification() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Info, "Test", None, None);
    assert_eq!(state.notifications.len(), 1);
    assert_eq!(state.count(), 1);
}

#[test]
fn test_push_sets_fields() {
    let mut state = NotificationState::new();
    state.push(
        NotificationLevel::Error,
        "Deploy failed",
        Some("Timeout after 30s".to_string()),
        Some(5000),
    );
    let n = &state.notifications[0];
    assert_eq!(n.level, NotificationLevel::Error);
    assert_eq!(n.title, "Deploy failed");
    assert_eq!(n.message.as_deref(), Some("Timeout after 30s"));
    assert_eq!(n.auto_dismiss_ms, Some(5000));
    assert!(!n.dismissed);
    assert!(n.created_at > 0);
}

#[test]
fn test_push_generates_unique_ids() {
    let mut state = NotificationState::new();
    let id1 = state.push(NotificationLevel::Info, "A", None, None);
    let id2 = state.push(NotificationLevel::Info, "B", None, None);
    assert_ne!(id1, id2);
}

#[test]
fn test_push_returns_id() {
    let mut state = NotificationState::new();
    let id = state.push(NotificationLevel::Success, "Done", None, None);
    assert!(!id.is_empty());
    assert_eq!(state.notifications[0].id, id);
}

// ===========================================================================
// NotificationState::dismiss()
// ===========================================================================

#[test]
fn test_dismiss_marks_dismissed() {
    let mut state = NotificationState::new();
    let id = state.push(NotificationLevel::Info, "Test", None, None);
    state.dismiss(&id);
    assert!(state.notifications[0].dismissed);
}

#[test]
fn test_dismiss_nonexistent_id_is_noop() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Info, "Test", None, None);
    state.dismiss("nonexistent-id");
    assert!(!state.notifications[0].dismissed);
}

#[test]
fn test_dismiss_reduces_count() {
    let mut state = NotificationState::new();
    let id = state.push(NotificationLevel::Info, "A", None, None);
    state.push(NotificationLevel::Info, "B", None, None);
    assert_eq!(state.count(), 2);
    state.dismiss(&id);
    assert_eq!(state.count(), 1);
}

// ===========================================================================
// NotificationState::dismiss_all()
// ===========================================================================

#[test]
fn test_dismiss_all() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Info, "A", None, None);
    state.push(NotificationLevel::Error, "B", None, None);
    state.push(NotificationLevel::Warning, "C", None, None);
    state.dismiss_all();
    assert_eq!(state.count(), 0);
    assert!(state.notifications.iter().all(|n| n.dismissed));
}

#[test]
fn test_dismiss_all_empty_state_is_noop() {
    let mut state = NotificationState::new();
    state.dismiss_all();
    assert_eq!(state.count(), 0);
}

// ===========================================================================
// NotificationState::visible()
// ===========================================================================

#[test]
fn test_visible_returns_undismissed() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Info, "A", None, None);
    let id2 = state.push(NotificationLevel::Error, "B", None, None);
    state.push(NotificationLevel::Warning, "C", None, None);
    state.dismiss(&id2);
    let visible = state.visible();
    assert_eq!(visible.len(), 2);
    assert!(visible.iter().all(|n| !n.dismissed));
}

#[test]
fn test_visible_limited_to_max_visible() {
    let mut state = NotificationState::new();
    for i in 0..10 {
        state.push(NotificationLevel::Info, format!("Notification {i}"), None, None);
    }
    let visible = state.visible();
    assert_eq!(visible.len(), 5);
}

#[test]
fn test_visible_most_recent_first() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Info, "First", None, None);
    state.push(NotificationLevel::Info, "Second", None, None);
    state.push(NotificationLevel::Info, "Third", None, None);
    let visible = state.visible();
    assert_eq!(visible[0].title, "Third");
    assert_eq!(visible[1].title, "Second");
    assert_eq!(visible[2].title, "First");
}

#[test]
fn test_visible_empty_when_all_dismissed() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Info, "A", None, None);
    state.dismiss_all();
    assert!(state.visible().is_empty());
}

// ===========================================================================
// NotificationState::remove_dismissed()
// ===========================================================================

#[test]
fn test_remove_dismissed_garbage_collects() {
    let mut state = NotificationState::new();
    let id1 = state.push(NotificationLevel::Info, "A", None, None);
    state.push(NotificationLevel::Error, "B", None, None);
    state.dismiss(&id1);
    state.remove_dismissed();
    assert_eq!(state.notifications.len(), 1);
    assert_eq!(state.notifications[0].title, "B");
}

#[test]
fn test_remove_dismissed_keeps_undismissed() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Info, "A", None, None);
    state.push(NotificationLevel::Info, "B", None, None);
    state.remove_dismissed();
    assert_eq!(state.notifications.len(), 2);
}

#[test]
fn test_remove_dismissed_clears_all_when_all_dismissed() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Info, "A", None, None);
    state.push(NotificationLevel::Info, "B", None, None);
    state.dismiss_all();
    state.remove_dismissed();
    assert!(state.notifications.is_empty());
}

// ===========================================================================
// NotificationState::has_errors()
// ===========================================================================

#[test]
fn test_has_errors_false_when_empty() {
    let state = NotificationState::new();
    assert!(!state.has_errors());
}

#[test]
fn test_has_errors_true_with_error() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Error, "Crash", None, None);
    assert!(state.has_errors());
}

#[test]
fn test_has_errors_false_with_only_non_errors() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Info, "A", None, None);
    state.push(NotificationLevel::Warning, "B", None, None);
    state.push(NotificationLevel::Success, "C", None, None);
    assert!(!state.has_errors());
}

#[test]
fn test_has_errors_false_after_dismissing_error() {
    let mut state = NotificationState::new();
    let id = state.push(NotificationLevel::Error, "Crash", None, None);
    state.dismiss(&id);
    assert!(!state.has_errors());
}

// ===========================================================================
// NotificationState::count()
// ===========================================================================

#[test]
fn test_count_zero_when_empty() {
    let state = NotificationState::new();
    assert_eq!(state.count(), 0);
}

#[test]
fn test_count_reflects_pushes() {
    let mut state = NotificationState::new();
    state.push(NotificationLevel::Info, "A", None, None);
    state.push(NotificationLevel::Info, "B", None, None);
    assert_eq!(state.count(), 2);
}

#[test]
fn test_count_excludes_dismissed() {
    let mut state = NotificationState::new();
    let id = state.push(NotificationLevel::Info, "A", None, None);
    state.push(NotificationLevel::Info, "B", None, None);
    state.dismiss(&id);
    assert_eq!(state.count(), 1);
}

// ===========================================================================
// NotificationViewComponent
// ===========================================================================

#[test]
fn test_view_component_level_color_success() {
    let state = NotificationState::new();
    let view = NotificationViewComponent::new(state, Theme::dark());
    let color = view.level_color(&NotificationLevel::Success);
    assert_eq!(color, Theme::dark().colors.success);
}

#[test]
fn test_view_component_level_color_error() {
    let state = NotificationState::new();
    let view = NotificationViewComponent::new(state, Theme::dark());
    let color = view.level_color(&NotificationLevel::Error);
    assert_eq!(color, Theme::dark().colors.error);
}

#[test]
fn test_view_component_level_color_warning() {
    let state = NotificationState::new();
    let view = NotificationViewComponent::new(state, Theme::dark());
    let color = view.level_color(&NotificationLevel::Warning);
    assert_eq!(color, Theme::dark().colors.warning);
}

#[test]
fn test_view_component_level_color_info() {
    let state = NotificationState::new();
    let view = NotificationViewComponent::new(state, Theme::dark());
    let color = view.level_color(&NotificationLevel::Info);
    assert_eq!(color, Theme::dark().colors.info);
}

#[test]
fn test_view_component_level_colors_differ_by_level() {
    let state = NotificationState::new();
    let view = NotificationViewComponent::new(state, Theme::dark());
    let colors: Vec<_> = [
        NotificationLevel::Success,
        NotificationLevel::Error,
        NotificationLevel::Warning,
        NotificationLevel::Info,
    ]
    .iter()
    .map(|l| view.level_color(l))
    .collect();

    // Success, Error, Warning should all be distinct
    assert_ne!(colors[0], colors[1]);
    assert_ne!(colors[1], colors[2]);
    assert_ne!(colors[0], colors[2]);
}

#[test]
fn test_view_component_light_theme() {
    let state = NotificationState::new();
    let view = NotificationViewComponent::new(state, Theme::light());
    let color = view.level_color(&NotificationLevel::Success);
    assert_eq!(color, Theme::light().colors.success);
}

#[test]
fn test_notification_serialization_roundtrip() {
    let notification = Notification {
        id: "test-id".to_string(),
        level: NotificationLevel::Warning,
        title: "Disk space low".to_string(),
        message: Some("Only 10% remaining".to_string()),
        auto_dismiss_ms: Some(10000),
        dismissed: false,
        created_at: 1700000000,
    };
    let json = serde_json::to_string(&notification).unwrap();
    let deserialized: Notification = serde_json::from_str(&json).unwrap();
    assert_eq!(notification, deserialized);
}
