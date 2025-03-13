use bevy::prelude::*;
use serde::Serialize;

pub trait HandlesSaving {
	type TSaveEntityMarker: Component;

	fn register_save_able_component<TComponent>(app: &mut App)
	where
		TComponent: Component + Serialize;
}
