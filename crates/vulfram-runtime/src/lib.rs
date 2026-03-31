#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeferredCommandKey {
    pub command_id: u64,
    pub command_signature: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeferredCommandMeta {
    pub first_frame: u64,
    pub attempts: u32,
    pub next_retry_frame: u64,
    pub last_reason: String,
}

pub const DEFER_MAX_ATTEMPTS: u32 = 120;
pub const DEFER_MAX_AGE_FRAMES: u64 = 1200;
pub const DEFER_BACKOFF_MAX_EXP: u32 = 6;

pub fn defer_backoff_frames(attempts: u32) -> u64 {
    let exp = attempts.saturating_sub(1).min(DEFER_BACKOFF_MAX_EXP);
    1_u64 << exp
}

pub fn should_drop_deferred(attempts: u32, age_frames: u64) -> bool {
    attempts >= DEFER_MAX_ATTEMPTS || age_frames >= DEFER_MAX_AGE_FRAMES
}

#[cfg(test)]
mod tests {
    use super::{
        DEFER_MAX_AGE_FRAMES, DEFER_MAX_ATTEMPTS, defer_backoff_frames, should_drop_deferred,
    };

    #[test]
    fn defer_backoff_caps_at_sixty_four_frames() {
        assert_eq!(defer_backoff_frames(1), 1);
        assert_eq!(defer_backoff_frames(2), 2);
        assert_eq!(defer_backoff_frames(3), 4);
        assert_eq!(defer_backoff_frames(7), 64);
        assert_eq!(defer_backoff_frames(100), 64);
    }

    #[test]
    fn deferred_drop_policy_uses_attempts_or_age() {
        assert!(should_drop_deferred(DEFER_MAX_ATTEMPTS, 0));
        assert!(should_drop_deferred(0, DEFER_MAX_AGE_FRAMES));
        assert!(!should_drop_deferred(
            DEFER_MAX_ATTEMPTS - 1,
            DEFER_MAX_AGE_FRAMES - 1
        ));
    }
}
