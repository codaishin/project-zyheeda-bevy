use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::{GetContextMut, GetProperty},
	handles_animations::{Animations, MoveDirectionMut},
	handles_physics::CharacterMotion,
};

impl<T> AnimateMovementForward for T where T: Component + GetProperty<CharacterMotion> {}

pub(crate) trait AnimateMovementForward:
	Component + GetProperty<CharacterMotion> + Sized
{
	fn animate_movement_forward<TAnimations>(
		mut animations: StaticSystemParam<TAnimations>,
		movements: Query<(Entity, &Self, &Transform), Changed<Self>>,
	) where
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: MoveDirectionMut>,
	{
		for (entity, movement, transform) in &movements {
			let key = Animations { entity };

			let Some(forward) = get_forward_direction(movement, transform) else {
				continue;
			};
			let Some(mut animations) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			*animations.move_direction_mut() = Some(forward);
		}
	}
}

fn get_forward_direction<TMovement>(movement: &TMovement, transform: &Transform) -> Option<Dir3>
where
	TMovement: GetProperty<CharacterMotion>,
{
	match movement.get_property() {
		CharacterMotion::Direction { direction, .. } => Some(direction),
		CharacterMotion::ToTarget { target, .. } => {
			Dir3::try_from(target - transform.translation).ok()
		}
		CharacterMotion::Stop => None,
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::{tools::speed::Speed, traits::handles_animations::MoveDirection};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Movement(CharacterMotion);

	impl GetProperty<CharacterMotion> for _Movement {
		fn get_property(&self) -> CharacterMotion {
			self.0
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Animations(Option<Dir3>);

	impl MoveDirection for _Animations {
		fn move_direction(&self) -> Option<Dir3> {
			self.0
		}
	}

	impl MoveDirectionMut for _Animations {
		fn move_direction_mut(&mut self) -> &mut Option<Dir3> {
			&mut self.0
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			_Movement::animate_movement_forward::<Query<Mut<_Animations>>>,
		);

		app
	}

	#[test]
	fn set_forward_from_direction() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Movement(CharacterMotion::Direction {
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
				_Movement(CharacterMotion::ToTarget {
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
				_Movement(CharacterMotion::Direction {
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
	fn act_again_if_movement_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Movement(CharacterMotion::Direction {
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
			.get_mut::<_Movement>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&_Animations(Some(Dir3::NEG_X))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}
}
