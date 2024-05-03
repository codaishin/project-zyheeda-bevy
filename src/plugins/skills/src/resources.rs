use crate::components::SlotKey;
use bevy::{ecs::system::Resource, input::keyboard::KeyCode};
use std::{collections::HashMap, hash::Hash};

type UIInputDisplay = &'static str;

#[derive(Resource, Debug, PartialEq, Clone)]
pub struct SlotMap<TButton = KeyCode>
where
	TButton: Eq + Hash,
{
	pub slots: HashMap<TButton, SlotKey>,
	pub ui_input_display: HashMap<SlotKey, UIInputDisplay>,
	pub keys: HashMap<SlotKey, TButton>,
}

impl<TButton: Eq + Hash> Default for SlotMap<TButton> {
	fn default() -> Self {
		Self {
			slots: Default::default(),
			ui_input_display: Default::default(),
			keys: Default::default(),
		}
	}
}

impl<TButton> SlotMap<TButton>
where
	TButton: Copy + Eq + Hash,
{
	pub fn new<const N: usize>(init: [(TButton, SlotKey, UIInputDisplay); N]) -> Self {
		let mut map = Self {
			slots: [].into(),
			ui_input_display: [].into(),
			keys: [].into(),
		};

		for (button, slot_key, ui_input_display) in &init {
			map.slots.insert(*button, *slot_key);
			map.ui_input_display.insert(*slot_key, ui_input_display);
			map.keys.insert(*slot_key, *button);
		}

		map
	}
}

#[cfg(test)]
mod test_slot_map {
	use super::*;
	use bevy::input::keyboard::KeyCode;
	use common::components::Side;

	#[test]
	fn init_slots() {
		let map = SlotMap::new([
			(KeyCode::KeyA, SlotKey::Hand(Side::Main), ""),
			(KeyCode::KeyB, SlotKey::SkillSpawn, ""),
		]);

		assert_eq!(
			HashMap::from([
				(KeyCode::KeyA, SlotKey::Hand(Side::Main)),
				(KeyCode::KeyB, SlotKey::SkillSpawn)
			]),
			map.slots
		)
	}

	#[test]
	fn init_ui_input_display() {
		let map = SlotMap::new([
			(KeyCode::KeyA, SlotKey::Hand(Side::Main), "A"),
			(KeyCode::KeyB, SlotKey::SkillSpawn, "B"),
		]);

		assert_eq!(
			HashMap::from([(SlotKey::Hand(Side::Main), "A"), (SlotKey::SkillSpawn, "B")]),
			map.ui_input_display
		)
	}

	#[test]
	fn init_keys() {
		let map = SlotMap::new([
			(KeyCode::KeyA, SlotKey::Hand(Side::Main), "A"),
			(KeyCode::KeyB, SlotKey::SkillSpawn, "B"),
		]);

		assert_eq!(
			HashMap::from([
				(SlotKey::Hand(Side::Main), KeyCode::KeyA),
				(SlotKey::SkillSpawn, KeyCode::KeyB)
			]),
			map.keys
		)
	}
}
