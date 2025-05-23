mod app;

use bevy::prelude::*;

pub trait RegisterRequiredComponentsMapped {
	fn register_required_components_mapped<TComponent, TRequired>(
		&mut self,
		cstr: fn(&TComponent) -> TRequired,
	) -> &mut Self
	where
		TComponent: Component,
		TRequired: Component;
}
