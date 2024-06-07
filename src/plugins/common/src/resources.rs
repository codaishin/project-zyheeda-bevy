pub mod key_map;
pub mod language_server;

use crate::{components::Outdated, traits::cache::Cache};
use bevy::{
	asset::{AssetServer, Handle},
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
	},
	hash::Hash,
};

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Resource)]
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

impl<TKey: Eq + Hash, T: Clone> Cache<TKey, T> for Shared<TKey, T> {
	fn cached(&mut self, key: TKey, new: impl FnOnce() -> T) -> T {
		match self.map.entry(key) {
			Occupied(entry) => entry.get().clone(),
			Vacant(entry) => entry.insert(new()).clone(),
		}
	}
}

#[cfg(test)]
mod test_shared_asset {
	use super::*;
	use bevy::{asset::AssetId, render::mesh::Mesh, utils::Uuid};

	#[test]
	fn get_new() {
		let mut called = false;
		let new_handle = Handle::<Mesh>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut shared_assets = Shared::<u32, Handle<Mesh>>::default();
		let handle = shared_assets.cached(42, || {
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
		_ = shared_assets.cached(42, || old_handle.clone());
		let handle = shared_assets.cached(42, || {
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
		_ = shared_assets.cached(42, Handle::default);
		let handle = shared_assets.cached(43, || {
			called = true;
			new_handle.clone()
		});

		assert_eq!((new_handle, true), (handle, called));
	}
}
