# Rust-Nostr SDK: Architecture & NIP-29 Integration Plan

## Executive Summary

The rust-nostr SDK is a modular, feature-rich Nostr protocol implementation organized into multiple crates. The architecture is well-designed for protocol extensions and already has patterns in place for NIP implementations. This document outlines the current architecture and provides a structured plan for integrating NIP-29 (Relay-based Groups).

---

## Project Structure Overview

### Workspace Organization
Located at `/home/user/rust-nostr/`, the project is organized as a Rust workspace with the following key components:

```
rust-nostr/
├── crates/
│   ├── nostr/                    # Core protocol implementation
│   ├── nostr-sdk/                # High-level client SDK
│   ├── nostr-relay-pool/         # Relay management
│   ├── nostr-relay-builder/      # Relay construction utilities
│   └── nwc/                      # Nostr Wallet Connect
├── database/
│   ├── nostr-database/           # Database trait and abstractions
│   ├── nostr-lmdb/               # LMDB backend
│   ├── nostr-ndb/                # NDB backend
│   └── nostr-indexeddb/          # IndexedDB backend
├── gossip/
│   ├── nostr-gossip/             # Gossip protocol implementation
│   ├── nostr-gossip-memory/      # In-memory gossip storage
│   └── nostr-gossip-test-suite/  # Testing utilities
├── signer/
│   ├── nostr-connect/            # Remote signing (NIP-46)
│   ├── nostr-browser-signer/     # Browser signer integration
│   └── nostr-browser-signer-proxy/
└── rfs/
    ├── nostr-blossom/            # File storage (BUD-01)
    └── nostr-http-file-storage/
```

---

## Core Protocol Implementation (`crates/nostr/src/`)

### 1. Event System

**Location:** `/crates/nostr/src/event/`

#### Event Structure
- **Core Type:** `Event` struct (24 bytes header + dynamic content)
  - `id: EventId` - SHA256 hash of the event
  - `pubkey: PublicKey` - Author's public key
  - `created_at: Timestamp` - UNIX timestamp in seconds
  - `kind: Kind` - Event classification (u16)
  - `tags: Tags` - List of tag arrays
  - `content: String` - Event content
  - `sig: Signature` - Schnorr signature

#### Key Files
- `mod.rs` - Event definition and core methods
- `builder.rs` - EventBuilder for constructing events (2296 lines)
- `kind.rs` - Event Kind enumeration with 100+ predefined kinds
- `tag/` - Comprehensive tag system
  - `mod.rs` - Tag struct and trait implementations
  - `standard.rs` - Standardized tags (200+ patterns)
  - `kind.rs` - TagKind enumeration
  - `list.rs` - Tags collection with querying capabilities

### 2. Event Kinds

**Location:** `/crates/nostr/src/event/kind.rs`

Event kinds are defined using a macro system that automatically generates From/To implementations:

```rust
// Kind ranges:
- REGULAR_RANGE: 1,000-10,000      // Standard events stored by relays
- REPLACEABLE_RANGE: 10,000-20,000 // Only latest per (pubkey, kind) stored
- EPHEMERAL_RANGE: 20,000-30,000   // Not stored by relays
- ADDRESSABLE_RANGE: 30,000-40,000 // Latest per (pubkey, kind, d-tag)
- NIP90_JOB_REQUEST: 5,000-6,000
- NIP90_JOB_RESULT: 6,000-7,000
```

**Currently Defined Kinds Include:**
- 0: Metadata
- 1: TextNote
- 3: ContactList
- 4: EncryptedDirectMessage
- 5: EventDeletion
- 40-44: Channel operations (NIP-28)
- 10000-10050: User lists (NIP-51)
- 30000+: Addressable events (replaceable with `d` tag)
- 30617-30618: Git operations (NIP-34)
- Custom kinds are supported via `Kind::Custom(u16)`

### 3. Tags System

**Location:** `/crates/nostr/src/event/tag/`

The tag system is highly structured with three layers:

#### Layer 1: Raw Tags
- `Tag` struct: `Vec<String>` with standardized variant caching
- Flexible parsing: `Tag::parse([key, value1, value2, ...])`
- Supports any custom tag format

#### Layer 2: Single-Letter Tags
- `SingleLetterTag`: Lowercase (a-z) and uppercase (A-Z) variants
- Used in Filter queries and event references
- Examples: `p` (pubkey), `e` (event), `a` (coordinate), `d` (identifier)

#### Layer 3: Standardized Tags (`TagStandard` enum)
Predefined patterns with type-safe parsing:
- Event references with relay hints and markers
- Public key references with aliases
- Coordinates (addressable events)
- Relay metadata (NIP-65)
- Content metadata (image, URL, hashtag, etc.)
- Protocol-specific tags (poll options, zap data, etc.)

**Current Standardized Tags include:**
- Event, PublicKey, Coordinate, Relay, RelayMetadata
- Image, Thumb, Description, Title, Summary
- Hashtag, URL, Web, Reference
- Identifier (d-tag for addressable events)
- Challenge (for auth), POW (for PoW)
- And 100+ more...

### 4. Filter System

**Location:** `/crates/nostr/src/filter.rs` (42KB)

The Filter enables querying events from relays:

```rust
pub struct Filter {
    ids: BTreeSet<EventId>,           // Event IDs
    authors: BTreeSet<PublicKey>,     // Author public keys
    kinds: BTreeSet<Kind>,            // Event kinds
    since: Option<Timestamp>,         // Min timestamp
    until: Option<Timestamp>,         // Max timestamp
    limit: Option<u64>,               // Result limit
    tags: BTreeMap<SingleLetterTag, BTreeSet<String>>, // Tag queries
}
```

**Methods:**
- `new()` - Create empty filter
- `kind(Kind)` / `kinds<I>(kinds: I)` - Add kind constraints
- `author(PublicKey)` / `authors<I>` - Add author constraints
- `id(EventId)` / `ids<I>` - Add event ID constraints
- `hashtag(s)`, `pubkey(s)`, `event(s)` - Add tag constraints
- `identifier(s)` - Query addressable events by d-tag
- Generic tag filters via single-letter tags (a-z, A-Z)

---

## NIP Implementation Patterns

### Current NIP Implementations

**Location:** `/crates/nostr/src/nips/` (45 files)

The project has comprehensive implementations of Nostr Improvement Proposals:

#### Fully Implemented NIPs
1. NIP-01: Basic protocol flow
2. NIP-02: Contact List
3. NIP-05: DNS-based verification
4. NIP-09: Event Deletion
5. NIP-10: Replies and threading
6. NIP-11: Relay Info Document
7. NIP-13: Proof of Work
8. NIP-15: Nostr Marketplace
9. NIP-17: Private Direct Messages
10. NIP-19: Bech32 encoding
11. NIP-21: URI Scheme
12. NIP-25: Reactions
13. NIP-28: Channels (relevant for groups)
14. NIP-34: Git Events
15. NIP-38: User Statuses
16. NIP-39: External Identities
17. NIP-42: Authentication
18. NIP-46: Remote Signing (feature-gated)
19. NIP-47: Wallet Service (feature-gated)
20. NIP-48: Proxy Tags
21. NIP-51: Lists (comprehensive)
22. NIP-53: Live Events
23. NIP-56: Reporting
24. NIP-57: Lightning Zaps (feature-gated)
25. NIP-58: Badges
26. NIP-62: Request to Vanish
27. NIP-65: Relay List Metadata
28. NIP-73: External Content ID
29. NIP-88: Polls
30. NIP-90: Data Vending Machine
31. NIP-94: File Metadata
32. NIP-96: HTTP File Storage (feature-gated)
33. NIP-98: HTTP Auth (feature-gated)
34. NIPB0: Web Bookmarks
35. NIPC0: Code Snippets

#### Feature-Gated NIPs
Some NIPs are behind feature flags (in `Cargo.toml`):
- `nip04` - Direct Messages Encryption
- `nip06` - Seed-based Key Derivation
- `nip44` - Encrypted Payloads
- `nip46` - Remote Signing
- `nip47` - Wallet Service
- `nip49` - Private Key Encryption
- `nip57` - Lightning Zaps
- `nip59` - Sealed & Seal
- `nip60` - Cashu Wallet
- `nip96` - HTTP File Storage
- `nip98` - HTTP Auth

### NIP Structure Pattern

#### Example: NIP-51 (Lists)

**File:** `/crates/nostr/src/nips/nip51.rs`

```rust
// Define data structures for the NIP
pub struct MuteList {
    pub public_keys: Vec<PublicKey>,
    pub hashtags: Vec<String>,
    pub event_ids: Vec<EventId>,
    pub words: Vec<String>,
}

// Convert to Event tags
impl From<MuteList> for Vec<Tag> { ... }

// Create with EventBuilder
EventBuilder::mute_list(MuteList { ... })
    .sign_with_keys(&keys)?
```

#### Example: NIP-28 (Channels)

**EventBuilder methods:**
```rust
pub fn channel(metadata: &Metadata) -> Self { ... }
pub fn channel_metadata(channel_id: EventId, relay_url, metadata) -> Self { ... }
pub fn channel_msg(channel_id: EventId, relay_url, content) -> Self { ... }
pub fn hide_channel_msg(message_id: EventId, reason: Option<S>) -> Self { ... }
pub fn mute_channel_user(public_key: PublicKey, reason: Option<S>) -> Self { ... }
```

### EventBuilder Pattern

**Location:** `/crates/nostr/src/event/builder.rs` (2296 lines)

The EventBuilder is the primary interface for creating events:

```rust
pub struct EventBuilder {
    kind: Kind,
    tags: Tags,
    content: String,
    custom_created_at: Option<Timestamp>,
    pow: Option<u8>,
    allow_self_tagging: bool,
    dedup_tags: bool,
}

// Key methods:
impl EventBuilder {
    pub fn new(kind: Kind, content: S) -> Self { ... }
    pub fn tag(self, tag: Tag) -> Self { ... }
    pub fn tags<I>(self, tags: I) -> Self { ... }
    pub fn sign_with_keys(self, keys: &Keys) -> Result<Event> { ... }
    pub fn custom_created_at(self, timestamp) -> Self { ... }
    pub fn pow(self, difficulty: u8) -> Self { ... }
    
    // NIP-specific helpers:
    pub fn text_note(content: S) -> Self { ... }
    pub fn contact_list(contacts: I) -> Self { ... }
    pub fn delete(request: EventDeletionRequest) -> Self { ... }
    pub fn reaction(target, reaction) -> Self { ... }
    pub fn channel(metadata: &Metadata) -> Self { ... }
    pub fn channel_msg(channel_id, relay_url, content) -> Self { ... }
    pub fn live_event(live_event) -> Self { ... }
    pub fn mute_list(list: MuteList) -> Self { ... }
    // ... 50+ more methods
}
```

---

## SDK Architecture (`crates/nostr-sdk/src/`)

### 1. Client Structure

**Location:** `/crates/nostr-sdk/src/client/`

The high-level client provides async/await interface:

```rust
pub struct Client {
    pool: RelayPool,              // Relay connection management
    gossip: Option<GossipWrapper>, // Gossip protocol for relay discovery
    opts: ClientOptions,
    gossip_sync: Arc<Semaphore>,
}

impl Client {
    pub fn new<T: IntoNostrSigner>(signer: T) -> Self { ... }
    pub fn builder() -> ClientBuilder { ... }
    
    // Relay management
    pub async fn add_relay<U: TryIntoUrl>(&self, url: U) -> Result<()> { ... }
    pub async fn remove_relay<U: TryIntoUrl>(&self, url: U) -> Result<()> { ... }
    pub async fn relays(&self) -> HashMap<RelayUrl, Relay> { ... }
    
    // Event operations
    pub async fn send_event(&self, event: Event) -> Result<Output> { ... }
    pub async fn batch_send_events(&self, events: Vec<Event>) -> Result<Output> { ... }
    
    // Subscriptions
    pub async fn subscribe(&self, filters: Vec<Filter>) -> Result<SubscriptionId> { ... }
    pub async fn unsubscribe(&self, id: SubscriptionId) -> Result<()> { ... }
    pub async fn unsubscribe_all(&self) -> Result<()> { ... }
    
    // Notifications
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> { ... }
    
    // Gossip/Discovery
    pub async fn gossip(&self) -> Result<Arc<NostrGossip>> { ... }
    pub async fn follow(&self, pubkey: PublicKey) -> Result<()> { ... }
}
```

### 2. Relay Pool

**Location:** `/crates/nostr-relay-pool/src/`

Manages multiple relay connections:

```rust
pub struct RelayPool {
    // Internal relay management
}

pub enum RelayPoolNotification {
    Relay(RelayUrl, RelayNotification),
    Message(RelayUrl, RelayMessage),
}

pub enum RelayNotification {
    Authenticated,
    Connected,
    Disconnected,
}
```

### 3. Database Layer

**Location:** `/database/nostr-database/src/`

Trait-based database abstraction:

```rust
pub trait NostrDatabase {
    async fn save_event(&self, event: &Event) -> Result<()> { ... }
    async fn query(&self, filters: &[Filter]) -> Result<Vec<Event>> { ... }
    async fn has_event(&self, id: &EventId) -> Result<bool> { ... }
    async fn delete_events(&self, ids: Vec<EventId>) -> Result<()> { ... }
}
```

Available backends:
- In-Memory (for testing)
- LMDB (embedded key-value store)
- NDB (newer database)
- IndexedDB (browser-based)

### 4. Gossip Protocol

**Location:** `/gossip/nostr-gossip/src/`

Manages relay discovery and reputation:

```rust
pub struct NostrGossip {
    // Relay reputation and discovery
}

pub enum GossipListKind {
    Relays,
    Following,
    FollowedBy,
    ProfilesCache,
    RelayMetadata,
}
```

---

## Message Protocol

**Location:** `/crates/nostr/src/message/`

### Client Messages
```rust
pub enum ClientMessage {
    Event(Event),           // ["EVENT", <event JSON>]
    Request(Filter),        // ["REQ", <id>, <filters...>]
    Close(SubscriptionId),  // ["CLOSE", <id>]
    Auth(Event),            // ["AUTH", <event JSON>]
    Count(Vec<Filter>),     // ["COUNT", <id>, <filters...>]
}
```

### Relay Messages
```rust
pub enum RelayMessage {
    Event(SubscriptionId, Event),      // ["EVENT", <id>, <event JSON>]
    Notice(String),                    // ["NOTICE", <message>]
    Eose(SubscriptionId),              // ["EOSE", <id>]
    Ok(EventId, bool, String),         // ["OK", <id>, <result>, <message>]
    Auth(String),                      // ["AUTH", <challenge>]
    Count(SubscriptionId, usize),      // ["COUNT", <id>, <count>]
}
```

---

## Existing Community/Group Features

### Current Group-Related Implementations

1. **NIP-28: Channels**
   - Kind 40: Channel Creation
   - Kind 41: Channel Metadata
   - Kind 42: Channel Message
   - Kind 43: Channel Hide Message
   - Kind 44: Channel Mute User
   - Kind 45-49: Reserved

2. **NIP-51: Lists**
   - Kind 10000: Mute List
   - Kind 10001: Pin List
   - Kind 10003: Bookmarks
   - Kind 10004: Communities
   - Kind 10005: Public Chats
   - Kind 10009: Simple Groups
   - Kind 30000-30030: Replaceable list sets

3. **Contact Management (NIP-02)**
   - Kind 3: Contact List
   - `p` tags for users

### Features NOT Yet Implemented

- **NIP-29: Relay-based Groups** (This is what we're planning!)
  - Group CRUD operations
  - Member management
  - Access control (public/private)
  - Admin roles
  - Group metadata
  - Permission models

---

## Feature Flags System

**Location:** `crates/nostr/Cargo.toml`

```toml
[features]
default = ["std"]
std = [...] # Standard library features
alloc = [...] # Allocation-only features (no_std)

# NIP features
nip04 = ["dep:aes", "dep:base64", "dep:cbc"]
nip06 = ["dep:bip39"]
nip44 = ["dep:base64", "dep:chacha20"]
nip46 = ["nip04", "nip44"]
# ... more NIP features

all-nips = ["nip04", "nip06", "nip44", "nip46", ...]
```

Features allow:
- Fine-grained dependency control
- Optional protocol extensions
- Reduced binary size for embedded systems
- no_std support with alloc only

---

## Prelude System

**Location:** `/crates/nostr/src/prelude.rs`

The prelude module re-exports commonly used types for convenience:

```rust
pub use crate::event::*;
pub use crate::filter::*;
pub use crate::key::*;
pub use crate::nips::*;
pub use crate::types::*;
// ... etc
```

SDK consumers import with:
```rust
use nostr_sdk::prelude::*;
```

---

## Design Patterns Observed

### 1. Builder Pattern
- EventBuilder for flexible event construction
- ClientBuilder for client configuration
- Filter builder methods for query construction

### 2. Trait-Based Abstraction
- NostrDatabase trait for pluggable backends
- NostrSigner trait for different signing methods
- IntoNostrSigner for flexible signer construction

### 3. Type Safety
- Kind enum for event classification
- Tag structure with standardized variants
- Coordinate for addressable event references
- PublicKey and EventId as distinct types

### 4. Async/Await
- All I/O operations are async
- broadcast channels for notifications
- Arc-wrapped shared state

### 5. Feature Gating
- Optional NIP implementations
- no_std support with alloc feature
- Compile-time protocol extensions

### 6. Tag-Based Extensibility
- Custom tags can be added without schema changes
- Standardized tags for common patterns
- Single-letter tags for query compatibility

---

## Integration Points for NIP-29

### Where NIP-29 Should Live

```
crates/nostr/src/nips/nip29.rs
```

### Required Components

1. **Event Kinds** (in `kind.rs`):
   - Add NIP-29 group event kinds (TBD by spec)
   - Likely addressable events (30,000-40,000 range)

2. **Standardized Tags** (in `event/tag/standard.rs`):
   - New tag variants for group-specific metadata
   - Permission/role tags
   - Group reference tags

3. **NIP-29 Module** (new `nip29.rs`):
   - Group data structures
   - Group event builders
   - Permission/access control helpers
   - Metadata parsing helpers

4. **EventBuilder Extensions** (in `event/builder.rs`):
   - `create_group(metadata)` → Kind for group creation
   - `group_add_member(group, pubkey)` → Member addition
   - `group_remove_member(group, pubkey)` → Member removal
   - `group_invite(group, pubkey)` → Invite event
   - `group_metadata_update(group, metadata)` → Metadata update
   - `group_admin_update(group, pubkey, role)` → Admin assignment
   - etc.

5. **Feature Gate** (in `Cargo.toml`):
   - Add `nip29` feature (optional dependencies if needed)
   - Include in `all-nips` feature

6. **Prelude Export** (in `prelude.rs`):
   - Export NIP-29 types and helpers
   - Use feature gate if not always enabled

### API Design Considerations

Based on existing patterns, NIP-29 API should:

1. **Use Event Kinds for Classification**
   - Define specific kinds for group operations
   - Use addressable events for group state

2. **Tag-Based Structure**
   - Use `d` tag for group identifier
   - Use `p` tags for members
   - Use custom tags for permissions/roles

3. **Builder Methods**
   - Follow EventBuilder pattern
   - Chain-able configuration
   - Error handling for invalid states

4. **Type-Safe Coordinates**
   - Use Coordinate struct for referencing groups
   - Leverage existing NIP-01 infrastructure

5. **Filter Integration**
   - Enable querying groups by identifier
   - Support filtering by members
   - Allow discovery by relay hints

### Example Implementation Sketch

```rust
// nip29.rs
pub struct Group {
    pub id: String,                    // d-tag identifier
    pub name: String,
    pub description: Option<String>,
    pub picture: Option<Url>,
    pub public: bool,                  // Access level
    pub owner: PublicKey,
    pub members: Vec<PublicKey>,
    pub admins: Vec<PublicKey>,
}

pub enum GroupRole {
    Owner,
    Admin,
    Moderator,
    Member,
    Viewer,
}

// In EventBuilder
pub fn create_group<S: Into<String>>(
    id: S,
    metadata: Group,
) -> Self {
    // Kind for group creation
    // Build with 'd' tag for identifier
    // Build with member 'p' tags
    // Build with role tags
}

pub fn group_add_member(
    group_id: String,
    member: PublicKey,
    role: GroupRole,
) -> Self {
    // Kind for member addition
    // Reference group with coordinate or identifier
    // Include member pubkey tag
}

// etc.
```

---

## Development Workflow

### Testing
- Use in-memory database for tests
- Mock relay responses
- Create fixtures for common scenarios

### Examples
- Add example in `/crates/nostr/examples/nip29.rs`
- Demonstrate group creation, member management, etc.

### Documentation
- Add NIP-29 protocol documentation
- Include rustdoc for all public APIs
- Add examples in doc comments

### Integration
- Test with nostr-sdk high-level client
- Ensure RelayPool handles group messages
- Verify database storage patterns

---

## Key Files Reference

| Component | File | Lines | Purpose |
|-----------|------|-------|---------|
| Event Core | `event/mod.rs` | 567 | Event struct and verification |
| Event Builder | `event/builder.rs` | 2296 | Event construction |
| Event Kinds | `event/kind.rs` | 398 | Kind enumeration |
| Tags | `event/tag/mod.rs` | 400+ | Tag structure |
| Tags Std | `event/tag/standard.rs` | 2500+ | Standardized tags |
| Filters | `filter.rs` | 42K | Event querying |
| NIPs | `nips/*.rs` | 45 files | Protocol extensions |
| Client | `../nostr-sdk/src/client/mod.rs` | 1300+ | High-level API |
| Prelude | `prelude.rs` | 90 | Type exports |

---

## Summary & Next Steps

The rust-nostr SDK provides excellent infrastructure for implementing NIP-29:

1. ✅ **Event system** - Ready to use
2. ✅ **Tag system** - Highly flexible
3. ✅ **EventBuilder** - Pattern established
4. ✅ **Filter system** - Query support
5. ✅ **Feature gating** - Conditional compilation
6. ✅ **Client/Relay pool** - Broadcasting support
7. ✅ **Database layer** - Event persistence
8. ⏳ **NIP-29** - To be implemented

### Recommended Implementation Order

1. Define NIP-29 event kinds
2. Design tag structures for group metadata
3. Create data structures in nip29.rs module
4. Implement EventBuilder helper methods
5. Add Filter support for group queries
6. Create comprehensive examples
7. Write tests and documentation
8. Integrate with nostr-sdk client
9. Add gossip protocol support (optional)

---

