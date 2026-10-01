//! The seam `docs/CONTRIBUTING.md` ("Building on top") opens to a crate: read the streamed objects
//! through [`benilla_app::Objects`] and interact with one through [`benilla_app::Interact`], from
//! outside the crate. It fails to build if either goes private, with no feature needed.

use benilla_app::{Interact, Objects};
use bevy::prelude::*;

/// What such a crate writes: an object it found, interacted with.
fn interact_with_the_nearest(objects: Objects, mut interact: MessageWriter<Interact>) {
    if let Some((_, entity, _)) = objects.iter().next() {
        interact.write(Interact(entity));
    }
}

/// A key press such a crate answers: the guid it picked, interacted with.
fn interact_with_guid(guid: u64, objects: &Objects, interact: &mut MessageWriter<Interact>) {
    if let Some(entity) = objects.entity(guid) {
        interact.write(Interact(entity));
    }
}

fn is_a_system<M>(_: impl IntoSystem<(), (), M>) {}

#[test]
fn a_crate_on_top_reads_objects_and_writes_interact() {
    is_a_system(interact_with_the_nearest);
    let _ = interact_with_guid;

    let mut app = App::new();
    app.add_message::<Interact>();
    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().write_message(Interact(entity));
    let sent: Vec<Interact> = app
        .world()
        .resource::<Messages<Interact>>()
        .iter_current_update_messages()
        .copied()
        .collect();
    assert_eq!(sent, [Interact(entity)]);
}
