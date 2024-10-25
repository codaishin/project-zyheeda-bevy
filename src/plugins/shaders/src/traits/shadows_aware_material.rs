use bevy::pbr::*;

pub(crate) trait ShadowsAwareMaterial: Material {
	fn casts_shadows() -> bool;
}
