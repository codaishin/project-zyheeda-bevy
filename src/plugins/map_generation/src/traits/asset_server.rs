use super::LoadMap;
use crate::map::Map;
use bevy::asset::{AssetServer, Handle};

impl LoadMap for AssetServer {
	fn load(&self) -> Handle<Map> {
		self.load("maps/map.txt")
	}
}
