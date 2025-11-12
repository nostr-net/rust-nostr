// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-29: Types

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use crate::event::tag::TagKind;
use crate::{PublicKey, Tag, Url};

use super::constants::TOP_LEVEL_GROUP_ID;
use super::Error;

/// Group identifier in format: `<relay-url>'<group-id>`
///
/// Group IDs must contain only characters: a-z, 0-9, -, _
/// The special ID "_" represents a top-level relay-local discussion group.
///
/// # Example
/// ```rust,no_run
/// use nostr::nips::nip29::GroupId;
/// use nostr::Url;
///
/// let url = Url::parse("wss://relay.example.com").unwrap();
/// let group_id = GroupId::new(url, "rust-devs".to_string()).unwrap();
/// assert_eq!(group_id.to_string(), "wss://relay.example.com'rust-devs");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupId {
    /// Relay URL where the group exists
    pub relay_url: Url,
    /// Group identifier
    pub id: String,
}

impl GroupId {
    /// Create a new group identifier
    ///
    /// Returns error if the ID contains invalid characters
    pub fn new(relay_url: Url, id: String) -> Result<Self, Error> {
        Self::validate_id(&id)?;
        Ok(Self { relay_url, id })
    }

    /// Validate group ID format
    fn validate_id(id: &str) -> Result<(), Error> {
        if id.is_empty() {
            return Err(Error::InvalidGroupId("Group ID cannot be empty".into()));
        }

        // Check if all characters are valid (a-z, 0-9, -, _)
        if !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
            return Err(Error::InvalidGroupId(
                "Group ID must contain only: a-z, 0-9, -, _".into(),
            ));
        }

        Ok(())
    }

    /// Check if this is the top-level group
    #[inline]
    pub fn is_top_level(&self) -> bool {
        self.id == TOP_LEVEL_GROUP_ID
    }

    /// Convert to tag value (format: `relay'id`)
    pub fn to_tag_value(&self) -> String {
        let url_str = self.relay_url.as_str().trim_end_matches('/');
        format!("{}'{}", url_str, self.id)
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_tag_value())
    }
}

impl FromStr for GroupId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Find the delimiter
        let parts: Vec<&str> = s.split('\'').collect();
        if parts.len() != 2 {
            return Err(Error::InvalidGroupIdentifier(
                "Expected format: relay-url'group-id".into(),
            ));
        }

        let relay_url = Url::parse(parts[0])
            .map_err(|e| Error::InvalidGroupIdentifier(format!("Invalid relay URL: {}", e)))?;
        let id = parts[1].to_string();

        Self::new(relay_url, id)
    }
}

/// Group privacy setting
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Privacy {
    /// Public - can be read by external users
    #[default]
    Public,
    /// Private - only visible to members
    Private,
}

impl Privacy {
    /// Convert to string representation
    pub fn as_str(&self) -> &str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }
}

impl fmt::Display for Privacy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Privacy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            _ => Err(Error::InvalidPrivacy(format!(
                "Expected 'public' or 'private', got: {}",
                s
            ))),
        }
    }
}

/// Group access model
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccessModel {
    /// Open - join requests automatically approved
    #[default]
    Open,
    /// Closed - join requests require approval
    Closed,
}

impl AccessModel {
    /// Convert to string representation
    pub fn as_str(&self) -> &str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }
}

impl fmt::Display for AccessModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AccessModel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            _ => Err(Error::InvalidAccessModel(format!(
                "Expected 'open' or 'closed', got: {}",
                s
            ))),
        }
    }
}

/// Group metadata
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupMetadata {
    /// Display name
    pub name: Option<String>,
    /// Description
    pub about: Option<String>,
    /// Group image URL
    pub picture: Option<Url>,
    /// Privacy setting
    pub privacy: Privacy,
    /// Access model
    pub closed: AccessModel,
}

impl From<GroupMetadata> for Vec<Tag> {
    fn from(metadata: GroupMetadata) -> Self {
        let mut tags = Vec::new();

        if let Some(name) = metadata.name {
            tags.push(Tag::custom(TagKind::Name, [name]));
        }

        if let Some(about) = metadata.about {
            tags.push(Tag::custom(TagKind::Description, [about]));
        }

        if let Some(picture) = metadata.picture {
            tags.push(Tag::custom(TagKind::Image, [picture.to_string()]));
        }

        tags.push(Tag::custom(TagKind::Custom("privacy".into()), [metadata.privacy.as_str()]));
        tags.push(Tag::custom(TagKind::Custom("closed".into()), [metadata.closed.as_str()]));

        tags
    }
}

/// Role definition
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Role {
    /// Role name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
}

impl Role {
    /// Create a new role
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            description: None,
        }
    }

    /// Create a new role with description
    pub fn with_description<S, D>(name: S, description: D) -> Self
    where
        S: Into<String>,
        D: Into<String>,
    {
        Self {
            name: name.into(),
            description: Some(description.into()),
        }
    }
}

/// Group roles definition
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupRoles {
    /// List of role definitions
    pub roles: Vec<Role>,
}

impl GroupRoles {
    /// Create new empty roles
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a role
    pub fn add_role(mut self, role: Role) -> Self {
        self.roles.push(role);
        self
    }
}

impl From<GroupRoles> for Vec<Tag> {
    fn from(roles: GroupRoles) -> Self {
        roles
            .roles
            .into_iter()
            .map(|role| {
                if let Some(desc) = role.description {
                    Tag::custom(TagKind::Custom("role".into()), [role.name, desc])
                } else {
                    Tag::custom(TagKind::Custom("role".into()), [role.name])
                }
            })
            .collect()
    }
}

/// Group admin with assigned roles
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupAdmin {
    /// Public key of admin
    pub public_key: PublicKey,
    /// Assigned roles
    pub roles: Vec<String>,
}

impl GroupAdmin {
    /// Create new admin with roles
    pub fn new(public_key: PublicKey, roles: Vec<String>) -> Self {
        Self { public_key, roles }
    }
}

/// Group admins list
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupAdmins {
    /// List of admins
    pub admins: Vec<GroupAdmin>,
}

impl GroupAdmins {
    /// Create new empty admins list
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an admin
    pub fn add_admin(mut self, admin: GroupAdmin) -> Self {
        self.admins.push(admin);
        self
    }
}

impl From<GroupAdmins> for Vec<Tag> {
    fn from(admins: GroupAdmins) -> Self {
        let mut tags = Vec::new();

        for admin in admins.admins {
            // Add p tag for public key
            tags.push(Tag::public_key(admin.public_key));

            // Add role tags
            for role in admin.roles {
                tags.push(Tag::custom(TagKind::Custom("role".into()), [role]));
            }
        }

        tags
    }
}

/// Group members list
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupMembers {
    /// List of member public keys
    pub members: Vec<PublicKey>,
}

impl GroupMembers {
    /// Create new empty members list
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a member
    pub fn add_member(mut self, public_key: PublicKey) -> Self {
        self.members.push(public_key);
        self
    }
}

impl From<GroupMembers> for Vec<Tag> {
    fn from(members: GroupMembers) -> Self {
        members
            .members
            .into_iter()
            .map(Tag::public_key)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_id_valid() {
        let url = Url::parse("wss://relay.example.com").unwrap();
        let group_id = GroupId::new(url.clone(), "rust-devs".to_string()).unwrap();
        assert_eq!(group_id.id, "rust-devs");
        assert_eq!(group_id.relay_url, url);
        assert!(!group_id.is_top_level());
    }

    #[test]
    fn test_group_id_top_level() {
        let url = Url::parse("wss://relay.example.com").unwrap();
        let group_id = GroupId::new(url, "_".to_string()).unwrap();
        assert!(group_id.is_top_level());
    }

    #[test]
    fn test_group_id_invalid() {
        let url = Url::parse("wss://relay.example.com").unwrap();

        // Invalid characters
        assert!(GroupId::new(url.clone(), "Rust Devs".to_string()).is_err());
        assert!(GroupId::new(url.clone(), "rust@devs".to_string()).is_err());
        assert!(GroupId::new(url.clone(), "".to_string()).is_err());

        // Valid
        assert!(GroupId::new(url.clone(), "rust-devs-123_test".to_string()).is_ok());
    }

    #[test]
    fn test_group_id_from_str() {
        let group_id = GroupId::from_str("wss://relay.example.com'rust-devs").unwrap();
        assert_eq!(group_id.id, "rust-devs");
        // URL normalization adds trailing slash
        assert_eq!(group_id.relay_url.as_str(), "wss://relay.example.com/");

        // Invalid format
        assert!(GroupId::from_str("no-delimiter").is_err());
        assert!(GroupId::from_str("too'many'parts").is_err());
    }

    #[test]
    fn test_group_id_to_string() {
        let url = Url::parse("wss://relay.example.com").unwrap();
        let group_id = GroupId::new(url, "rust-devs".to_string()).unwrap();
        assert_eq!(group_id.to_string(), "wss://relay.example.com'rust-devs");
    }

    #[test]
    fn test_privacy() {
        assert_eq!(Privacy::Public.as_str(), "public");
        assert_eq!(Privacy::Private.as_str(), "private");

        assert_eq!(Privacy::from_str("public").unwrap(), Privacy::Public);
        assert_eq!(Privacy::from_str("private").unwrap(), Privacy::Private);
        assert_eq!(Privacy::from_str("PUBLIC").unwrap(), Privacy::Public);
        assert!(Privacy::from_str("invalid").is_err());
    }

    #[test]
    fn test_access_model() {
        assert_eq!(AccessModel::Open.as_str(), "open");
        assert_eq!(AccessModel::Closed.as_str(), "closed");

        assert_eq!(AccessModel::from_str("open").unwrap(), AccessModel::Open);
        assert_eq!(AccessModel::from_str("closed").unwrap(), AccessModel::Closed);
        assert!(AccessModel::from_str("invalid").is_err());
    }

    #[test]
    fn test_group_metadata_tags() {
        let metadata = GroupMetadata {
            name: Some("Rust Developers".into()),
            about: Some("A group for Rust enthusiasts".into()),
            picture: Some(Url::parse("https://example.com/image.png").unwrap()),
            privacy: Privacy::Public,
            closed: AccessModel::Closed,
        };

        let tags: Vec<Tag> = metadata.into();
        assert_eq!(tags.len(), 5);
    }

    #[test]
    fn test_role() {
        let role = Role::new("admin");
        assert_eq!(role.name, "admin");
        assert_eq!(role.description, None);

        let role = Role::with_description("moderator", "Can moderate messages");
        assert_eq!(role.name, "moderator");
        assert_eq!(role.description, Some("Can moderate messages".into()));
    }

    #[test]
    fn test_group_roles_tags() {
        let roles = GroupRoles::new()
            .add_role(Role::new("admin"))
            .add_role(Role::with_description("moderator", "Can moderate"));

        let tags: Vec<Tag> = roles.into();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_group_admins_tags() {
        let pk1 = PublicKey::from_slice(&[0x01; 32]).unwrap();
        let pk2 = PublicKey::from_slice(&[0x02; 32]).unwrap();

        let admins = GroupAdmins::new()
            .add_admin(GroupAdmin::new(pk1, vec!["admin".into()]))
            .add_admin(GroupAdmin::new(pk2, vec!["moderator".into(), "member".into()]));

        let tags: Vec<Tag> = admins.into();
        // 2 p tags + 1 role for pk1 + 2 roles for pk2 = 5 tags
        assert_eq!(tags.len(), 5);
    }

    #[test]
    fn test_group_members_tags() {
        let pk1 = PublicKey::from_slice(&[0x01; 32]).unwrap();
        let pk2 = PublicKey::from_slice(&[0x02; 32]).unwrap();

        let members = GroupMembers::new()
            .add_member(pk1)
            .add_member(pk2);

        let tags: Vec<Tag> = members.into();
        assert_eq!(tags.len(), 2);
    }
}
