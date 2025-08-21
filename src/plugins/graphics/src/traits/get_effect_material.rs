mod force;
mod gravity;
mod health_damage;

use bevy::prelude::*;

pub(crate) trait GetEffectMaterial {
	type TMaterial: Asset + Material;

	fn get_effect_material(first_pass: &Handle<Image>) -> Self::TMaterial;
}
