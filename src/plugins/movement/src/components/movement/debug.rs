use crate::{
	MovementSystems,
	components::movement::path_or_direction::{Mode, PathOrDirection},
};
use bevy::{color::palettes::css::LIGHT_CYAN, prelude::*};
use common::traits::{accessors::get::GetProperty, handles_physics::CharacterMotion};

pub(crate) fn draw<TMotion>(app: &mut App)
where
	TMotion: GetProperty<CharacterMotion> + Component,
{
	app.add_systems(Update, draw_path::<TMotion>.after(MovementSystems));
}

#[allow(clippy::type_complexity)]
fn draw_path<TMotion>(
	paths: Query<(&Transform, &TMotion, &PathOrDirection<TMotion>)>,
	mut gizmos: Gizmos,
) where
	TMotion: GetProperty<CharacterMotion> + Component,
{
	for (transform, motion, path) in paths {
		let mut current = match motion.get_property() {
			CharacterMotion::Stop => continue,
			CharacterMotion::Direction { speed, direction } => {
				let target = transform.translation + *direction * *speed;
				gizmos.arrow(transform.translation, target, LIGHT_CYAN);
				continue;
			}
			CharacterMotion::ToTarget { target, .. } => {
				gizmos.arrow(transform.translation, target, LIGHT_CYAN);
				target
			}
		};

		let Mode::Path(remaining_path) = &path.mode else {
			continue;
		};

		for wp in remaining_path {
			gizmos.arrow(current, *wp, LIGHT_CYAN);
			current = *wp;
		}
	}
}
