use std::marker::PhantomData;

use crate::map::Map;
use bevy::{
	asset::Handle,
	ecs::{component::Component, system::Resource},
	reflect::TypePath,
};

#[derive(Clone)]
pub(crate) struct Wall;

pub(crate) struct WallBack;

pub(crate) struct Corridor;

impl Corridor {
	pub const MODEL_PATH_PREFIX: &'static str = "models/corridor_";
}

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct LoadLevelCommand<TCell: TypePath + Send + Sync>(pub Handle<Map<TCell>>);

pub(crate) struct Floating;

#[derive(Component, Clone)]
pub(crate) struct Light<T>(PhantomData<T>);

impl<T> Default for Light<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

#[derive(Component)]
pub(crate) struct Unlit;
