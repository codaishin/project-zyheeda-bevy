use crate::map_loader::Map;
use bevy::{asset::Handle, ecs::system::Resource};

pub(crate) struct Wall;

pub(crate) struct Corner;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct LoadLevelCommand(pub Handle<Map>);
