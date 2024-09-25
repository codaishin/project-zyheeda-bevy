use bevy::prelude::*;

pub trait GetEffectMaterial {
	type TMaterial: Asset;

	fn get_effect_material(&self) -> Self::TMaterial;
}
