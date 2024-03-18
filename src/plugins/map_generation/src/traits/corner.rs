use super::Definition;
use crate::components::Corner;
use bevy::ecs::system::EntityCommands;
use bevy_rapier3d::geometry::Collider;
use common::components::NoTarget;

const SIDES: [&str; 4] = ["NXNZ", "NXPZ", "PXPZ", "PXNZ"];
const SUFFIXES: [&str; 3] = ["", "HalfLeft", "HalfRight"];

impl Definition for Corner {
	fn target_names() -> Vec<String> {
		let build_name = |side| move |suffix| format!("Corner{}Wall{}Data", side, suffix);
		let build_names = |side| SUFFIXES.iter().map(build_name(side));

		SIDES.iter().flat_map(build_names).collect()
	}

	fn insert_bundle(entity: &mut EntityCommands) {
		entity.try_insert((Collider::cuboid(0.05, 1., 0.05), NoTarget));
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn target_names() {
		let mut all_names = vec![];
		for side in SIDES {
			for suffix in SUFFIXES {
				all_names.push("Corner".to_owned() + side + "Wall" + suffix + "Data");
			}
		}

		assert_eq!(all_names, Corner::target_names());
	}
}