// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-29: Relay-based Groups Example

use nostr::nips::nip29::{
    AccessModel, GroupAdmin, GroupAdmins, GroupId, GroupMembers, GroupMetadata, GroupRoles,
    Privacy, Role,
};
use nostr::prelude::*;

fn main() -> Result<()> {
    // Generate keys for admin and members
    let admin_keys = Keys::generate();
    let member_keys = Keys::generate();

    println!("Admin pubkey: {}", admin_keys.public_key());
    println!("Member pubkey: {}", member_keys.public_key());
    println!();

    // Create a group identifier
    let relay_url = Url::parse("wss://relay.example.com")?;
    let group_id = GroupId::new(relay_url, "rust-devs".to_string())?;
    println!("Group ID: {}", group_id);
    println!();

    // ========================================
    // 1. Create Group (kind 9007)
    // ========================================
    println!("=== Creating Group ===");

    let metadata = GroupMetadata {
        name: Some("Rust Developers".into()),
        about: Some("A group for Rust enthusiasts and developers".into()),
        picture: Some(Url::parse("https://rust-lang.org/logo.png")?),
        privacy: Privacy::Public,
        closed: AccessModel::Closed,
    };

    let create_event = EventBuilder::group_create(group_id.clone(), metadata.clone())
        .sign_with_keys(&admin_keys)?;

    println!("Create group event:");
    println!("  Kind: {}", create_event.kind);
    println!("  Tags: {:?}", create_event.tags);
    println!();

    // ========================================
    // 2. Join Request (kind 9021)
    // ========================================
    println!("=== Member Join Request ===");

    let join_event = EventBuilder::group_join_request(
        group_id.clone(),
        Some("I'm passionate about Rust and want to join!"),
    )
    .sign_with_keys(&member_keys)?;

    println!("Join request event:");
    println!("  Kind: {}", join_event.kind);
    println!("  Content: {}", join_event.content);
    println!();

    // ========================================
    // 3. Add Member (kind 9000)
    // ========================================
    println!("=== Admin Adds Member ===");

    let add_member_event = EventBuilder::group_put_user(
        group_id.clone(),
        member_keys.public_key(),
        vec!["member".to_string()],
    )
    .sign_with_keys(&admin_keys)?;

    println!("Add member event:");
    println!("  Kind: {}", add_member_event.kind);
    println!("  Tags: {:?}", add_member_event.tags);
    println!();

    // ========================================
    // 4. Send Message (kind 9)
    // ========================================
    println!("=== Sending Group Message ===");

    let message_event = EventBuilder::group_message(group_id.clone(), "Hello, Rust community!")
        .sign_with_keys(&member_keys)?;

    println!("Group message event:");
    println!("  Kind: {}", message_event.kind);
    println!("  Content: {}", message_event.content);
    println!();

    // ========================================
    // 5. Message with Timeline References
    // ========================================
    println!("=== Message with Previous References ===");

    let previous_ids = vec![create_event.id, join_event.id];

    let message_with_refs =
        EventBuilder::group_message(group_id.clone(), "Replying to previous messages")
            .with_previous_events(previous_ids.clone())
            .sign_with_keys(&member_keys)?;

    println!("Message with timeline references:");
    println!("  Previous event IDs: {:?}", previous_ids);
    println!();

    // ========================================
    // 6. Edit Group Metadata (kind 9002)
    // ========================================
    println!("=== Editing Group Metadata ===");

    let updated_metadata = GroupMetadata {
        name: Some("Rust Developers & Enthusiasts".into()),
        about: Some("Updated description".into()),
        picture: metadata.picture.clone(),
        privacy: Privacy::Public,
        closed: AccessModel::Open, // Changed to open
    };

    let edit_metadata_event =
        EventBuilder::group_edit_metadata(group_id.clone(), updated_metadata)
            .sign_with_keys(&admin_keys)?;

    println!("Edit metadata event:");
    println!("  Kind: {}", edit_metadata_event.kind);
    println!();

    // ========================================
    // 7. Remove Member (kind 9001)
    // ========================================
    println!("=== Removing Member ===");

    let remove_event =
        EventBuilder::group_remove_user(group_id.clone(), member_keys.public_key())
            .sign_with_keys(&admin_keys)?;

    println!("Remove user event:");
    println!("  Kind: {}", remove_event.kind);
    println!();

    // ========================================
    // 8. Delete Event (kind 9005)
    // ========================================
    println!("=== Deleting Event ===");

    let delete_event =
        EventBuilder::group_delete_event(group_id.clone(), message_event.id)
            .sign_with_keys(&admin_keys)?;

    println!("Delete event:");
    println!("  Kind: {}", delete_event.kind);
    println!();

    // ========================================
    // 9. Create Invite Code (kind 9009)
    // ========================================
    println!("=== Creating Invite Code ===");

    let invite_event = EventBuilder::group_create_invite(group_id.clone())
        .sign_with_keys(&admin_keys)?;

    println!("Create invite event:");
    println!("  Kind: {}", invite_event.kind);
    println!();

    // ========================================
    // 10. Join with Invite Code
    // ========================================
    println!("=== Join with Invite Code ===");

    let join_with_code_event = EventBuilder::group_join_with_code(group_id.clone(), "INVITE123")
        .sign_with_keys(&member_keys)?;

    println!("Join with code event:");
    println!("  Tags: {:?}", join_with_code_event.tags);
    println!();

    // ========================================
    // 11. Leave Group (kind 9022)
    // ========================================
    println!("=== Leaving Group ===");

    let leave_event = EventBuilder::group_leave_request(group_id.clone())
        .sign_with_keys(&member_keys)?;

    println!("Leave request event:");
    println!("  Kind: {}", leave_event.kind);
    println!();

    // ========================================
    // 12. Relay-Generated Metadata Events
    // ========================================
    println!("=== Relay-Generated Metadata Events ===");

    // Group metadata (kind 39000)
    let metadata_event = EventBuilder::group_metadata(group_id.clone(), metadata.clone())
        .sign_with_keys(&admin_keys)?;

    println!("Group metadata event (kind 39000):");
    println!("  Addressable: Yes (d tag: {})", group_id.id);
    println!();

    // Group admins (kind 39001)
    let admins = GroupAdmins::new().add_admin(GroupAdmin::new(
        admin_keys.public_key(),
        vec!["admin".to_string(), "moderator".to_string()],
    ));

    let admins_event = EventBuilder::group_admins(group_id.clone(), admins)
        .sign_with_keys(&admin_keys)?;

    println!("Group admins event (kind 39001):");
    println!("  Tags: {:?}", admins_event.tags);
    println!();

    // Group members (kind 39002)
    let members = GroupMembers::new()
        .add_member(admin_keys.public_key())
        .add_member(member_keys.public_key());

    let members_event = EventBuilder::group_members(group_id.clone(), members)
        .sign_with_keys(&admin_keys)?;

    println!("Group members event (kind 39002):");
    println!("  Members count: 2");
    println!();

    // Group roles (kind 39003)
    let roles = GroupRoles::new()
        .add_role(Role::with_description("admin", "Full access"))
        .add_role(Role::with_description("moderator", "Can moderate messages"))
        .add_role(Role::new("member"));

    let roles_event = EventBuilder::group_roles(group_id.clone(), roles)
        .sign_with_keys(&admin_keys)?;

    println!("Group roles event (kind 39003):");
    println!("  Roles count: 3");
    println!();

    // ========================================
    // 13. Delete Group (kind 9008)
    // ========================================
    println!("=== Deleting Group ===");

    let delete_group_event = EventBuilder::group_delete(group_id.clone())
        .sign_with_keys(&admin_keys)?;

    println!("Delete group event:");
    println!("  Kind: {}", delete_group_event.kind);
    println!();

    println!("=== Example Complete ===");
    println!("Successfully demonstrated all NIP-29 event types!");

    Ok(())
}
