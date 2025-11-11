// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-29: Errors

use alloc::string::String;
use core::fmt;

/// NIP-29 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid group ID format
    InvalidGroupId(String),
    /// Invalid privacy value
    InvalidPrivacy(String),
    /// Invalid access model value
    InvalidAccessModel(String),
    /// Missing required tag
    MissingRequiredTag(String),
    /// Invalid group identifier format (should be host'id)
    InvalidGroupIdentifier(String),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGroupId(msg) => write!(f, "Invalid group ID: {msg}"),
            Self::InvalidPrivacy(msg) => write!(f, "Invalid privacy value: {msg}"),
            Self::InvalidAccessModel(msg) => write!(f, "Invalid access model value: {msg}"),
            Self::MissingRequiredTag(tag) => write!(f, "Missing required tag: {tag}"),
            Self::InvalidGroupIdentifier(msg) => {
                write!(f, "Invalid group identifier format: {msg}")
            }
        }
    }
}
