pub mod skill_templates;
use crate::types::{File, Key, SceneId};
use bevy::{
	animation::AnimationClip,
	asset::{Asset, AssetServer, Handle},
	ecs::system::Resource,
	math::Ray,
	prelude::Res,
	render::mesh::Mesh,
	scene::Scene,
};
use std::{
	collections::{
		hash_map::Entry::{Occupied, Vacant},
		HashMap,
	},
	hash::Hash,
	marker::PhantomData,
};

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

#[derive(Resource, Default)]
pub struct CamRay(pub Option<Ray>);

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

#[derive(Resource)]
pub struct Shared<TKey: Eq + Hash, T: Clone> {
	map: HashMap<TKey, T>,
}

impl<TKey: Eq + Hash, T: Clone> Default for Shared<TKey, T> {
	fn default() -> Self {
		Self {
			map: HashMap::default(),
		}
	}
}

impl<TKey: Eq + Hash, T: Clone> Shared<TKey, T> {
	pub fn get_handle(&mut self, key: TKey, mut new_handle: impl FnMut() -> T) -> T {
		match self.map.entry(key) {
			Occupied(entry) => entry.get().clone(),
			Vacant(entry) => entry.insert(new_handle()).clone(),
		}
	}

	pub fn get(&self, key: &TKey) -> Option<&T> {
		self.map.get(key)
	}
}

#[cfg(test)]
mod test_shared_asset {
	use super::*;
	use bevy::{asset::AssetId, utils::Uuid};

	#[test]
	fn get_new() {
		let mut called = false;
		let new_handle = Handle::<Mesh>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut shared_assets = Shared::<u32, Handle<Mesh>>::default();
		let handle = shared_assets.get_handle(42, || {
			called = true;
			new_handle.clone()
		});

		assert_eq!((new_handle, true), (handle, called));
	}

	#[test]
	fn get_registered() {
		let mut called = false;
		let old_handle = Handle::<Mesh>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut shared_assets = Shared::<u32, Handle<Mesh>>::default();
		_ = shared_assets.get_handle(42, || old_handle.clone());
		let handle = shared_assets.get_handle(42, || {
			called = true;
			Handle::default()
		});

		assert_eq!((old_handle, false), (handle, called));
	}

	#[test]
	fn get_new_on_different_key() {
		let mut called = false;
		let new_handle = Handle::<Mesh>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut shared_assets = Shared::<u32, Handle<Mesh>>::default();
		_ = shared_assets.get_handle(42, Handle::default);
		let handle = shared_assets.get_handle(43, || {
			called = true;
			new_handle.clone()
		});

		assert_eq!((new_handle, true), (handle, called));
	}
}
