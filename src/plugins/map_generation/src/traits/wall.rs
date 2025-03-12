use super::ExtraComponentsDefinition;
use crate::components::{Unlit, Wall, WallBack};
use bevy::ecs::system::EntityCommands;
use bevy_rapier3d::geometry::Collider;
use common::{blocker::Blocker, components::NoTarget};

impl ExtraComponentsDefinition for Wall {
	fn target_names() -> Vec<String> {
		vec!["HalfWallData".to_owned(), "WallCornerData".to_owned()]
	}

	fn insert_bundle<TLights>(entity: &mut EntityCommands) {
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
			"WallFloorData".to_owned(),
			"HalfWallBackData".to_owned(),
			"HalfWallRotatedBackData".to_owned(),
			"WallCornerBackData".to_owned(),
		]
	}

	fn insert_bundle<TLights>(entity: &mut EntityCommands) {
		entity.try_insert(Unlit);
	}
}
