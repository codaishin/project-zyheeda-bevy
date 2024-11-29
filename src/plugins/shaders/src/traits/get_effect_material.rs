mod gravity;

use bevy::prelude::*;

pub(crate) trait GetEffectMaterial {
	type TMaterial: Asset + Material;

	fn get_effect_material(&self) -> Self::TMaterial;
}
