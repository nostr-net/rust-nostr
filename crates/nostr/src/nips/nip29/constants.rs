// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-29: Constants

use crate::Kind;

/// NIP-29 Moderation event kinds
pub const NIP29_MODERATION_KINDS: [u16; 7] = [9000, 9001, 9002, 9005, 9007, 9008, 9009];

/// NIP-29 Metadata event kinds (addressable)
pub const NIP29_METADATA_KINDS: [u16; 4] = [39000, 39001, 39002, 39003];

/// NIP-29 User-generated event kinds
pub const NIP29_USER_KINDS: [u16; 2] = [9021, 9022];

/// Special group ID for top-level relay-local discussion
pub const TOP_LEVEL_GROUP_ID: &str = "_";

/// Valid characters for group IDs: a-z, 0-9, -, _
pub const GROUP_ID_PATTERN: &str = r"^[a-z0-9_-]+$";

impl Kind {
    /// Check if kind is a NIP-29 moderation event
    ///
    /// Returns `true` for kinds 9000-9009 (excluding 9003, 9004, 9006 which are reserved)
    #[inline]
    pub fn is_group_moderation(&self) -> bool {
        NIP29_MODERATION_KINDS.binary_search(&self.as_u16()).is_ok()
    }

    /// Check if kind is a NIP-29 metadata event
    ///
    /// Returns `true` for kinds 39000-39003 (addressable)
    #[inline]
    pub fn is_group_metadata(&self) -> bool {
        NIP29_METADATA_KINDS.binary_search(&self.as_u16()).is_ok()
    }

    /// Check if kind is any NIP-29 related event
    #[inline]
    pub fn is_group_event(&self) -> bool {
        self.is_group_moderation()
            || self.is_group_metadata()
            || NIP29_USER_KINDS.binary_search(&self.as_u16()).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moderation_kinds() {
        assert!(Kind::from(9000).is_group_moderation());
        assert!(Kind::from(9001).is_group_moderation());
        assert!(Kind::from(9002).is_group_moderation());
        assert!(Kind::from(9005).is_group_moderation());
        assert!(Kind::from(9007).is_group_moderation());
        assert!(Kind::from(9008).is_group_moderation());
        assert!(Kind::from(9009).is_group_moderation());

        // Not moderation kinds
        assert!(!Kind::from(9003).is_group_moderation());
        assert!(!Kind::from(9004).is_group_moderation());
        assert!(!Kind::from(9006).is_group_moderation());
        assert!(!Kind::from(9010).is_group_moderation());
    }

    #[test]
    fn test_metadata_kinds() {
        assert!(Kind::from(39000).is_group_metadata());
        assert!(Kind::from(39001).is_group_metadata());
        assert!(Kind::from(39002).is_group_metadata());
        assert!(Kind::from(39003).is_group_metadata());

        assert!(!Kind::from(39004).is_group_metadata());
        assert!(!Kind::from(38999).is_group_metadata());
    }

    #[test]
    fn test_group_event() {
        // Moderation
        assert!(Kind::from(9000).is_group_event());

        // Metadata
        assert!(Kind::from(39000).is_group_event());

        // User events
        assert!(Kind::from(9021).is_group_event());
        assert!(Kind::from(9022).is_group_event());

        // Not group events
        assert!(!Kind::from(1).is_group_event());
        assert!(!Kind::from(4).is_group_event());
    }
}
