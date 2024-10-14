use crate::items::slot_key::SlotKey;
use bevy::prelude::*;
use common::{
	components::Side,
	traits::{
		get::Get,
		track::{IsTracking, Track, Untrack},
	},
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component)]
pub(crate) struct SkillSpawners<TMapping = PlayerSpawnsMapping>
where
	TMapping: SlotKeyMapping,
{
	entities: HashMap<Option<SlotKey>, Entity>,
	phantom_data: PhantomData<TMapping>,
}

impl<T: SlotKeyMapping> Default for SkillSpawners<T> {
	fn default() -> Self {
		Self {
			entities: default(),
			phantom_data: default(),
		}
	}
}

impl Get<Option<SlotKey>, Entity> for SkillSpawners {
	fn get(&self, key: &Option<SlotKey>) -> Option<&Entity> {
		self.entities.get(key)
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

		self.entities.insert(key, entity);
	}
}

impl IsTracking<Name> for SkillSpawners {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.entities.values().any(|tracked| tracked == entity)
	}
}

impl Untrack<Name> for SkillSpawners {
	fn untrack(&mut self, entity: &Entity) {
		self.entities.retain(|_, tracked| tracked != entity);
	}
}

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
			HashMap::from([(Some(SlotKey::TopHand(Side::Right)), Entity::from_raw(1))]),
			spawners.entities,
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
				(Some(SlotKey::TopHand(Side::Right)), Entity::from_raw(10)),
				(Some(SlotKey::BottomHand(Side::Left)), Entity::from_raw(20)),
				(None, Entity::from_raw(30)),
			]),
			spawners.entities,
		)
	}

	#[test]
	fn is_tracking() {
		let spawners = SkillSpawners {
			entities: HashMap::from([(None, Entity::from_raw(10))]),
			..default()
		};

		assert!(spawners.is_tracking(&Entity::from_raw(10)));
	}

	#[test]
	fn is_not_tracking() {
		let spawners = SkillSpawners {
			entities: HashMap::from([(None, Entity::from_raw(11))]),
			..default()
		};

		assert!(!spawners.is_tracking(&Entity::from_raw(10)));
	}

	#[test]
	fn untrack() {
		let mut spawners = SkillSpawners {
			entities: HashMap::from([(None, Entity::from_raw(10))]),
			..default()
		};
		spawners.untrack(&Entity::from_raw(10));

		assert!(spawners.entities.is_empty());
	}

	#[test]
	fn untrack_only_matching() {
		let mut spawners = SkillSpawners {
			entities: HashMap::from([
				(None, Entity::from_raw(100)),
				(Some(SlotKey::TopHand(Side::Left)), Entity::from_raw(200)),
			]),
			..default()
		};
		spawners.untrack(&Entity::from_raw(200));

		assert_eq!(
			HashMap::from([(None, Entity::from_raw(100))]),
			spawners.entities
		);
	}

	#[test]
	fn get_entity() {
		let spawners = SkillSpawners {
			entities: HashMap::from([
				(None, Entity::from_raw(100)),
				(Some(SlotKey::TopHand(Side::Left)), Entity::from_raw(200)),
			]),
			..default()
		};

		assert_eq!(
			[
				Some(&Entity::from_raw(100)),
				Some(&Entity::from_raw(200)),
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
