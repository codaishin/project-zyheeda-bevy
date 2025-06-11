use bevy::prelude::*;

#[derive(Component)]
pub struct GroundOffset(pub Vec3);

impl From<Vec3> for GroundOffset {
	fn from(value: Vec3) -> Self {
		GroundOffset(value)
	}
}
