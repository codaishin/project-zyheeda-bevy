use super::ExtraComponentsDefinition;
use crate::components::{Unlit, WallBack};
use bevy::ecs::system::EntityCommands;

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
