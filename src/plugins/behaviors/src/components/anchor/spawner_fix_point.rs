use bevy::prelude::*;
use common::{
	tools::action_key::slot::{Side, SlotKey},
	traits::{
		handles_skill_behaviors::Spawner,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct SpawnerFixPoint(pub(crate) Spawner);

impl SpawnerFixPoint {
	pub(crate) fn insert(mut commands: Commands, names: Query<(Entity, &Name), Changed<Name>>) {
		for (entity, name) in &names {
			let spawner = match name.as_str() {
				"skill_spawn" => Some(Spawner::Center),
				"skill_spawn_top.R" => Some(Spawner::Slot(SlotKey::TopHand(Side::Right))),
				"skill_spawn_top.L" => Some(Spawner::Slot(SlotKey::TopHand(Side::Left))),
				"skill_spawn_bottom.R" => Some(Spawner::Slot(SlotKey::BottomHand(Side::Right))),
				"skill_spawn_bottom.L" => Some(Spawner::Slot(SlotKey::BottomHand(Side::Left))),
				_ => None,
			};

			match spawner {
				Some(spawner) => {
					commands.try_insert_on(entity, SpawnerFixPoint(spawner));
				}
				None => {
					commands.try_remove_from::<SpawnerFixPoint>(entity);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;
	use test_case::test_case;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, SpawnerFixPoint::insert);

		app
	}

	#[test_case("invalid", None; "none")]
	#[test_case("skill_spawn", Some(&SpawnerFixPoint(Spawner::Center)); "center")]
	#[test_case(
		"skill_spawn_top.R",
		Some(&SpawnerFixPoint(Spawner::Slot(SlotKey::TopHand(Side::Right))));
		"top right"
	)]
	#[test_case(
		"skill_spawn_top.L",
		Some(&SpawnerFixPoint(Spawner::Slot(SlotKey::TopHand(Side::Left))));
		"top left"
	)]
	#[test_case(
		"skill_spawn_bottom.R",
		Some(&SpawnerFixPoint(Spawner::Slot(SlotKey::BottomHand(Side::Right))));
		"bottom right"
	)]
	#[test_case(
		"skill_spawn_bottom.L",
		Some(&SpawnerFixPoint(Spawner::Slot(SlotKey::BottomHand(Side::Left))));
		"bottom left"
	)]
	fn insert(name: &str, expected: Option<&SpawnerFixPoint>) {
		let mut app = setup();
		let entity = app.world_mut().spawn(Name::from(name)).id();

		app.update();

		assert_eq!(
			expected,
			app.world().entity(entity).get::<SpawnerFixPoint>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Name::from("skill_spawn")).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<SpawnerFixPoint>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SpawnerFixPoint>());
	}

	#[test]
	fn act_again_if_name_changed() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Name::from("skill_spawn")).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<SpawnerFixPoint>()
			.get_mut::<Name>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&SpawnerFixPoint(Spawner::Center)),
			app.world().entity(entity).get::<SpawnerFixPoint>()
		);
	}

	#[test]
	fn remove_fix_point_when_name_becomes_invalid() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Name::from("skill_spawn")).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Name::from("unicorn"));
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SpawnerFixPoint>());
	}
}
