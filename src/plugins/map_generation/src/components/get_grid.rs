use bevy::prelude::*;
use common::traits::accessors::get::Getter;

#[derive(Component, Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GetGrid(Option<Entity>);

impl Getter<Option<Entity>> for GetGrid {
	fn get(&self) -> Option<Entity> {
		self.0
	}
}
