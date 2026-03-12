// T052: Exec confirmation dialog integration tests.
// Tests the ExecConfirmState, ConfirmDialogState::exec_into_pod(), and
// ConfirmDialogState::port_forward() factory methods.

use baeus_ui::components::confirm_dialog::*;
use baeus_ui::views::resource_detail::*;

// ========================================================================
// ExecConfirmState basics
// ========================================================================

#[test]
fn test_exec_confirm_state_new_starts_with_required() {
    let state = ExecConfirmState::new("nginx-abc", None);
    assert!(
        matches!(state.confirmation, ExecConfirmation::Required { .. }),
        "Expected Required confirmation"
    );
    assert!(!state.is_confirmed());
}

#[test]
fn test_exec_confirm_state_confirm_transitions_to_confirmed() {
    let mut state = ExecConfirmState::new("nginx-abc", None);
    assert!(!state.is_confirmed());
    state.confirm();
    assert!(state.is_confirmed());
    assert_eq!(state.confirmation, ExecConfirmation::Confirmed);
}

#[test]
fn test_exec_confirm_state_new_with_container_name() {
    let state = ExecConfirmState::new("nginx-abc", Some("app"));
    assert_eq!(state.pod_name, "nginx-abc");
    assert_eq!(state.container_name.as_deref(), Some("app"));
    if let ExecConfirmation::Required { message } = &state.confirmation {
        assert!(message.contains("nginx-abc"));
        assert!(message.contains("app"));
    } else {
        panic!("Expected Required");
    }
}

#[test]
fn test_exec_confirm_state_new_without_container_name() {
    let state = ExecConfirmState::new("nginx-abc", None);
    assert!(state.container_name.is_none());
    if let ExecConfirmation::Required { message } = &state.confirmation {
        assert!(message.contains("nginx-abc"));
        // Should not reference a container when none is specified
        assert!(!message.contains("container:"));
    } else {
        panic!("Expected Required");
    }
}

// ========================================================================
// ConfirmDialogState::exec_into_pod() factory
// ========================================================================

#[test]
fn test_exec_into_pod_title() {
    let dialog = ConfirmDialogState::exec_into_pod("web-pod", None);
    assert_eq!(dialog.title, "Exec into Pod");
}

#[test]
fn test_exec_into_pod_severity_is_warning() {
    let dialog = ConfirmDialogState::exec_into_pod("web-pod", None);
    assert_eq!(dialog.severity, DialogSeverity::Warning);
}

#[test]
fn test_exec_into_pod_confirm_label() {
    let dialog = ConfirmDialogState::exec_into_pod("web-pod", None);
    assert_eq!(dialog.confirm_label, "Open Terminal");
}

#[test]
fn test_exec_into_pod_has_resource_name() {
    let dialog = ConfirmDialogState::exec_into_pod("web-pod", None);
    assert_eq!(dialog.resource_name.as_deref(), Some("web-pod"));
}

#[test]
fn test_exec_into_pod_message_contains_pod_name() {
    let dialog = ConfirmDialogState::exec_into_pod("nginx-xyz", None);
    assert!(dialog.message.contains("nginx-xyz"));
}

#[test]
fn test_exec_into_pod_message_with_container() {
    let dialog = ConfirmDialogState::exec_into_pod("nginx-xyz", Some("sidecar"));
    assert!(dialog.message.contains("nginx-xyz"));
    assert!(dialog.message.contains("sidecar"));
}

#[test]
fn test_exec_into_pod_message_without_container() {
    let dialog = ConfirmDialogState::exec_into_pod("nginx-xyz", None);
    assert!(dialog.message.contains("nginx-xyz"));
    // When no container specified, should not mention a container name
    assert!(!dialog.message.contains("sidecar"));
}

#[test]
fn test_exec_into_pod_starts_hidden() {
    let dialog = ConfirmDialogState::exec_into_pod("web-pod", None);
    assert!(!dialog.visible);
}

#[test]
fn test_exec_into_pod_default_cancel_label() {
    let dialog = ConfirmDialogState::exec_into_pod("web-pod", None);
    assert_eq!(dialog.cancel_label, "Cancel");
}

// ========================================================================
// ConfirmDialogState::port_forward() factory
// ========================================================================

#[test]
fn test_port_forward_title() {
    let dialog = ConfirmDialogState::port_forward("web-svc", "Service", Some(8080), Some(80));
    assert_eq!(dialog.title, "Port Forward");
}

#[test]
fn test_port_forward_severity_is_info() {
    let dialog = ConfirmDialogState::port_forward("web-svc", "Service", Some(8080), Some(80));
    assert_eq!(dialog.severity, DialogSeverity::Info);
}

#[test]
fn test_port_forward_confirm_label() {
    let dialog = ConfirmDialogState::port_forward("web-svc", "Service", None, None);
    assert_eq!(dialog.confirm_label, "Start Forwarding");
}

#[test]
fn test_port_forward_has_resource_name() {
    let dialog = ConfirmDialogState::port_forward("web-svc", "Service", None, None);
    assert_eq!(dialog.resource_name.as_deref(), Some("web-svc"));
}

#[test]
fn test_port_forward_message_contains_resource() {
    let dialog = ConfirmDialogState::port_forward("nginx", "Pod", Some(8080), Some(80));
    assert!(dialog.message.contains("Pod/nginx"));
    assert!(dialog.message.contains("8080"));
    assert!(dialog.message.contains("80"));
}

#[test]
fn test_port_forward_message_auto_ports() {
    let dialog = ConfirmDialogState::port_forward("nginx", "Pod", None, None);
    assert!(dialog.message.contains("<auto>"));
}

#[test]
fn test_port_forward_starts_hidden() {
    let dialog = ConfirmDialogState::port_forward("web-svc", "Service", None, None);
    assert!(!dialog.visible);
}

#[test]
fn test_port_forward_default_cancel_label() {
    let dialog = ConfirmDialogState::port_forward("web-svc", "Service", None, None);
    assert_eq!(dialog.cancel_label, "Cancel");
}

#[test]
fn test_port_forward_message_with_only_remote_port() {
    let dialog = ConfirmDialogState::port_forward("nginx", "Pod", None, Some(80));
    assert!(dialog.message.contains("80"));
    assert!(dialog.message.contains("<auto>"));
}

#[test]
fn test_port_forward_message_with_only_local_port() {
    let dialog = ConfirmDialogState::port_forward("nginx", "Pod", Some(3000), None);
    assert!(dialog.message.contains("3000"));
    assert!(dialog.message.contains("<auto>"));
}

#[test]
fn test_port_forward_is_not_destructive() {
    let dialog = ConfirmDialogState::port_forward("web-svc", "Service", None, None);
    assert!(!dialog.is_destructive());
}

// ========================================================================
// Combined workflow: ExecConfirmState + ConfirmDialogState factory
// ========================================================================

#[test]
fn test_exec_confirm_then_dialog_factory_workflow() {
    // Simulate requesting exec -> creating the dialog -> showing it
    let exec_state = ExecConfirmState::new("nginx-pod", Some("app"));
    let mut dialog = ConfirmDialogState::exec_into_pod(
        &exec_state.pod_name,
        exec_state.container_name.as_deref(),
    );

    assert_eq!(dialog.title, "Exec into Pod");
    assert!(dialog.message.contains("nginx-pod"));
    assert!(dialog.message.contains("app"));
    assert!(!dialog.visible);

    dialog.show();
    assert!(dialog.visible);

    dialog.hide();
    assert!(!dialog.visible);
}
