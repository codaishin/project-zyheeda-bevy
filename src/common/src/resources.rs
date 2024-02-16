use bevy::{
	asset::{AssetServer, Handle},
	ecs::{
		entity::Entity,
		system::{Res, Resource},
	},
	math::Ray,
	scene::Scene,
};
use std::collections::HashMap;

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

#[derive(Resource, Default)]
pub struct CamRay(pub Option<Ray>);

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
