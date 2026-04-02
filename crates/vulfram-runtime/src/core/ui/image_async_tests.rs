use super::*;

#[test]
fn cancel_only_marks_existing_pending_images() {
    let mut manager = UiImageAsyncManager::new();
    assert!(!manager.cancel(42));
    assert!(!manager.was_canceled(42));

    manager.pending.insert(42);
    assert!(manager.cancel(42));
    assert!(manager.was_canceled(42));
    assert!(!manager.was_canceled(42));
}

#[test]
fn pending_image_ids_reports_current_pending_set() {
    let mut manager = UiImageAsyncManager::new();
    manager.pending.insert(1);
    manager.pending.insert(8);
    assert_eq!(manager.pending_image_ids(), HashSet::from([1, 8]));
}
