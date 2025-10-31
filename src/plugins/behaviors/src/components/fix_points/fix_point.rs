use crate::components::fix_points::{FixPoints, FixPointsDefinition};
use bevy::prelude::*;
use common::{
	tools::bone::Bone,
	traits::{accessors::get::TryApplyOn, handles_skill_behaviors::SkillSpawner},
	zyheeda_commands::ZyheedaCommands,
};

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[relationship(relationship_target = FixPoints)]
pub struct FixPointOf(pub(crate) Entity);

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct FixPointSpawner(pub(crate) SkillSpawner);

impl FixPointSpawner {
	pub(crate) fn insert(
		mut commands: ZyheedaCommands,
		bones: Query<(Entity, &Name), Changed<Name>>,
		definitions: Query<&FixPointsDefinition>,
		parents: Query<&ChildOf>,
	) {
		for (entity, name) in &bones {
			let Some((root, config)) = get_root(&definitions, &parents, entity) else {
				continue;
			};

			match config.0.get(&Bone(name.as_str())) {
				Some(spawner) => {
					commands.try_apply_on(&entity, |mut e| {
						e.try_insert((FixPointOf(root), FixPointSpawner(*spawner)));
					});
				}
				None => {
					commands.try_apply_on(&entity, |mut e| {
						e.try_remove::<(FixPointOf, FixPointSpawner)>();
					});
				}
			}
		}
	}
}

fn get_root<'a>(
	definitions: &'a Query<&FixPointsDefinition>,
	parents: &Query<&ChildOf>,
	entity: Entity,
) -> Option<(Entity, &'a FixPointsDefinition)> {
	parents
		.iter_ancestors(entity)
		.find_map(|ancestor| Some((ancestor, definitions.get(ancestor).ok()?)))
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _T {
		A,
		B,
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, FixPointSpawner::insert);

		app
	}

	#[test]
	fn insert() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(FixPointsDefinition(HashMap::from([(
				Bone("a"),
				SkillSpawner::Neutral,
			)])))
			.id();
		let in_between = app.world_mut().spawn(ChildOf(agent)).id();
		let entity = app
			.world_mut()
			.spawn((Name::from("a"), ChildOf(in_between)))
			.id();

		app.update();

		assert_eq!(
			(
				Some(&FixPointSpawner(SkillSpawner::Neutral)),
				Some(&FixPointOf(agent))
			),
			(
				app.world().entity(entity).get::<FixPointSpawner>(),
				app.world().entity(entity).get::<FixPointOf>(),
			)
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(FixPointsDefinition(HashMap::from([(
				Bone("a"),
				SkillSpawner::Neutral,
			)])))
			.id();
		let in_between = app.world_mut().spawn(ChildOf(agent)).id();
		let entity = app
			.world_mut()
			.spawn((Name::from("a"), ChildOf(in_between)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<FixPointSpawner>()
			.remove::<FixPointOf>();
		app.update();

		assert_eq!(
			(None, None),
			(
				app.world().entity(entity).get::<FixPointSpawner>(),
				app.world().entity(entity).get::<FixPointOf>(),
			)
		);
	}

	#[test]
	fn act_again_if_name_changed() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(FixPointsDefinition(HashMap::from([(
				Bone("a"),
				SkillSpawner::Neutral,
			)])))
			.id();
		let in_between = app.world_mut().spawn(ChildOf(agent)).id();
		let entity = app
			.world_mut()
			.spawn((Name::from("a"), ChildOf(in_between)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<FixPointSpawner>()
			.remove::<FixPointOf>()
			.get_mut::<Name>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			(
				Some(&FixPointSpawner(SkillSpawner::Neutral)),
				Some(&FixPointOf(agent))
			),
			(
				app.world().entity(entity).get::<FixPointSpawner>(),
				app.world().entity(entity).get::<FixPointOf>(),
			)
		);
	}

	#[test]
	fn remove_fix_point_when_name_becomes_invalid() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(FixPointsDefinition(HashMap::from([(
				Bone("a"),
				SkillSpawner::Neutral,
			)])))
			.id();
		let in_between = app.world_mut().spawn(ChildOf(agent)).id();
		let entity = app
			.world_mut()
			.spawn((Name::from("a"), ChildOf(in_between)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Name::from("invalid"));
		app.update();

		assert_eq!(
			(None, None),
			(
				app.world().entity(entity).get::<FixPointSpawner>(),
				app.world().entity(entity).get::<FixPointOf>(),
			)
		);
	}
}
