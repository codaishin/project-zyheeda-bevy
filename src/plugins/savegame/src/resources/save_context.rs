use bevy::prelude::*;
use serde::Serialize;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct SaveContext;

impl SaveContext {
	pub(crate) fn save<TComponent>(&mut self, _: &EntityRef)
	where
		TComponent: Component + Serialize + 'static,
	{
	}
}
