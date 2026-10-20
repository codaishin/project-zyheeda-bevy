use crate::{MovementSystems, components::movement::Movement};
use bevy::{
	color::palettes::css::LIGHT_CYAN,
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::prelude::*;

pub(crate) fn draw<TMotion, TInput>(app: &mut App)
where
	TMotion: Component + View<CharacterMotion>,
	TInput: ThreadSafe + HandlesInput + SystemSetDefinition,
{
	app.init_resource::<DrawPathGizmos>().add_systems(
		Update,
		(set_draw::<TInput::TInput>, draw_path::<TMotion>)
			.chain()
			.after(MovementSystems)
			.after_plugin(TInput::SYSTEMS),
	);
}

fn set_draw<TInput>(mut draw: ResMut<DrawPathGizmos>, input: Option<StaticSystemParam<TInput>>)
where
	TInput: for<'w, 's> SystemParam<Item<'w, 's>: GetInputState>,
{
	let Some(input) = input else {
		return;
	};

	let state = input.get_input_state(DebugKey::ToggleMovementPath);

	if state != InputState::just_pressed() {
		return;
	}

	*draw = !*draw;
}

#[allow(clippy::type_complexity)]
fn draw_path<TMotion>(
	draw: Res<DrawPathGizmos>,
	paths: Query<(&Transform, &TMotion, &Movement)>,
	mut gizmos: Gizmos,
) where
	TMotion: Component + View<CharacterMotion>,
{
	if *draw == DrawPathGizmos(false) {
		return;
	}

	for (transform, motion, movement) in paths {
		let mut current = match motion.view() {
			CharacterMotion::Done => continue,
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

		for wp in remaining_path.iter() {
			gizmos.arrow(current, *wp, LIGHT_CYAN);
			current = *wp;
		}
	}
}

#[derive(Resource, Debug, PartialEq, Default, Clone, Copy)]
struct DrawPathGizmos(bool);

impl std::ops::Not for DrawPathGizmos {
	type Output = DrawPathGizmos;

	fn not(self) -> Self::Output {
		let Self(draw) = self;
		Self(!draw)
	}
}
