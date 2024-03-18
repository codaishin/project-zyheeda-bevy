use super::Definition;
use crate::components::Wall;
use bevy_rapier3d::geometry::Collider;
use common::components::NoTarget;

impl Definition<(Collider, NoTarget)> for Wall {
	fn target_names() -> Vec<String> {
		vec![
			"WallNZData".to_owned(),
			"WallNXData".to_owned(),
			"WallPZData".to_owned(),
			"WallPXData".to_owned(),
		]
	}

	fn bundle() -> (Collider, NoTarget) {
		(Collider::cuboid(0.9, 1., 0.05), NoTarget)
	}
}
