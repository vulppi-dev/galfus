use super::*;

#[test]
fn cancel_only_marks_existing_pending_entries() {
    let mut manager = TextureAsyncManager::new();
    assert!(!manager.cancel(10));
    assert!(!manager.was_canceled(10));

    manager.pending.insert(10);
    assert!(manager.cancel(10));
    assert!(manager.was_canceled(10));
    assert!(!manager.was_canceled(10));
}
