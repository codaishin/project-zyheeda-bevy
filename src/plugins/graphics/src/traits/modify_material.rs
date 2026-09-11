mod force;
mod gravity;
mod health_damage;

use crate::components::effect_material_data::EffectMaterialData;

pub(crate) trait ModifyMaterial {
	fn modify_material(material: &mut EffectMaterialData);
}
