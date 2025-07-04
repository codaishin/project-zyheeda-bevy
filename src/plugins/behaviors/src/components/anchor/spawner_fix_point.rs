use bevy::prelude::*;
use common::{
	tools::action_key::slot::{Side, SlotKey},
	traits::{
		handles_skill_behaviors::Spawner,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

use crate::components::anchor::AnchorFixPointKey;

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

impl From<SpawnerFixPoint> for AnchorFixPointKey {
	fn from(SpawnerFixPoint(spawner): SpawnerFixPoint) -> Self {
		match spawner {
			Spawner::Center => AnchorFixPointKey::new::<SpawnerFixPoint>(0),
			Spawner::Slot(SlotKey::BottomHand(Side::Left)) => {
				AnchorFixPointKey::new::<SpawnerFixPoint>(1)
			}
			Spawner::Slot(SlotKey::BottomHand(Side::Right)) => {
				AnchorFixPointKey::new::<SpawnerFixPoint>(2)
			}
			Spawner::Slot(SlotKey::TopHand(Side::Left)) => {
				AnchorFixPointKey::new::<SpawnerFixPoint>(3)
			}
			Spawner::Slot(SlotKey::TopHand(Side::Right)) => {
				AnchorFixPointKey::new::<SpawnerFixPoint>(4)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::iteration::IterFinite;
	use std::{any::TypeId, collections::HashSet};
	use test_case::test_case;
	use testing::SingleThreadedApp;

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

	#[test]
	fn spawner_to_anchor_fix_point_key_has_no_duplicate_values() {
		let slot_keys = SlotKey::iterator()
			.map(Spawner::Slot)
			.chain(std::iter::once(Spawner::Center))
			.map(SpawnerFixPoint)
			.collect::<Vec<_>>();

		let anchor_keys = slot_keys
			.iter()
			.copied()
			.map(AnchorFixPointKey::from)
			.collect::<HashSet<_>>();

		assert_eq!(slot_keys.len(), anchor_keys.len());
	}

	#[test]
	fn spawner_to_anchor_fix_point_key_has_correct_source() {
		let slot_keys = SlotKey::iterator()
			.map(Spawner::Slot)
			.chain(std::iter::once(Spawner::Center))
			.map(SpawnerFixPoint);

		let mut anchor_keys = slot_keys.map(AnchorFixPointKey::from);

		assert!(anchor_keys.all(|key| key.source_type == TypeId::of::<SpawnerFixPoint>()));
	}
}
