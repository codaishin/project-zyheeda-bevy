use bevy::pbr::*;

pub(crate) trait ShadowsAwareMaterial: Material {
	fn shadows_enabled() -> bool;
}
