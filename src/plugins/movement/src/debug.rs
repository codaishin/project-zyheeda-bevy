use crate::{
	MovementSystems,
	components::{
		movement_path::{Mode, MovementPath},
		ongoing_movement::OngoingMovement,
	},
};
use bevy::{color::palettes::css::LIGHT_CYAN, prelude::*};
use common::traits::handles_movement::MovementTarget;

pub(crate) fn draw(app: &mut App) {
	app.add_systems(Update, draw_path.after(MovementSystems));
}

#[allow(clippy::type_complexity)]
fn draw_path(paths: Query<(&Transform, &OngoingMovement, &MovementPath)>, mut gizmos: Gizmos) {
	for (transform, movement, path) in paths {
		let mut current = match movement {
			OngoingMovement::Stopped => continue,
			OngoingMovement::Target(MovementTarget::Dir(direction)) => {
				let target = transform.translation + **direction;
				gizmos.arrow(transform.translation, target, LIGHT_CYAN);
				continue;
			}
			OngoingMovement::Target(MovementTarget::Point(point)) => {
				gizmos.arrow(transform.translation, *point, LIGHT_CYAN);
				*point
			}
		};

		let Mode::Path(remaining_path) = &path.0 else {
			continue;
		};

		for wp in remaining_path {
			gizmos.arrow(current, *wp, LIGHT_CYAN);
			current = *wp;
		}
	}
}
