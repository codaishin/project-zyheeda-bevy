use super::ExtraComponentsDefinition;
use crate::components::{Unlit, Wall, WallBack};
use bevy::ecs::system::EntityCommands;
use bevy_rapier3d::geometry::Collider;
use common::{blocker::Blocker, components::NoTarget};

impl ExtraComponentsDefinition for Wall {
	fn target_names() -> Vec<String> {
		vec![]
	}

	fn insert_bundle<TLights>(entity: &mut EntityCommands) {
		entity.try_insert((
			Blocker::insert([Blocker::Physical]),
			Collider::cuboid(1., 1., 0.05),
			NoTarget,
		));
	}
}

const WALL_PARTS: &[&str] = &["Floor", "Forward", "Left", "CornerOutside", "CornerInside"];

impl ExtraComponentsDefinition for WallBack {
	fn target_names() -> Vec<String> {
		WALL_PARTS
			.iter()
			.map(|part| format!("Wall{part}BackData"))
			.collect()
	}

	fn insert_bundle<TLights>(entity: &mut EntityCommands) {
		entity.try_insert(Unlit);
	}
}
