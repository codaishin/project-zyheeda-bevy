use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::{GetContextMut, View},
	handles_animations::{Animations, GetMoveDirectionMut},
	handles_physics::CharacterMotion,
};

impl<T> SetForwardAnimationDirection for T where T: Component + View<CharacterMotion> {}

pub(crate) trait SetForwardAnimationDirection:
	Component + View<CharacterMotion> + Sized
{
	fn animate_forward<TAnimations>(
		mut animations: StaticSystemParam<TAnimations>,
		movements: Query<(Entity, &Self, &Transform), Changed<Self>>,
	) where
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: GetMoveDirectionMut>,
	{
		for (entity, movement, transform) in &movements {
			let key = Animations { entity };

			let Some(forward) = get_forward_direction(movement, transform) else {
				continue;
			};
			let Some(mut animations) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			*animations.get_move_direction_mut() = Some(forward);
		}
	}
}

fn get_forward_direction<TMovement>(movement: &TMovement, transform: &Transform) -> Option<Dir3>
where
	TMovement: View<CharacterMotion>,
{
	match movement.view() {
		CharacterMotion::Direction { direction, .. } => Some(direction),
		CharacterMotion::ToTarget { target, .. } => {
			Dir3::try_from(target - transform.translation).ok()
		}
		CharacterMotion::Done => None,
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::{tools::speed::Speed, traits::handles_animations::GetMoveDirection};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Motion(CharacterMotion);

	impl View<CharacterMotion> for _Motion {
		fn view(&self) -> CharacterMotion {
			self.0
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Animations(Option<Dir3>);

	impl GetMoveDirection for _Animations {
		fn get_move_direction(&self) -> Option<Dir3> {
			self.0
		}
	}

	impl GetMoveDirectionMut for _Animations {
		fn get_move_direction_mut(&mut self) -> &mut Option<Dir3> {
			&mut self.0
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _Motion::animate_forward::<Query<Mut<_Animations>>>);

		app
	}

	#[test]
	fn set_forward_from_direction() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Motion(CharacterMotion::Direction {
					direction: Dir3::NEG_X,
					speed: Speed::ZERO,
				}),
				_Animations(None),
				Transform::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(Some(Dir3::NEG_X))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn set_forward_from_target_point() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Motion(CharacterMotion::ToTarget {
					target: Vec3::new(1., 2., 3.),
					speed: Speed::ZERO,
				}),
				_Animations(None),
				Transform::from_xyz(2., 2., 3.),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(Some(Dir3::NEG_X))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Motion(CharacterMotion::Direction {
					direction: Dir3::NEG_X,
					speed: Speed::ZERO,
				}),
				_Animations(None),
				Transform::default(),
			))
			.id();

		app.update();
		*app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Animations>()
			.unwrap() = _Animations(None);
		app.update();

		assert_eq!(
			Some(&_Animations(None)),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn act_again_if_motion_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Motion(CharacterMotion::Direction {
					direction: Dir3::NEG_X,
					speed: Speed::ZERO,
				}),
				_Animations(None),
				Transform::default(),
			))
			.id();

		app.update();
		*app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Animations>()
			.unwrap() = _Animations(None);
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Motion>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&_Animations(Some(Dir3::NEG_X))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}
}
