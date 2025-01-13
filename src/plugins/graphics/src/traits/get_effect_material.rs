mod deal_damage;
mod force_shield;
mod gravity;

use bevy::prelude::*;

pub(crate) trait GetEffectMaterial {
	type TMaterial: Asset + Material;

	fn get_effect_material(first_pass: &Handle<Image>) -> Self::TMaterial;
}
