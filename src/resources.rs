pub mod skill_templates;

use crate::{
	components::SlotKey,
	types::{File, Key, SceneId},
};
use bevy::prelude::*;
use std::{collections::HashMap, hash::Hash, marker::PhantomData};

#[derive(Resource)]
pub struct Animation<TAgent, TMarker> {
	phantom_agent: PhantomData<TAgent>,
	phantom_marker: PhantomData<TMarker>,
	pub clip: Handle<AnimationClip>,
}

impl<TAgent, TMarker> Animation<TAgent, TMarker> {
	pub fn new(clip: Handle<AnimationClip>) -> Self {
		Self {
			phantom_agent: PhantomData,
			phantom_marker: PhantomData,
			clip,
		}
	}
}

#[derive(Resource)]
pub struct Models(pub HashMap<&'static Key, Handle<Scene>>);

impl Models {
	pub fn new<const C: usize>(
		pairs: [(&'static Key, &File, SceneId); C],
		asset_server: &Res<AssetServer>,
	) -> Self {
		Models(
			pairs
				.map(|(key, file, scene_id)| {
					(
						key,
						asset_server.load(format!("models/{file}#Scene{scene_id}")),
					)
				})
				.into_iter()
				.collect(),
		)
	}
}

#[derive(Resource)]
pub struct SkillIcons(pub HashMap<&'static str, Handle<Image>>);

type UIInputDisplay = &'static str;

#[derive(Resource)]
pub struct SlotMap<TButton>
where
	TButton: Eq + Hash,
{
	pub slots: HashMap<TButton, SlotKey>,
	pub ui_input_display: HashMap<SlotKey, UIInputDisplay>,
}

impl<TButton> SlotMap<TButton>
where
	TButton: Copy + Eq + Hash,
{
	pub fn new<const N: usize>(init: [(TButton, SlotKey, UIInputDisplay); N]) -> Self {
		let mut map = Self {
			slots: [].into(),
			ui_input_display: [].into(),
		};

		for (button, slot_key, ui_input_display) in &init {
			map.slots.insert(*button, *slot_key);
			map.ui_input_display.insert(*slot_key, ui_input_display);
		}

		map
	}
}

#[cfg(test)]
mod test_slot_map {
	use super::*;

	#[test]
	fn init_slots() {
		let map = SlotMap::new([
			(KeyCode::A, SlotKey::Legs, ""),
			(KeyCode::B, SlotKey::SkillSpawn, ""),
		]);

		assert_eq!(
			HashMap::from([
				(KeyCode::A, SlotKey::Legs),
				(KeyCode::B, SlotKey::SkillSpawn)
			]),
			map.slots
		)
	}

	#[test]
	fn init_ui_input_display() {
		let map = SlotMap::new([
			(KeyCode::A, SlotKey::Legs, "A"),
			(KeyCode::B, SlotKey::SkillSpawn, "B"),
		]);

		assert_eq!(
			HashMap::from([(SlotKey::Legs, "A"), (SlotKey::SkillSpawn, "B")]),
			map.ui_input_display
		)
	}
}
