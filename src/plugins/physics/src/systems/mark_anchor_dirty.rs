use crate::components::anchored_skill::{AnchoredSkill, AnchoredSkillDirty};
use bevy::prelude::*;
use common::prelude::*;

impl AnchoredSkill {
	pub(crate) fn mark_dirty(
		mut commands: ZyheedaCommands,
		anchors: Query<Entity, (With<Self>, Without<AnchoredSkillDirty>)>,
	) {
		for entity in anchors {
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(AnchoredSkillDirty);
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				AnchoredSkill::mark_dirty,
				IsChanged::<AnchoredSkillDirty>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn insert_anchor_dirty() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(AnchoredSkill::to(PersistentEntity::default()).on(SkillMount::Center))
			.remove::<AnchoredSkillDirty>()
			.id();

		app.update();

		assert_eq!(
			Some(&AnchoredSkillDirty),
			app.world().entity(entity).get::<AnchoredSkillDirty>(),
		);
	}

	#[test]
	fn do_not_insert_on_non_anchors() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<AnchoredSkillDirty>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(AnchoredSkill::to(PersistentEntity::default()).on(SkillMount::Center))
			.remove::<AnchoredSkillDirty>()
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(entity)
				.get::<IsChanged<AnchoredSkillDirty>>(),
		);
	}

	#[test]
	fn act_again_if_dirty_marker_missing() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(AnchoredSkill::to(PersistentEntity::default()).on(SkillMount::Center))
			.remove::<AnchoredSkillDirty>()
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<AnchoredSkillDirty>();
		app.update();

		assert_eq!(
			Some(&AnchoredSkillDirty),
			app.world().entity(entity).get::<AnchoredSkillDirty>(),
		);
	}
}
