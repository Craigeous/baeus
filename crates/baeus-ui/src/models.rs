//! Reactive state models that bridge domain state to GPUI rendering.
//!
//! This module provides `gpui::Entity<T>` wrappers for core state types,
//! enabling Tokio async updates to trigger GPUI re-renders.

use gpui::{App, AppContext as _, Entity};

/// Helper to create a new entity from a value within an App context.
pub fn create_model<T: 'static>(cx: &mut App, value: T) -> Entity<T> {
    cx.new(|_cx| value)
}
