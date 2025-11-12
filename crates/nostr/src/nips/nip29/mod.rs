// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-29: Relay-based Groups
//!
//! <https://github.com/nostr-protocol/nips/blob/master/29.md>
//!
//! This module provides types and utilities for working with NIP-29 relay-based groups.
//!
//! ## Overview
//!
//! NIP-29 defines a protocol for closed-membership groups that operate on Nostr relays
//! with relay-enforced moderation, roles, and permissions.
//!
//! ## Event Kinds
//!
//! ### Moderation Events (9000-9009)
//! - `9000`: Add/update user with roles
//! - `9001`: Remove user
//! - `9002`: Edit group metadata
//! - `9005`: Delete event
//! - `9007`: Create group
//! - `9008`: Delete group
//! - `9009`: Create invite
//!
//! ### User Events
//! - `9021`: Join request
//! - `9022`: Leave request
//!
//! ### Metadata Events (Addressable, 39000-39003)
//! - `39000`: Group metadata
//! - `39001`: Group admins list
//! - `39002`: Group members list
//! - `39003`: Group roles definition
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use nostr::prelude::*;
//! use nostr::nips::nip29::{GroupId, GroupMetadata, Privacy, AccessModel};
//!
//! # fn example() -> Result<()> {
//! let keys = Keys::generate();
//! let relay_url = Url::parse("wss://relay.example.com")?;
//!
//! // Create a group
//! let group_id = GroupId::new(relay_url, "rust-devs".to_string())?;
//! let metadata = GroupMetadata {
//!     name: Some("Rust Developers".into()),
//!     about: Some("A group for Rust enthusiasts".into()),
//!     privacy: Privacy::Public,
//!     closed: AccessModel::Closed,
//!     ..Default::default()
//! };
//!
//! let event = EventBuilder::group_create(group_id.clone(), metadata)
//!     .sign_with_keys(&keys)?;
//!
//! // Send a group message
//! let msg = EventBuilder::group_message(group_id, "Hello everyone!")
//!     .sign_with_keys(&keys)?;
//! # Ok(())
//! # }
//! ```

pub mod constants;
pub mod error;
pub mod types;

pub use self::constants::*;
pub use self::error::Error;
pub use self::types::*;
