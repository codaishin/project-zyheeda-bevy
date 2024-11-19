use bevy::prelude::*;

pub trait RegisterForEffectShading {
	fn register_for_effect_shading<TComponent>(app: &mut App)
	where
		TComponent: Component;
}
