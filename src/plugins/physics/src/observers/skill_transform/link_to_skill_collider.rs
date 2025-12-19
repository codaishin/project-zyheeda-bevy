use crate::components::{
	skill_prefabs::skill_contact::SkillContact,
	skill_transform::{SkillTransform, SkillTransformOf},
};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl SkillTransform {
	pub(crate) fn link_to_skill_collider(
		trigger: Trigger<OnAdd, Self>,
		mut commands: ZyheedaCommands,
		contacts: Query<(), With<SkillContact>>,
		children: Query<&ChildOf>,
	) {
		let entity = trigger.target();
		let mut parents = children.iter_ancestors(entity);

		let Some(parent) = parents.find(|p| contacts.contains(*p)) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(SkillTransformOf(parent));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		skill_prefabs::skill_contact::{CreatedFrom, SkillContact},
		skill_transform::SkillTransformOf,
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::Units,
		traits::handles_skill_physics::{ContactShape, Motion, SkillCaster, SkillSpawner},
	};
	use std::collections::HashSet;
	use testing::SingleThreadedApp;

	fn skill_contact() -> SkillContact {
		SkillContact {
			created_from: CreatedFrom::Contact,
			shape: ContactShape::Sphere {
				radius: Units::ZERO,
				hollow_collider: false,
				destroyed_by: HashSet::from([]),
			},
			motion: Motion::HeldBy {
				caster: SkillCaster(PersistentEntity::default()),
				spawner: SkillSpawner::Neutral,
			},
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(SkillTransform::link_to_skill_collider);

		app
	}

	#[test]
	fn link_children_to_collider() {
		let mut app = setup();
		let contact = app.world_mut().spawn(skill_contact()).id();

		let children = [
			app.world_mut()
				.spawn((ChildOf(contact), SkillTransform))
				.id(),
			app.world_mut().spawn(ChildOf(contact)).id(),
			app.world_mut()
				.spawn((ChildOf(contact), SkillTransform))
				.id(),
		];

		assert_eq!(
			[
				Some(&SkillTransformOf(contact)),
				None,
				Some(&SkillTransformOf(contact))
			],
			app.world()
				.entity(children)
				.map(|c| c.get::<SkillTransformOf>()),
		);
	}

	#[test]
	fn do_not_link_when_parent_is_not_skill_contact() {
		let mut app = setup();
		let contact = app.world_mut().spawn_empty().id();

		let child = app
			.world_mut()
			.spawn((ChildOf(contact), SkillTransform))
			.id();

		assert_eq!(None, app.world().entity(child).get::<SkillTransformOf>());
	}

	#[test]
	fn link_distant_child_to_collider() {
		let mut app = setup();
		let contact = app.world_mut().spawn(skill_contact()).id();
		let in_between = app.world_mut().spawn(ChildOf(contact)).id();

		let child = app
			.world_mut()
			.spawn((ChildOf(in_between), SkillTransform))
			.id();

		assert_eq!(
			Some(&SkillTransformOf(contact)),
			app.world().entity(child).get::<SkillTransformOf>(),
		);
	}

	#[test]
	fn only_act_once() {
		let mut app = setup();
		let contact = app.world_mut().spawn(skill_contact()).id();
		let in_between = app.world_mut().spawn(ChildOf(contact)).id();

		let child = app
			.world_mut()
			.spawn((ChildOf(in_between), SkillTransform))
			.remove::<SkillTransformOf>()
			.insert(SkillTransform)
			.id();

		assert_eq!(None, app.world().entity(child).get::<SkillTransformOf>());
	}
}
