use super::ExtraComponentsDefinition;
use crate::components::{Unlit, Wall, WallBack};
use bevy::ecs::system::EntityCommands;
use bevy_rapier3d::geometry::Collider;
use common::components::NoTarget;
use interactions::components::blocker::Blocker;

impl ExtraComponentsDefinition for Wall {
	fn target_names() -> Vec<String> {
		vec![
			"WallNZData".to_owned(),
			"WallNXData".to_owned(),
			"WallPZData".to_owned(),
			"WallPXData".to_owned(),
		]
	}

	fn insert_bundle(entity: &mut EntityCommands) {
		entity.try_insert((
			Blocker::insert([Blocker::Physical]),
			Collider::cuboid(1., 1., 0.05),
			NoTarget,
		));
	}
}

impl ExtraComponentsDefinition for WallBack {
	fn target_names() -> Vec<String> {
		vec![
			"WallNZBackData".to_owned(),
			"WallPZBackData".to_owned(),
			"WallNXBackData".to_owned(),
			"WallPXBackData".to_owned(),
		]
	}

	fn insert_bundle(entity: &mut EntityCommands) {
		entity.try_insert(Unlit);
	}
}
