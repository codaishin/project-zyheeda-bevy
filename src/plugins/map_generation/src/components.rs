use bevy::{
	asset::{Asset, Handle},
	ecs::system::Resource,
};

pub(crate) struct Wall;

pub(crate) struct Corner;

pub(crate) struct Corridor;

impl Corridor {
	pub const MODEL_PATH_PREFIX: &'static str = "models/corridor_";
}

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct LoadLevelCommand<TMap: Asset>(pub Handle<TMap>);
