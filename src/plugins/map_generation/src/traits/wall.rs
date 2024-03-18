use super::{Definition, ForChildren};
use crate::components::Wall;
use bevy_rapier3d::geometry::Collider;
use common::components::NoTarget;

impl Definition<(Collider, NoTarget)> for Wall {
	fn target_names() -> Vec<String> {
		vec![
			"WallNZ".to_owned(),
			"WallNX".to_owned(),
			"WallPZ".to_owned(),
			"WallPX".to_owned(),
		]
	}

	fn bundle() -> ((Collider, NoTarget), ForChildren) {
		(
			(Collider::cuboid(0.9, 1., 0.05), NoTarget),
			ForChildren::from(false),
		)
	}
}
