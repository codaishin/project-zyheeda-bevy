use super::ColliderDefinition;
use crate::components::Wall;
use bevy_rapier3d::geometry::Collider;

impl ColliderDefinition for Wall {
	const IS_TARGET: bool = false;

	fn target_names() -> Vec<String> {
		vec![
			"WallNZ".to_owned(),
			"WallNX".to_owned(),
			"WallPZ".to_owned(),
			"WallPX".to_owned(),
		]
	}

	fn collider() -> Collider {
		Collider::cuboid(0.9, 1., 0.05)
	}
}
