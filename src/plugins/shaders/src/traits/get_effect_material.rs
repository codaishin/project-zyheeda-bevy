mod force;

use bevy::prelude::*;

pub(crate) trait GetEffectMaterial {
	type TMaterial: Asset;

	fn get_effect_material(&self) -> Self::TMaterial;
}
