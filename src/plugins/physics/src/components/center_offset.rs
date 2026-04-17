use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct CenterOffset(pub(crate) f32);

pub(crate) trait ComputeOffsetTranslation {
	fn compute_translation(self, transform: &GlobalTransform) -> Vec3;
}

impl ComputeOffsetTranslation for Option<&CenterOffset> {
	fn compute_translation(self, transform: &GlobalTransform) -> Vec3 {
		match self {
			Some(CenterOffset(offset)) => transform.translation() + Vec3::new(0., *offset, 0.),
			None => transform.translation(),
		}
	}
}
