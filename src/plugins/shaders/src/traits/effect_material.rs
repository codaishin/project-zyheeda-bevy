use bevy::pbr::*;

pub(crate) trait EffectMaterial: Material {
	fn casts_shadows() -> bool;
}
