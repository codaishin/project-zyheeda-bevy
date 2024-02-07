pub mod skill_templates;
use crate::{
	components::SlotKey,
	types::{File, Key, SceneId},
};
use bevy::{
	animation::AnimationClip,
	asset::{Asset, AssetServer, Handle},
	ecs::{entity::Entity, system::Resource},
	math::Ray,
	prelude::Res,
	render::{mesh::Mesh, texture::Image},
	scene::Scene,
};
use std::{collections::HashMap, hash::Hash, marker::PhantomData};

#[derive(Resource)]
pub struct Animations<T>(pub HashMap<T, Handle<AnimationClip>>);

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

#[derive(Resource, Default)]
pub struct ModelData<TMaterial: Asset, TModel> {
	pub material: Handle<TMaterial>,
	pub mesh: Handle<Mesh>,
	phantom_data: PhantomData<TModel>,
}

impl<TMaterial: Asset, TModel> ModelData<TMaterial, TModel> {
	pub fn new(material: Handle<TMaterial>, mesh: Handle<Mesh>) -> Self {
		Self {
			material,
			mesh,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Resource)]
pub struct SkillIcons(pub HashMap<&'static str, Handle<Image>>);

type UIInputDisplay = &'static str;

#[derive(Resource, Debug, PartialEq, Clone)]
pub struct SlotMap<TButton>
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

	#[test]
	fn init_keys() {
		let map = SlotMap::new([
			(KeyCode::A, SlotKey::Legs, "A"),
			(KeyCode::B, SlotKey::SkillSpawn, "B"),
		]);

		assert_eq!(
			HashMap::from([
				(SlotKey::Legs, KeyCode::A),
				(SlotKey::SkillSpawn, KeyCode::B)
			]),
			map.keys
		)
	}
}

#[derive(Resource, Default)]
pub struct CamRay(pub Option<Ray>);

#[derive(Debug, PartialEq, Clone)]
pub struct ColliderInfo<T> {
	pub collider: T,
	pub root: Option<T>,
}

#[derive(Resource, Debug, PartialEq, Clone)]
pub struct MouseHover<T = Entity>(pub Option<ColliderInfo<T>>);

impl<T> Default for MouseHover<T> {
	fn default() -> Self {
		Self(None)
	}
}

#[derive(Resource)]
pub struct Prefab<TFor, TParent, TChildren> {
	pub parent: TParent,
	pub children: TChildren,
	phantom_data: PhantomData<TFor>,
}

impl<TFor, TParent, TChildren> Prefab<TFor, TParent, TChildren> {
	pub fn new(parent: TParent, children: TChildren) -> Self {
		Self {
			parent,
			children,
			phantom_data: PhantomData,
		}
	}
}
