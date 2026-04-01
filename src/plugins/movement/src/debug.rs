use crate::{MovementSystems, components::movement::Movement};
use bevy::{color::palettes::css::LIGHT_CYAN, prelude::*};
use common::traits::{accessors::get::View, handles_physics::CharacterMotion};

pub(crate) fn draw<TMotion>(app: &mut App)
where
	TMotion: Component + View<CharacterMotion>,
{
	app.add_systems(Update, draw_path::<TMotion>.after(MovementSystems));
}

#[allow(clippy::type_complexity)]
fn draw_path<TMotion>(paths: Query<(&Transform, &TMotion, &Movement)>, mut gizmos: Gizmos)
where
	TMotion: Component + View<CharacterMotion>,
{
	for (transform, motion, movement) in paths {
		let mut current = match motion.view() {
			CharacterMotion::Stop => continue,
			CharacterMotion::Direction { direction, speed } => {
				let target = transform.translation + direction * *speed;
				gizmos.arrow(transform.translation, target, LIGHT_CYAN);
				continue;
			}
			CharacterMotion::ToTarget { target, .. } => {
				gizmos.arrow(transform.translation, target, LIGHT_CYAN);
				target
			}
		};

		let Movement::Path(remaining_path) = movement else {
			continue;
		};

		for wp in remaining_path {
			gizmos.arrow(current, *wp, LIGHT_CYAN);
			current = *wp;
		}
	}
}
