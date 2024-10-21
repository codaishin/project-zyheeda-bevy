use crate::{behaviors::SkillSpawner, slot_key::SlotKey};
use bevy::prelude::*;
use common::{
	components::Side,
	traits::{
		accessors::get::GetRef,
		track::{IsTracking, Track, Untrack},
	},
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct SkillSpawners<TMapping = PlayerSpawnsMapping> {
	spawners: HashMap<Option<SlotKey>, SkillSpawner>,
	phantom_data: PhantomData<TMapping>,
}

impl SkillSpawners {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(spawners: [(Option<SlotKey>, SkillSpawner); N]) -> Self {
		Self {
			spawners: HashMap::from(spawners),
			phantom_data: PhantomData,
		}
	}
}

impl<T: SlotKeyMapping> Default for SkillSpawners<T> {
	fn default() -> Self {
		Self {
			spawners: default(),
			phantom_data: default(),
		}
	}
}

impl GetRef<Option<SlotKey>, SkillSpawner> for SkillSpawners {
	fn get(&self, key: &Option<SlotKey>) -> Option<&SkillSpawner> {
		self.spawners.get(key)
	}
}

impl<TMapping> Track<Name> for SkillSpawners<TMapping>
where
	TMapping: SlotKeyMapping,
{
	fn track(&mut self, entity: Entity, name: &Name) {
		let Ok(key) = TMapping::get_slot_key(name) else {
			return;
		};

		self.spawners.insert(key, SkillSpawner(entity));
	}
}

impl IsTracking<Name> for SkillSpawners {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.spawners
			.values()
			.any(|SkillSpawner(tracked)| tracked == entity)
	}
}

impl Untrack<Name> for SkillSpawners {
	fn untrack(&mut self, entity: &Entity) {
		self.spawners
			.retain(|_, SkillSpawner(tracked)| tracked != entity);
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct PlayerSpawnsMapping;

#[derive(Debug, PartialEq)]
pub(crate) struct NotMapped;

pub(crate) trait SlotKeyMapping {
	fn get_slot_key(name: &Name) -> Result<Option<SlotKey>, NotMapped>;
}

impl SlotKeyMapping for PlayerSpawnsMapping {
	fn get_slot_key(name: &Name) -> Result<Option<SlotKey>, NotMapped> {
		match name.as_str() {
			"skill_spawn" => Ok(None),
			"skill_spawn_top.R" => Ok(Some(SlotKey::TopHand(Side::Right))),
			"skill_spawn_top.L" => Ok(Some(SlotKey::TopHand(Side::Left))),
			"skill_spawn_bottom.R" => Ok(Some(SlotKey::BottomHand(Side::Right))),
			"skill_spawn_bottom.L" => Ok(Some(SlotKey::BottomHand(Side::Left))),
			_ => Err(NotMapped),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::components::Side;

	#[test]
	fn name_to_slot_key() {
		let names = [
			"invalid",
			"skill_spawn",
			"skill_spawn_top.R",
			"skill_spawn_top.L",
			"skill_spawn_bottom.R",
			"skill_spawn_bottom.L",
		]
		.map(Name::from);

		assert_eq!(
			[
				Err(NotMapped),
				Ok(None),
				Ok(Some(SlotKey::TopHand(Side::Right))),
				Ok(Some(SlotKey::TopHand(Side::Left))),
				Ok(Some(SlotKey::BottomHand(Side::Right))),
				Ok(Some(SlotKey::BottomHand(Side::Left))),
			],
			names.map(|name| PlayerSpawnsMapping::get_slot_key(&name))
		)
	}

	#[test]
	fn track_when_name_mappable() {
		struct _Mapping;

		impl SlotKeyMapping for _Mapping {
			fn get_slot_key(_: &Name) -> Result<Option<SlotKey>, NotMapped> {
				Ok(Some(SlotKey::TopHand(Side::Right)))
			}
		}

		let mut spawners = SkillSpawners::<_Mapping>::default();
		spawners.track(Entity::from_raw(1), &Name::from(""));

		assert_eq!(
			HashMap::from([(
				Some(SlotKey::TopHand(Side::Right)),
				SkillSpawner(Entity::from_raw(1))
			)]),
			spawners.spawners,
		)
	}

	#[test]
	fn track_when_names_mappable() {
		struct _Mapping;

		impl SlotKeyMapping for _Mapping {
			fn get_slot_key(name: &Name) -> Result<Option<SlotKey>, NotMapped> {
				match name.as_str() {
					"a" => Ok(Some(SlotKey::TopHand(Side::Right))),
					"b" => Ok(Some(SlotKey::BottomHand(Side::Left))),
					"c" => Ok(None),
					_ => Err(NotMapped),
				}
			}
		}

		let mut spawners = SkillSpawners::<_Mapping>::default();
		spawners.track(Entity::from_raw(10), &Name::from("a"));
		spawners.track(Entity::from_raw(20), &Name::from("b"));
		spawners.track(Entity::from_raw(30), &Name::from("c"));

		assert_eq!(
			HashMap::from([
				(
					Some(SlotKey::TopHand(Side::Right)),
					SkillSpawner(Entity::from_raw(10))
				),
				(
					Some(SlotKey::BottomHand(Side::Left)),
					SkillSpawner(Entity::from_raw(20))
				),
				(None, SkillSpawner(Entity::from_raw(30))),
			]),
			spawners.spawners,
		)
	}

	#[test]
	fn is_tracking() {
		let spawners = SkillSpawners {
			spawners: HashMap::from([(None, SkillSpawner(Entity::from_raw(10)))]),
			..default()
		};

		assert!(spawners.is_tracking(&Entity::from_raw(10)));
	}

	#[test]
	fn is_not_tracking() {
		let spawners = SkillSpawners {
			spawners: HashMap::from([(None, SkillSpawner(Entity::from_raw(11)))]),
			..default()
		};

		assert!(!spawners.is_tracking(&Entity::from_raw(10)));
	}

	#[test]
	fn untrack() {
		let mut spawners = SkillSpawners {
			spawners: HashMap::from([(None, SkillSpawner(Entity::from_raw(10)))]),
			..default()
		};
		spawners.untrack(&Entity::from_raw(10));

		assert!(spawners.spawners.is_empty());
	}

	#[test]
	fn untrack_only_matching() {
		let mut spawners = SkillSpawners {
			spawners: HashMap::from([
				(None, SkillSpawner(Entity::from_raw(100))),
				(
					Some(SlotKey::TopHand(Side::Left)),
					SkillSpawner(Entity::from_raw(200)),
				),
			]),
			..default()
		};
		spawners.untrack(&Entity::from_raw(200));

		assert_eq!(
			HashMap::from([(None, SkillSpawner(Entity::from_raw(100)))]),
			spawners.spawners
		);
	}

	#[test]
	fn get_entity() {
		let spawners = SkillSpawners {
			spawners: HashMap::from([
				(None, SkillSpawner(Entity::from_raw(100))),
				(
					Some(SlotKey::TopHand(Side::Left)),
					SkillSpawner(Entity::from_raw(200)),
				),
			]),
			..default()
		};

		assert_eq!(
			[
				Some(&SkillSpawner(Entity::from_raw(100))),
				Some(&SkillSpawner(Entity::from_raw(200))),
				None
			],
			[
				spawners.get(&None),
				spawners.get(&Some(SlotKey::TopHand(Side::Left))),
				spawners.get(&Some(SlotKey::TopHand(Side::Right)))
			],
		)
	}
}
