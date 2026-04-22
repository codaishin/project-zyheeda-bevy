use crate::components::anchor::{Anchor, AnchorDirty};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl Anchor {
	pub(crate) fn mark_dirty(
		mut commands: ZyheedaCommands,
		anchors: Query<Entity, (With<Self>, Without<AnchorDirty>)>,
	) {
		for entity in anchors {
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(AnchorDirty);
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::anchor::AnchorDirty;
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::handles_skill_physics::SkillMount,
	};
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(Anchor::mark_dirty, IsChanged::<AnchorDirty>::detect).chain(),
		);

		app
	}

	#[test]
	fn insert_anchor_dirty() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Anchor::attach_to(PersistentEntity::default()).on(SkillMount::NeutralSlot))
			.remove::<AnchorDirty>()
			.id();

		app.update();

		assert_eq!(
			Some(&AnchorDirty),
			app.world().entity(entity).get::<AnchorDirty>(),
		);
	}

	#[test]
	fn do_not_insert_on_non_anchors() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<AnchorDirty>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Anchor::attach_to(PersistentEntity::default()).on(SkillMount::NeutralSlot))
			.remove::<AnchorDirty>()
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<AnchorDirty>>(),
		);
	}

	#[test]
	fn act_again_if_dirty_marker_missing() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Anchor::attach_to(PersistentEntity::default()).on(SkillMount::NeutralSlot))
			.remove::<AnchorDirty>()
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<AnchorDirty>();
		app.update();

		assert_eq!(
			Some(&AnchorDirty),
			app.world().entity(entity).get::<AnchorDirty>(),
		);
	}
}
