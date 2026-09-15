mod force;
mod gravity;
mod health_damage;

use crate::materials::effect_material::EffectMaterial;

pub(crate) trait ModifyMaterial {
	fn modify_material(material: &mut EffectMaterial);
}
