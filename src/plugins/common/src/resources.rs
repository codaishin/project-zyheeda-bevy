pub mod key_map;
pub mod language_server;

use crate::{components::Outdated, traits::cache::Storage};
use bevy::{
	asset::{Asset, AssetServer, Handle, LoadedFolder},
	ecs::{
		component::Component,
		entity::Entity,
		system::{Res, Resource},
	},
	math::Ray3d,
	scene::Scene,
};
use std::{
	collections::{
		hash_map::Entry::{Occupied, Vacant},
		HashMap,
		HashSet,
	},
	fmt::Debug,
	hash::Hash,
	marker::PhantomData,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ColliderInfo<T> {
	pub collider: T,
	pub root: Option<T>,
}

impl ColliderInfo<Entity> {
	pub fn with_component<TComponent: Component + Clone>(
		&self,
		get_component: impl Fn(Entity) -> Option<TComponent>,
	) -> Option<ColliderInfo<Outdated<TComponent>>> {
		Some(ColliderInfo {
			collider: Outdated {
				component: get_component(self.collider)?,
				entity: self.collider,
			},
			root: self.root.and_then(|root| {
				Some(Outdated {
					component: get_component(root)?,
					entity: root,
				})
			}),
		})
	}
}

#[derive(Resource, Debug, PartialEq, Clone)]
pub struct MouseHover<T = Entity>(pub Option<ColliderInfo<T>>);

impl<T> Default for MouseHover<T> {
	fn default() -> Self {
		Self(None)
	}
}

#[derive(Resource, Default)]
pub struct CamRay(pub Option<Ray3d>);

#[derive(Resource, Default)]
pub struct Models(pub HashMap<&'static str, Handle<Scene>>);

pub type File = str;
pub type SceneId = u8;

impl Models {
	pub fn new<const C: usize>(
		pairs: [(&'static str, &File, SceneId); C],
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
pub struct Shared<TKey: Eq + Hash, T: Clone> {
	map: HashMap<TKey, T>,
}

impl<TKey: Eq + Hash + Debug, T: Clone + Debug> Debug for Shared<TKey, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Shared").field("map", &self.map).finish()
	}
}

impl<TKey: Eq + Hash + PartialEq, T: Clone + PartialEq> PartialEq for Shared<TKey, T> {
	fn eq(&self, other: &Self) -> bool {
		self.map == other.map
	}
}

impl<TKey: Eq + Hash, T: Clone> Shared<TKey, T> {
	pub fn new<const N: usize>(values: [(TKey, T); N]) -> Self {
		Self {
			map: HashMap::from(values),
		}
	}
}

impl<TKey: Eq + Hash, T: Clone> Default for Shared<TKey, T> {
	fn default() -> Self {
		Self {
			map: HashMap::default(),
		}
	}
}

impl<TKey: Eq + Hash, T: Clone> Shared<TKey, T> {
	pub fn get(&self, key: &TKey) -> Option<&T> {
		self.map.get(key)
	}
}

impl<TKey: Eq + Hash, T: Clone> Storage<TKey, T> for Shared<TKey, T> {
	fn get_or_create(&mut self, key: TKey, create: impl FnOnce() -> T) -> T {
		match self.map.entry(key) {
			Occupied(entry) => entry.get().clone(),
			Vacant(entry) => entry.insert(create()).clone(),
		}
	}
}

impl<TKey: Eq + Hash, T: Clone> From<HashMap<TKey, T>> for Shared<TKey, T> {
	fn from(map: HashMap<TKey, T>) -> Self {
		Self { map }
	}
}

#[derive(Resource, Debug, PartialEq)]
pub struct AssetFolder<TAsset: Asset> {
	phantom_data: PhantomData<TAsset>,
	pub folder: Handle<LoadedFolder>,
}

impl<TAsset: Asset> AssetFolder<TAsset> {
	pub(crate) fn new(folder: Handle<LoadedFolder>) -> Self {
		Self {
			phantom_data: PhantomData,
			folder,
		}
	}
}

#[derive(Resource, Debug, PartialEq)]
pub struct AliveAssets<TAsset: Asset>(HashSet<Handle<TAsset>>);

impl<TAsset: Asset> Default for AliveAssets<TAsset> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<TAsset: Asset> AliveAssets<TAsset> {
	#[cfg(test)]
	pub(crate) fn iter(&self) -> impl Iterator<Item = &Handle<TAsset>> {
		self.0.iter()
	}

	pub(crate) fn insert(&mut self, handle: Handle<TAsset>) {
		self.0.insert(handle);
	}
}

#[cfg(test)]
mod test_shared_asset {
	use super::*;
	use bevy::{asset::AssetId, render::mesh::Mesh};
	use uuid::Uuid;

	#[test]
	fn get_new() {
		let mut called = false;
		let new_handle = Handle::<Mesh>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut shared_assets = Shared::<u32, Handle<Mesh>>::default();
		let handle = shared_assets.get_or_create(42, || {
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
		_ = shared_assets.get_or_create(42, || old_handle.clone());
		let handle = shared_assets.get_or_create(42, || {
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
		_ = shared_assets.get_or_create(42, Handle::default);
		let handle = shared_assets.get_or_create(43, || {
			called = true;
			new_handle.clone()
		});

		assert_eq!((new_handle, true), (handle, called));
	}
}
