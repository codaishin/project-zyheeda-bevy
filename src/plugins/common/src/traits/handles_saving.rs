use bevy::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

pub trait HandlesSaving {
	type TSaveEntityMarker: Component + Default;

	fn register_savable_component<TComponent>(app: &mut App)
	where
		TComponent: Component + Clone + Serialize + DeserializeOwned;

	fn register_savable_component_dto<TComponent, TDto>(app: &mut App)
	where
		TComponent: Component + Clone,
		TDto: From<TComponent> + Serialize + DeserializeOwned;
}
