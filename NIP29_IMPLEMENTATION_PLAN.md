# NIP-29 Relay-Based Groups Implementation Plan for rust-nostr

**Date:** 2025-11-11
**Status:** Planning Phase
**Target:** Add comprehensive NIP-29 support to rust-nostr SDK

---

## Executive Summary

This document provides a complete implementation plan for adding NIP-29 (Relay-based Groups) support to the rust-nostr SDK. NIP-29 defines a protocol for closed-membership groups that operate on Nostr relays with relay-enforced moderation, roles, and permissions.

**Key Implementation Areas:**
1. Core protocol types and data structures
2. Event kinds (9000-9022, 39000-39003)
3. Tag standards for groups
4. EventBuilder methods for group operations
5. Client SDK integration
6. Database/storage support
7. Testing and examples

---

## Table of Contents

1. [NIP-29 Specification Overview](#nip-29-specification-overview)
2. [Reference Implementations Analysis](#reference-implementations-analysis)
3. [Rust-Nostr Architecture Review](#rust-nostr-architecture-review)
4. [Implementation Design](#implementation-design)
5. [Detailed Implementation Plan](#detailed-implementation-plan)
6. [API Design](#api-design)
7. [Testing Strategy](#testing-strategy)
8. [Migration and Compatibility](#migration-and-compatibility)
9. [Timeline and Milestones](#timeline-and-milestones)

---

## 1. NIP-29 Specification Overview

### 1.1 Core Concepts

**Group Identification:**
- Format: `<host>'<group-id>` (e.g., `relay.example.com'mygroup`)
- Group IDs: alphanumeric + `-_` characters only
- Special ID `_`: top-level relay-local discussion group

**Group Types:**
- **Managed Groups**: Relay-enforced rules, admins, moderation
- **Unmanaged Groups**: Open groups with no relay enforcement (everybody is a member)

**Access Models:**
- **Open**: Auto-approve join requests
- **Closed**: Require approval or invite codes

### 1.2 Event Kinds

#### User-Generated Events

| Kind | Name | Description | Required Tags |
|------|------|-------------|---------------|
| 9 | Chat Message | Regular group messages | `h` (group-id) |
| 9021 | Join Request | Request to join group | `h` (group-id) |
| 9022 | Leave Request | Request to leave group | `h` (group-id) |

#### Moderation Events (9000-9020)

| Kind | Action | Description | Parameters |
|------|--------|-------------|------------|
| 9000 | put-user | Add/update user + roles | `p` tag + optional roles |
| 9001 | remove-user | Remove user from group | `p` tag |
| 9002 | edit-metadata | Update group metadata | Metadata fields |
| 9005 | delete-event | Delete specific event | `e` tag |
| 9007 | create-group | Create new group | Group details |
| 9008 | delete-group | Delete entire group | - |
| 9009 | create-invite | Generate invite code | - |

#### Metadata Events (Addressable, 30000-40000 range)

| Kind | Name | Description | d-tag |
|------|------|-------------|-------|
| 39000 | Group Metadata | Display name, about, picture, privacy | group-id |
| 39001 | Group Admins | List of admins with roles | group-id |
| 39002 | Group Members | Member roster (optional visibility) | group-id |
| 39003 | Group Roles | Role definitions | group-id |

### 1.3 Tags

**Required Tags:**
- `h`: Group ID (required on all group events)
- `d`: Identifier for addressable events
- `p`: Public key (for user operations)
- `e`: Event ID (for references/deletions)

**Timeline Tags:**
- `previous`: Reference to recent events (first 8 chars)
- Prevents out-of-context propagation
- Recommended: include at least 3 recent events

**Group-Specific:**
- `name`: Group display name
- `about`: Group description
- `picture`: Group image URL
- `privacy`: `public` or `private`
- `closed`: `open` or `closed`

### 1.4 Roles and Permissions

**Role System:**
- Custom role names defined per-group
- Stored in kind 39003
- Applied via kind 39001 (admins list)
- Permissions enforced by relay

**Common Roles:**
- Admin, Moderator, Member (relay-specific)
- Multiple roles per user supported

---

## 2. Reference Implementations Analysis

### 2.1 nostr-groups (TypeScript/React)

**Architecture:**
- Frontend: React + TypeScript + Vite
- Nostr: NDK (Nostr Dev Kit) + nostr-hooks
- State: Zustand for state management
- UI: Tailwind + Shadcn components

**Key Features Implemented:**
- All event kinds (9000-9022, 39000-39002)
- Group creation/deletion
- User management (add/remove)
- Message reactions (kind 7)
- Join/leave requests
- Real-time WebSocket updates
- Dark mode, responsive design

**Pattern Insights:**
- Relay-centric architecture
- Event-driven state updates
- Cached metadata for performance
- Progressive enhancement approach

### 2.2 go-nostr (Go)

**Architecture:**
- Minimal, focused implementation
- Type-safe approach with custom types

**Key Patterns:**
```go
type Role struct {
    Name        string
    Description string
}

type KindRange []int

const (
    ModerationEventKinds = KindRange{9000, 9001, 9002, 9005, 9007, 9008, 9009}
    MetadataEventKinds = KindRange{39000, 39001, 39002, 39003}
)

func (kr KindRange) Includes(kind int) bool {
    // Binary search for efficiency
}
```

**Insights:**
- Simple type aliases for clarity
- Efficient kind checking with binary search
- Separation of moderation vs metadata events
- Lightweight API surface

---

## 3. Rust-Nostr Architecture Review

### 3.1 Project Structure

```
rust-nostr/
├── crates/
│   ├── nostr/           # Core protocol library
│   │   ├── src/
│   │   │   ├── event/   # Event, Kind, Builder, Tags
│   │   │   ├── nips/    # NIP implementations (45 files)
│   │   │   ├── types/   # PublicKey, EventId, etc.
│   │   │   └── ...
│   ├── nostr-sdk/       # High-level client SDK
│   │   ├── src/
│   │   │   ├── client/  # Client, RelayPool
│   │   │   ├── database/# Storage backends
│   │   │   └── ...
│   └── ...
```

### 3.2 Event System Architecture

**Kind Enum (crates/nostr/src/event/kind.rs):**
```rust
// Declarative macro for defining kinds
kind_variants! {
    ChannelCreation => 40, "Channel Creation", "NIP-28",
    ChannelMetadata => 41, "Channel Metadata", "NIP-28",
    ChannelMessage => 42, "Channel Message", "NIP-28",
    // ... 100+ variants
}

// Ranges
const REGULAR_RANGE: Range<u16> = 1_000..10_000;
const REPLACEABLE_RANGE: Range<u16> = 10_000..20_000;
const EPHEMERAL_RANGE: Range<u16> = 20_000..30_000;
const ADDRESSABLE_RANGE: Range<u16> = 30_000..40_000;
```

**Event Structure:**
```rust
pub struct Event {
    pub id: EventId,
    pub pubkey: PublicKey,
    pub created_at: Timestamp,
    pub kind: Kind,
    pub tags: Vec<Tag>,
    pub content: String,
    pub sig: Signature,
}
```

### 3.3 Tag System (3-Layer Architecture)

**Layer 1 - Raw Tags:**
```rust
pub struct Tag(Vec<String>);  // Flexible, parseable
```

**Layer 2 - Single Letter Tags:**
```rust
pub enum SingleLetterTag {
    E, P, A, D, H, T, // ... a-z, A-Z
}
```

**Layer 3 - Standardized Tags:**
```rust
pub enum TagStandard {
    Event { event_id, relay_url, marker, public_key, uppercase },
    PublicKey { public_key, relay_url, alias, uppercase },
    Hashtag(String),
    Identifier(String),
    Coordinate { coordinate, relay_url, uppercase },
    // ... 200+ variants
}
```

### 3.4 EventBuilder Pattern

**Example from NIP-28 (Channels):**
```rust
impl EventBuilder {
    pub fn channel_metadata(
        channel_id: EventId,
        relay_url: Option<RelayUrl>,
        metadata: &Metadata,
    ) -> Self {
        Self::new(Kind::ChannelMetadata, metadata.as_json())
            .tags([Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: channel_id,
                relay_url,
                marker: None,
                public_key: None,
                uppercase: false,
            })])
    }

    pub fn channel_msg<S>(channel_id: EventId, relay_url: RelayUrl, content: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::ChannelMessage, content)
            .tags([Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: channel_id,
                relay_url: Some(relay_url),
                marker: Some(Marker::Root),
                public_key: None,
                uppercase: false,
            })])
    }
}
```

### 3.5 NIP Implementation Pattern (NIP-51 Example)

**Data Structures:**
```rust
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MuteList {
    pub public_keys: Vec<PublicKey>,
    pub hashtags: Vec<String>,
    pub event_ids: Vec<EventId>,
    pub words: Vec<String>,
}

impl From<MuteList> for Vec<Tag> {
    fn from(list: MuteList) -> Self {
        let mut tags = Vec::with_capacity(/* ... */);
        tags.extend(list.public_keys.into_iter().map(Tag::public_key));
        tags.extend(list.hashtags.into_iter().map(Tag::hashtag));
        // ...
        tags
    }
}
```

### 3.6 Existing Group Features

**Currently Implemented:**
- **NIP-28**: Public channels (kinds 40-44)
- **NIP-51**: Lists including Communities (10004), PublicChats (10005), SimpleGroups (10009)
- **Contact Lists**: Kind 3

**Missing:** NIP-29 relay-based groups ← **THIS IS WHAT WE NEED TO ADD**

---

## 4. Implementation Design

### 4.1 Design Principles

Following rust-nostr patterns:

1. **Type Safety**: Strong typing for all group concepts
2. **Builder Pattern**: Fluent APIs for event creation
3. **Tag Extensibility**: Leverage existing tag system
4. **No-std Compatible**: Core lib works with `alloc` only
5. **Feature Gating**: Optional `nip29` feature
6. **Trait Abstraction**: Pluggable storage/signers
7. **Async/Await**: Non-blocking I/O throughout

### 4.2 Module Structure

```
crates/nostr/src/nips/nip29/
├── mod.rs              # Module root + re-exports
├── types.rs            # Core types (Group, GroupMetadata, Role, etc.)
├── event.rs            # Event-specific structs
├── error.rs            # NIP-29 specific errors
└── constants.rs        # Kind constants, ranges

crates/nostr/src/event/
├── kind.rs             # Add NIP-29 kinds
└── builder.rs          # Add group builder methods

crates/nostr/src/event/tag/
└── standard.rs         # Add group-specific tags

crates/nostr-sdk/src/
└── client/groups.rs    # High-level group operations
```

### 4.3 Core Data Structures

#### Group Identification

```rust
/// Group identifier in format: <host>'<group-id>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupId {
    pub relay_url: RelayUrl,
    pub id: String,
}

impl GroupId {
    pub fn new(relay_url: RelayUrl, id: String) -> Result<Self, Error> {
        validate_group_id(&id)?;
        Ok(Self { relay_url, id })
    }

    pub fn to_tag(&self) -> String {
        format!("{}'{}",  self.relay_url.as_str(), self.id)
    }

    pub fn is_top_level(&self) -> bool {
        self.id == "_"
    }
}

impl FromStr for GroupId {
    // Parse from "host'id" format
}
```

#### Group Metadata

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroupMetadata {
    pub name: Option<String>,
    pub about: Option<String>,
    pub picture: Option<Url>,
    pub privacy: Privacy,
    pub closed: AccessModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Privacy {
    Public,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessModel {
    Open,
    Closed,
}

impl From<GroupMetadata> for Vec<Tag> {
    fn from(metadata: GroupMetadata) -> Self {
        let mut tags = Vec::new();
        if let Some(name) = metadata.name {
            tags.push(Tag::from(TagStandard::Name(name)));
        }
        // ... convert all fields to tags
        tags
    }
}
```

#### Roles

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Role {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroupRoles {
    pub roles: Vec<Role>,
}

impl From<GroupRoles> for Vec<Tag> {
    fn from(roles: GroupRoles) -> Self {
        roles.roles.into_iter()
            .map(|r| Tag::from(TagStandard::Role {
                name: r.name,
                description: r.description,
            }))
            .collect()
    }
}
```

#### Admin List

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupAdmin {
    pub public_key: PublicKey,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroupAdmins {
    pub admins: Vec<GroupAdmin>,
}

impl From<GroupAdmins> for Vec<Tag> {
    fn from(admins: GroupAdmins) -> Self {
        admins.admins.into_iter()
            .flat_map(|admin| {
                let mut tags = vec![Tag::public_key(admin.public_key)];
                tags.extend(admin.roles.into_iter()
                    .map(|role| Tag::from(TagStandard::Role {
                        name: role,
                        description: None,
                    })));
                tags
            })
            .collect()
    }
}
```

#### Members List

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroupMembers {
    pub members: Vec<PublicKey>,
}

impl From<GroupMembers> for Vec<Tag> {
    fn from(members: GroupMembers) -> Self {
        members.members.into_iter()
            .map(Tag::public_key)
            .collect()
    }
}
```

#### Moderation Actions

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModerationAction {
    PutUser { public_key: PublicKey, roles: Vec<String> },
    RemoveUser(PublicKey),
    EditMetadata(GroupMetadata),
    DeleteEvent(EventId),
    CreateGroup,
    DeleteGroup,
    CreateInvite,
}

impl ModerationAction {
    pub fn kind(&self) -> Kind {
        match self {
            Self::PutUser { .. } => Kind::GroupPutUser,
            Self::RemoveUser(_) => Kind::GroupRemoveUser,
            Self::EditMetadata(_) => Kind::GroupEditMetadata,
            Self::DeleteEvent(_) => Kind::GroupDeleteEvent,
            Self::CreateGroup => Kind::GroupCreate,
            Self::DeleteGroup => Kind::GroupDelete,
            Self::CreateInvite => Kind::GroupCreateInvite,
        }
    }
}
```

### 4.4 Event Kind Additions

**Add to `crates/nostr/src/event/kind.rs`:**

```rust
kind_variants! {
    // ... existing variants ...

    // NIP-29: Relay-based Groups
    GroupPutUser => 9000, "Group: Add/Update User", "https://github.com/nostr-protocol/nips/blob/master/29.md",
    GroupRemoveUser => 9001, "Group: Remove User", "https://github.com/nostr-protocol/nips/blob/master/29.md",
    GroupEditMetadata => 9002, "Group: Edit Metadata", "https://github.com/nostr-protocol/nips/blob/master/29.md",
    GroupDeleteEvent => 9005, "Group: Delete Event", "https://github.com/nostr-protocol/nips/blob/master/29.md",
    GroupCreate => 9007, "Group: Create Group", "https://github.com/nostr-protocol/nips/blob/master/29.md",
    GroupDelete => 9008, "Group: Delete Group", "https://github.com/nostr-protocol/nips/blob/master/29.md",
    GroupCreateInvite => 9009, "Group: Create Invite", "https://github.com/nostr-protocol/nips/blob/master/29.md",

    GroupJoinRequest => 9021, "Group: Join Request", "https://github.com/nostr-protocol/nips/blob/master/29.md",
    GroupLeaveRequest => 9022, "Group: Leave Request", "https://github.com/nostr-protocol/nips/blob/master/29.md",

    GroupMetadata => 39000, "Group: Metadata", "https://github.com/nostr-protocol/nips/blob/master/29.md",
    GroupAdmins => 39001, "Group: Admins List", "https://github.com/nostr-protocol/nips/blob/master/29.md",
    GroupMembers => 39002, "Group: Members List", "https://github.com/nostr-protocol/nips/blob/master/29.md",
    GroupRoles => 39003, "Group: Roles Definition", "https://github.com/nostr-protocol/nips/blob/master/29.md",
}

// Helper constants
pub const NIP29_MODERATION_RANGE: [u16; 7] = [9000, 9001, 9002, 9005, 9007, 9008, 9009];
pub const NIP29_METADATA_RANGE: [u16; 4] = [39000, 39001, 39002, 39003];

impl Kind {
    pub fn is_group_moderation(&self) -> bool {
        NIP29_MODERATION_RANGE.contains(&self.as_u16())
    }

    pub fn is_group_metadata(&self) -> bool {
        NIP29_METADATA_RANGE.contains(&self.as_u16())
    }
}
```

### 4.5 Tag Standard Additions

**Add to `crates/nostr/src/event/tag/standard.rs`:**

```rust
pub enum TagStandard {
    // ... existing variants ...

    /// Group identifier (`h` tag)
    /// NIP-29: https://github.com/nostr-protocol/nips/blob/master/29.md
    GroupId(String),

    /// Timeline reference (`previous` tag)
    /// NIP-29: References to recent events (first 8 chars)
    Previous(Vec<String>),

    /// Invite code (`code` tag)
    /// NIP-29: Pre-authorized admission code
    InviteCode(String),

    /// Privacy setting
    Privacy(Privacy),

    /// Access model
    Closed(AccessModel),

    /// Role assignment
    Role {
        name: String,
        description: Option<String>,
    },
}
```

### 4.6 EventBuilder Extensions

**Add to `crates/nostr/src/event/builder.rs`:**

```rust
impl EventBuilder {
    // ========================================
    // Group Messages
    // ========================================

    /// Create a group message (kind 9)
    ///
    /// # Example
    /// ```
    /// let event = EventBuilder::group_message(group_id, "Hello group!")
    ///     .sign_with_keys(&keys)?;
    /// ```
    pub fn group_message<S>(group_id: GroupId, content: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::ChatMessage, content)
            .tags([Tag::from_standardized_without_cell(
                TagStandard::GroupId(group_id.to_tag())
            )])
    }

    /// Add timeline references (previous events)
    pub fn with_previous(mut self, event_ids: Vec<EventId>) -> Self {
        let refs: Vec<String> = event_ids.iter()
            .map(|id| id.to_string()[..8].to_string())
            .collect();
        self = self.tag(Tag::from(TagStandard::Previous(refs)));
        self
    }

    // ========================================
    // Join/Leave Requests
    // ========================================

    /// Join request (kind 9021)
    pub fn group_join_request<S>(group_id: GroupId, message: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::GroupJoinRequest, message)
            .tag(Tag::from(TagStandard::GroupId(group_id.to_tag())))
    }

    /// Join request with invite code
    pub fn group_join_with_code<S>(group_id: GroupId, code: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(Kind::GroupJoinRequest, "")
            .tags([
                Tag::from(TagStandard::GroupId(group_id.to_tag())),
                Tag::from(TagStandard::InviteCode(code.into())),
            ])
    }

    /// Leave request (kind 9022)
    pub fn group_leave_request(group_id: GroupId) -> Self {
        Self::new(Kind::GroupLeaveRequest, "")
            .tag(Tag::from(TagStandard::GroupId(group_id.to_tag())))
    }

    // ========================================
    // Moderation Actions (9000-9009)
    // ========================================

    /// Add or update user with roles (kind 9000)
    pub fn group_put_user(
        group_id: GroupId,
        public_key: PublicKey,
        roles: Vec<String>
    ) -> Self {
        let mut builder = Self::new(Kind::GroupPutUser, "")
            .tags([
                Tag::from(TagStandard::GroupId(group_id.to_tag())),
                Tag::public_key(public_key),
            ]);

        for role in roles {
            builder = builder.tag(Tag::from(TagStandard::Role {
                name: role,
                description: None,
            }));
        }

        builder
    }

    /// Remove user (kind 9001)
    pub fn group_remove_user(group_id: GroupId, public_key: PublicKey) -> Self {
        Self::new(Kind::GroupRemoveUser, "")
            .tags([
                Tag::from(TagStandard::GroupId(group_id.to_tag())),
                Tag::public_key(public_key),
            ])
    }

    /// Edit group metadata (kind 9002)
    pub fn group_edit_metadata(group_id: GroupId, metadata: GroupMetadata) -> Self {
        let mut builder = Self::new(Kind::GroupEditMetadata, "")
            .tag(Tag::from(TagStandard::GroupId(group_id.to_tag())));

        builder = builder.tags(Vec::<Tag>::from(metadata));
        builder
    }

    /// Delete event (kind 9005)
    pub fn group_delete_event(group_id: GroupId, event_id: EventId) -> Self {
        Self::new(Kind::GroupDeleteEvent, "")
            .tags([
                Tag::from(TagStandard::GroupId(group_id.to_tag())),
                Tag::event(event_id),
            ])
    }

    /// Create group (kind 9007)
    pub fn group_create(group_id: GroupId, metadata: GroupMetadata) -> Self {
        let mut builder = Self::new(Kind::GroupCreate, "")
            .tag(Tag::from(TagStandard::GroupId(group_id.to_tag())));

        builder = builder.tags(Vec::<Tag>::from(metadata));
        builder
    }

    /// Delete group (kind 9008)
    pub fn group_delete(group_id: GroupId) -> Self {
        Self::new(Kind::GroupDelete, "")
            .tag(Tag::from(TagStandard::GroupId(group_id.to_tag())))
    }

    /// Create invite code (kind 9009)
    pub fn group_create_invite(group_id: GroupId) -> Self {
        Self::new(Kind::GroupCreateInvite, "")
            .tag(Tag::from(TagStandard::GroupId(group_id.to_tag())))
    }

    // ========================================
    // Metadata Events (39000-39003)
    // ========================================

    /// Group metadata event (kind 39000, addressable)
    pub fn group_metadata(group_id: GroupId, metadata: GroupMetadata) -> Self {
        let mut builder = Self::new(Kind::GroupMetadata, "")
            .tags([
                Tag::identifier(group_id.id.clone()),
                Tag::from(TagStandard::GroupId(group_id.to_tag())),
            ]);

        builder = builder.tags(Vec::<Tag>::from(metadata));
        builder
    }

    /// Group admins list (kind 39001, addressable)
    pub fn group_admins(group_id: GroupId, admins: GroupAdmins) -> Self {
        Self::new(Kind::GroupAdmins, "")
            .tags([Tag::identifier(group_id.id.clone())])
            .tags(Vec::<Tag>::from(admins))
    }

    /// Group members list (kind 39002, addressable)
    pub fn group_members(group_id: GroupId, members: GroupMembers) -> Self {
        Self::new(Kind::GroupMembers, "")
            .tags([Tag::identifier(group_id.id.clone())])
            .tags(Vec::<Tag>::from(members))
    }

    /// Group roles definition (kind 39003, addressable)
    pub fn group_roles(group_id: GroupId, roles: GroupRoles) -> Self {
        Self::new(Kind::GroupRoles, "")
            .tags([Tag::identifier(group_id.id.clone())])
            .tags(Vec::<Tag>::from(roles))
    }
}
```

---

## 5. Detailed Implementation Plan

### Phase 1: Core Protocol Types (Week 1)

**Files to Create/Modify:**

1. **Create `crates/nostr/src/nips/nip29/mod.rs`**
   - Module declaration and re-exports
   - Feature gating: `#![cfg(feature = "nip29")]`

2. **Create `crates/nostr/src/nips/nip29/types.rs`**
   - `GroupId` struct with validation
   - `GroupMetadata` struct
   - `Privacy` and `AccessModel` enums
   - `Role` and `GroupRoles` structs
   - `GroupAdmin` and `GroupAdmins` structs
   - `GroupMembers` struct
   - `ModerationAction` enum
   - All `From<T> for Vec<Tag>` implementations
   - Serialization/deserialization support

3. **Create `crates/nostr/src/nips/nip29/error.rs`**
   - `Nip29Error` enum
   - Invalid group ID format
   - Invalid privacy/access values
   - Missing required tags

4. **Create `crates/nostr/src/nips/nip29/constants.rs`**
   - Kind ranges
   - Special group ID constants
   - Validation regex patterns

**Tests:**
- Unit tests for all data structures
- Tag conversion tests
- Serialization round-trip tests
- GroupId parsing and validation

**Estimated Time:** 3-4 days

---

### Phase 2: Event Kinds and Tags (Week 1-2)

**Files to Modify:**

1. **`crates/nostr/src/event/kind.rs`**
   - Add 13 new Kind variants (see section 4.4)
   - Add helper methods: `is_group_moderation()`, `is_group_metadata()`
   - Add kind range constants

2. **`crates/nostr/src/event/tag/standard.rs`**
   - Add `GroupId` variant
   - Add `Previous` variant (timeline references)
   - Add `InviteCode` variant
   - Add `Privacy` and `Closed` variants
   - Add `Role` variant
   - Implement parsing for all new tags

3. **`crates/nostr/src/event/tag/mod.rs`**
   - Add convenience constructors:
     - `Tag::group_id()`
     - `Tag::previous()`
     - `Tag::invite_code()`
     - etc.

**Tests:**
- Kind conversion tests (u16 ↔ Kind)
- Tag parsing tests
- Tag serialization tests
- Kind range validation tests

**Estimated Time:** 2-3 days

---

### Phase 3: EventBuilder Integration (Week 2)

**Files to Modify:**

1. **`crates/nostr/src/event/builder.rs`**
   - Add all 16 group builder methods (see section 4.6)
   - Comprehensive documentation with examples
   - Ensure proper tag ordering

**Tests:**
- Builder method tests for each function
- Tag correctness validation
- Content serialization tests
- Integration tests with signing

**Estimated Time:** 3-4 days

---

### Phase 4: Client SDK Integration (Week 3)

**Files to Create/Modify:**

1. **Create `crates/nostr-sdk/src/client/groups.rs`**
   - High-level group operations
   - Trait: `GroupOperations`

```rust
#[async_trait]
pub trait GroupOperations {
    // Group management
    async fn create_group(&self, relay: RelayUrl, id: String, metadata: GroupMetadata) -> Result<Event>;
    async fn delete_group(&self, group_id: GroupId) -> Result<Event>;

    // Membership
    async fn join_group(&self, group_id: GroupId, message: Option<String>) -> Result<Event>;
    async fn join_group_with_code(&self, group_id: GroupId, code: String) -> Result<Event>;
    async fn leave_group(&self, group_id: GroupId) -> Result<Event>;

    // Moderation
    async fn add_group_member(&self, group_id: GroupId, pubkey: PublicKey, roles: Vec<String>) -> Result<Event>;
    async fn remove_group_member(&self, group_id: GroupId, pubkey: PublicKey) -> Result<Event>;
    async fn update_group_metadata(&self, group_id: GroupId, metadata: GroupMetadata) -> Result<Event>;
    async fn delete_group_event(&self, group_id: GroupId, event_id: EventId) -> Result<Event>;

    // Messaging
    async fn send_group_message(&self, group_id: GroupId, content: String, previous: Option<Vec<EventId>>) -> Result<Event>;

    // Queries
    async fn get_group_metadata(&self, group_id: GroupId) -> Result<Option<GroupMetadata>>;
    async fn get_group_admins(&self, group_id: GroupId) -> Result<Vec<GroupAdmin>>;
    async fn get_group_members(&self, group_id: GroupId) -> Result<Vec<PublicKey>>;
    async fn get_group_messages(&self, group_id: GroupId, limit: Option<usize>) -> Result<Vec<Event>>;
    async fn check_membership(&self, group_id: GroupId, pubkey: PublicKey) -> Result<bool>;
}

impl GroupOperations for Client {
    // Implementation...
}
```

2. **Modify `crates/nostr-sdk/src/client/mod.rs`**
   - Re-export group operations
   - Integrate into main Client struct

**Tests:**
- Mock relay integration tests
- End-to-end workflow tests
- Error handling tests

**Estimated Time:** 4-5 days

---

### Phase 5: Filter Support (Week 3)

**Files to Modify:**

1. **`crates/nostr/src/event/filter.rs`**
   - Add group-specific filter helpers

```rust
impl Filter {
    /// Filter for group events
    pub fn group(group_id: GroupId) -> Self {
        Self::new().custom_tag(SingleLetterTag::lowercase(Alphabet::H), [group_id.to_tag()])
    }

    /// Filter for group metadata events
    pub fn group_metadata(group_id: GroupId) -> Self {
        Self::new()
            .kind(Kind::GroupMetadata)
            .identifier(group_id.id)
    }

    /// Filter for group messages
    pub fn group_messages(group_id: GroupId) -> Self {
        Self::new()
            .kind(Kind::ChatMessage)
            .custom_tag(SingleLetterTag::lowercase(Alphabet::H), [group_id.to_tag()])
    }

    /// Filter for moderation events
    pub fn group_moderation(group_id: GroupId) -> Self {
        Self::new()
            .kinds(NIP29_MODERATION_RANGE.iter().map(|&k| Kind::from(k)))
            .custom_tag(SingleLetterTag::lowercase(Alphabet::H), [group_id.to_tag()])
    }
}
```

**Tests:**
- Filter construction tests
- Filter serialization tests
- Integration with relay queries

**Estimated Time:** 2 days

---

### Phase 6: Database Support (Week 4)

**Files to Modify:**

1. **`crates/nostr-sdk/src/database/mod.rs`**
   - Extend `NostrDatabase` trait with group queries

```rust
#[async_trait]
pub trait NostrDatabase: Send + Sync {
    // Existing methods...

    // Group-specific queries
    async fn get_group_metadata(&self, group_id: &GroupId) -> Result<Option<Event>>;
    async fn get_group_admins(&self, group_id: &GroupId) -> Result<Option<Event>>;
    async fn get_group_members(&self, group_id: &GroupId) -> Result<Option<Event>>;
    async fn get_group_messages(&self, group_id: &GroupId, limit: usize) -> Result<Vec<Event>>;
    async fn list_groups(&self) -> Result<Vec<GroupId>>;
}
```

2. **Update all database backends:**
   - LMDB: `crates/nostr-lmdb/src/store.rs`
   - NDB: `crates/nostr-ndb/src/database.rs`
   - IndexedDB: `crates/nostr-indexeddb/src/lib.rs`
   - Memory: `crates/nostr-sdk/src/database/memory.rs`

**Indexes to Add:**
- `h` tag index (group ID)
- Kind index for group kinds
- Composite index: group ID + timestamp

**Tests:**
- Database CRUD tests for each backend
- Query performance tests
- Concurrent access tests

**Estimated Time:** 5-6 days

---

### Phase 7: Examples and Documentation (Week 4-5)

**Files to Create:**

1. **`examples/nip29_basic.rs`**
   - Simple group creation and messaging

2. **`examples/nip29_moderation.rs`**
   - Admin operations demo

3. **`examples/nip29_client.rs`**
   - Full-featured group client

4. **`crates/nostr/src/nips/nip29/README.md`**
   - Comprehensive usage guide
   - Architecture overview
   - API documentation

5. **Update main docs:**
   - Add NIP-29 to supported NIPs list
   - Add to CHANGELOG.md
   - Update README.md

**Estimated Time:** 3-4 days

---

### Phase 8: Testing and Validation (Week 5)

**Test Categories:**

1. **Unit Tests:**
   - All data structures
   - Tag conversions
   - Event building
   - Validation logic

2. **Integration Tests:**
   - End-to-end workflows
   - Multi-relay scenarios
   - Database persistence

3. **Compliance Tests:**
   - NIP-29 specification compliance
   - Event format validation
   - Tag requirements

4. **Performance Tests:**
   - Large group handling
   - Message throughput
   - Database query performance

**Test Coverage Goal:** >90%

**Estimated Time:** 4-5 days

---

## 6. API Design

### 6.1 Low-Level API (nostr crate)

**Event Creation:**
```rust
use nostr::prelude::*;

// Create group
let group_id = GroupId::new(relay_url, "rust-devs".to_string())?;
let metadata = GroupMetadata {
    name: Some("Rust Developers".into()),
    about: Some("A group for Rust enthusiasts".into()),
    privacy: Privacy::Public,
    closed: AccessModel::Closed,
    ..Default::default()
};

let event = EventBuilder::group_create(group_id.clone(), metadata)
    .sign_with_keys(&keys)?;

// Send message
let msg_event = EventBuilder::group_message(group_id, "Hello everyone!")
    .with_previous(recent_event_ids)
    .sign_with_keys(&keys)?;

// Moderation
let add_user = EventBuilder::group_put_user(
    group_id,
    new_member_pubkey,
    vec!["member".to_string()]
).sign_with_keys(&admin_keys)?;
```

### 6.2 High-Level API (nostr-sdk crate)

**Client Operations:**
```rust
use nostr_sdk::prelude::*;

// Initialize client
let client = Client::new(&keys);
client.add_relay("wss://relay.example.com").await?;
client.connect().await;

// Create and join group
let group_id = client.create_group(
    relay_url,
    "rust-devs".to_string(),
    metadata
).await?;

client.join_group(group_id.clone(), Some("I love Rust!")).await?;

// Send messages
client.send_group_message(group_id.clone(), "Hello!", None).await?;

// Query
let messages = client.get_group_messages(group_id.clone(), Some(50)).await?;
let metadata = client.get_group_metadata(group_id.clone()).await?;

// Admin operations
if client.check_admin(group_id.clone()).await? {
    client.add_group_member(
        group_id.clone(),
        new_member,
        vec!["member".to_string()]
    ).await?;
}

// Subscribe to group events
client.subscribe_group(group_id.clone(), |event| {
    println!("New event: {:?}", event);
}).await?;
```

### 6.3 Error Handling

```rust
use nostr::nips::nip29::Nip29Error;

#[derive(Debug, Error)]
pub enum Nip29Error {
    #[error("Invalid group ID format: {0}")]
    InvalidGroupId(String),

    #[error("Missing required tag: {0}")]
    MissingRequiredTag(String),

    #[error("Not authorized for this action")]
    NotAuthorized,

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Already a member")]
    AlreadyMember,

    #[error("Not a member")]
    NotMember,
}
```

---

## 7. Testing Strategy

### 7.1 Unit Tests

**Location:** Inline in each module

**Coverage:**
- All public API functions
- Edge cases (empty groups, invalid IDs)
- Boundary conditions
- Error paths

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_id_parsing() {
        let id = GroupId::from_str("relay.example.com'test-group").unwrap();
        assert_eq!(id.id, "test-group");
        assert_eq!(id.relay_url.as_str(), "wss://relay.example.com");
    }

    #[test]
    fn test_invalid_group_id() {
        assert!(GroupId::from_str("no-delimiter").is_err());
        assert!(GroupId::from_str("relay.com'invalid spaces").is_err());
    }

    #[test]
    fn test_group_metadata_tags() {
        let metadata = GroupMetadata {
            name: Some("Test".into()),
            privacy: Privacy::Public,
            ..Default::default()
        };
        let tags: Vec<Tag> = metadata.into();
        assert!(tags.iter().any(|t| matches!(t, Tag::Name(_))));
    }
}
```

### 7.2 Integration Tests

**Location:** `crates/nostr/tests/nip29_integration.rs`

**Scenarios:**
1. Create group → join → send message
2. Admin adds member → member sends message
3. Admin removes member → verify access denied
4. Group deletion → verify all events cleaned up
5. Multiple relays → same group ID handling

**Example:**
```rust
#[tokio::test]
async fn test_full_group_workflow() {
    let admin_keys = Keys::generate();
    let member_keys = Keys::generate();

    // Create group
    let group_id = GroupId::new(relay_url(), "test".into()).unwrap();
    let create_event = EventBuilder::group_create(group_id.clone(), Default::default())
        .sign_with_keys(&admin_keys).unwrap();

    // Member joins
    let join_event = EventBuilder::group_join_request(group_id.clone(), "Hello")
        .sign_with_keys(&member_keys).unwrap();

    // Admin approves
    let approve_event = EventBuilder::group_put_user(
        group_id.clone(),
        member_keys.public_key(),
        vec!["member".into()]
    ).sign_with_keys(&admin_keys).unwrap();

    // Member sends message
    let msg_event = EventBuilder::group_message(group_id.clone(), "Hi everyone!")
        .sign_with_keys(&member_keys).unwrap();

    // Verify event structure
    assert_eq!(msg_event.kind, Kind::ChatMessage);
    assert!(msg_event.tags.iter().any(|t| matches!(t, Tag::GroupId(_))));
}
```

### 7.3 Compliance Tests

**Location:** `crates/nostr/tests/nip29_compliance.rs`

**Tests:**
- All event kinds present
- Tag requirements met
- Timeline references validated
- Group ID format compliance
- Addressable event structure

### 7.4 Property-Based Tests

**Using:** `proptest` or `quickcheck`

**Properties:**
- Event serialization round-trip
- Tag order preservation
- ID uniqueness
- Signature validation

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_group_id_roundtrip(id in "[a-z0-9_-]+") {
        let relay = RelayUrl::parse("wss://relay.com").unwrap();
        let group_id = GroupId::new(relay, id.clone()).unwrap();
        let serialized = group_id.to_string();
        let parsed = GroupId::from_str(&serialized).unwrap();
        assert_eq!(group_id, parsed);
    }
}
```

### 7.5 Performance Tests

**Location:** `benches/nip29_benches.rs`

**Benchmarks:**
- Event creation throughput
- Tag parsing speed
- Database query performance
- Large group handling (1000+ members)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_group_message_creation(c: &mut Criterion) {
    let keys = Keys::generate();
    let group_id = GroupId::new(/*...*/).unwrap();

    c.bench_function("group_message", |b| {
        b.iter(|| {
            EventBuilder::group_message(black_box(group_id.clone()), "test")
                .sign_with_keys(&keys)
        })
    });
}

criterion_group!(benches, bench_group_message_creation);
criterion_main!(benches);
```

---

## 8. Migration and Compatibility

### 8.1 Backward Compatibility

**Guarantees:**
- Existing APIs unchanged
- Feature-gated (opt-in)
- No breaking changes to core types

### 8.2 Forward Compatibility

**Considerations:**
- Custom event kinds supported
- Tag extensibility maintained
- Future NIP-29 amendments accommodated

### 8.3 Interoperability

**Testing with:**
- go-nostr implementations
- nostr-groups client
- Other NIP-29 compliant relays

### 8.4 Database Migration

**Strategy:**
- No migration needed (new feature)
- Existing databases unaffected
- New indexes added transparently

---

## 9. Timeline and Milestones

### Week 1: Foundation (Days 1-7)
- [ ] Core data structures (types.rs)
- [ ] Error handling
- [ ] Event kinds addition
- [ ] Tag standards addition
- [ ] Unit tests for Phase 1-2

**Deliverable:** Core NIP-29 types functional

### Week 2: Builders (Days 8-14)
- [ ] EventBuilder methods (all 16)
- [ ] Documentation with examples
- [ ] Filter support
- [ ] Unit tests for Phase 3
- [ ] Integration tests for Phase 5

**Deliverable:** Event creation API complete

### Week 3: Client SDK (Days 15-21)
- [ ] High-level Client operations
- [ ] GroupOperations trait
- [ ] Async workflows
- [ ] Error handling
- [ ] Integration tests for Phase 4

**Deliverable:** User-facing API complete

### Week 4: Storage & Polish (Days 22-28)
- [ ] Database trait extensions
- [ ] LMDB backend implementation
- [ ] NDB backend implementation
- [ ] Memory backend implementation
- [ ] IndexedDB backend (WASM)
- [ ] Database integration tests

**Deliverable:** Persistent storage functional

### Week 5: Documentation & Release (Days 29-35)
- [ ] Examples (3 comprehensive demos)
- [ ] API documentation
- [ ] Usage guides
- [ ] CHANGELOG update
- [ ] Compliance testing
- [ ] Performance benchmarks
- [ ] Pre-release review

**Deliverable:** Release-ready implementation

---

## 10. Open Questions and Decisions

### 10.1 Relay Selection
**Question:** How should multi-relay scenarios handle same group IDs?
**Decision:** GroupId includes relay URL; treat as separate groups per-relay

### 10.2 Timeline References
**Question:** Should we auto-populate `previous` tags?
**Decision:** Provide helper method; let users opt-in (not automatic)

### 10.3 Caching Strategy
**Question:** How long to cache group metadata?
**Decision:** Use replaceable event semantics; always use latest

### 10.4 Permission Checking
**Question:** Should SDK validate permissions before sending?
**Decision:** No local validation; let relay reject (auth is relay responsibility)

### 10.5 Group Discovery
**Question:** How do users find groups?
**Decision:** Out of scope; implement in future NIP or use NIP-51 lists

---

## 11. Success Criteria

### Must Have (MVP)
- ✅ All 13 event kinds supported
- ✅ EventBuilder methods for all operations
- ✅ GroupId parsing and validation
- ✅ Tag system integration
- ✅ Basic Client API
- ✅ At least one database backend (LMDB)
- ✅ 3 working examples
- ✅ >80% test coverage

### Should Have
- ✅ All database backends supported
- ✅ Filter helpers
- ✅ Comprehensive documentation
- ✅ Performance benchmarks
- ✅ Compliance tests

### Nice to Have
- 🔮 Group invite link generation
- 🔮 Group discovery helpers
- 🔮 Admin UI helpers
- 🔮 Notification system integration

---

## 12. References

### Specifications
- [NIP-29](https://github.com/nostr-protocol/nips/blob/master/29.md) - Relay-based Groups
- [NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md) - Basic Protocol
- [NIP-51](https://github.com/nostr-protocol/nips/blob/master/51.md) - Lists

### Reference Implementations
- [nostr-groups](https://github.com/prosto/nostr-groups) - TypeScript/React client
- [go-nostr](https://github.com/nbd-wtf/go-nostr) - Go implementation

### Related NIPs
- NIP-28: Public Channels (similar, but public)
- NIP-51: Lists (group membership lists)
- NIP-10: Reply Threading (timeline references)

---

## Appendix A: File Checklist

### Files to Create
- [ ] `crates/nostr/src/nips/nip29/mod.rs`
- [ ] `crates/nostr/src/nips/nip29/types.rs`
- [ ] `crates/nostr/src/nips/nip29/error.rs`
- [ ] `crates/nostr/src/nips/nip29/constants.rs`
- [ ] `crates/nostr-sdk/src/client/groups.rs`
- [ ] `examples/nip29_basic.rs`
- [ ] `examples/nip29_moderation.rs`
- [ ] `examples/nip29_client.rs`
- [ ] `crates/nostr/tests/nip29_integration.rs`
- [ ] `crates/nostr/tests/nip29_compliance.rs`
- [ ] `benches/nip29_benches.rs`

### Files to Modify
- [ ] `crates/nostr/src/nips/mod.rs` (add nip29 module)
- [ ] `crates/nostr/src/event/kind.rs` (add 13 kinds)
- [ ] `crates/nostr/src/event/builder.rs` (add 16 methods)
- [ ] `crates/nostr/src/event/tag/standard.rs` (add 6 tag variants)
- [ ] `crates/nostr/src/event/tag/mod.rs` (add helpers)
- [ ] `crates/nostr/src/event/filter.rs` (add group filters)
- [ ] `crates/nostr-sdk/src/client/mod.rs` (integrate groups)
- [ ] `crates/nostr-sdk/src/database/mod.rs` (extend trait)
- [ ] `crates/nostr-lmdb/src/store.rs` (add queries)
- [ ] `crates/nostr-ndb/src/database.rs` (add queries)
- [ ] `crates/nostr-indexeddb/src/lib.rs` (add queries)
- [ ] `crates/nostr-sdk/src/database/memory.rs` (add queries)
- [ ] `crates/nostr/Cargo.toml` (add feature flag)
- [ ] `CHANGELOG.md`
- [ ] `README.md`

---

## Appendix B: Code Size Estimates

| Component | Estimated LOC | Files |
|-----------|---------------|-------|
| Core types | 500-700 | 3 |
| Event kinds | 50-100 | 1 |
| Tag standards | 100-150 | 2 |
| EventBuilder | 300-400 | 1 |
| Client API | 400-600 | 2 |
| Database | 600-800 | 5 |
| Tests | 1000-1500 | 5 |
| Examples | 300-500 | 3 |
| Documentation | 500-800 | 5 |
| **Total** | **~4000-5000** | **27** |

---

## Appendix C: Dependencies

### New Dependencies
None required! All features built on existing dependencies:
- `serde` (serialization)
- `secp256k1` (crypto)
- `tokio` (async)
- `async-trait` (traits)

### Optional Dev Dependencies
- `proptest` (property testing)
- `criterion` (benchmarking)
- `mockito` (mock relays)

---

**End of Implementation Plan**

This plan provides a comprehensive roadmap for implementing NIP-29 support in rust-nostr. The implementation follows established patterns, maintains backward compatibility, and provides both low-level and high-level APIs for developers.

**Next Steps:**
1. Review and approve this plan
2. Create GitHub issue/project board
3. Begin Phase 1 implementation
4. Iterate based on feedback

**Questions?** Open an issue or discussion in the rust-nostr repository.
