use crate::components::movement_definition::MovementDefinition;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::{GetContextMut, GetProperty},
	animation::{Animations, SetMovementDirection},
	handles_movement::MovementTarget,
};

impl MovementDefinition {
	pub(crate) fn animate_movement_forward<TMovement, TAnimations>(
		mut animations: StaticSystemParam<TAnimations>,
		movements: Query<(Entity, &TMovement, &Transform), Changed<TMovement>>,
	) where
		TMovement: Component + GetProperty<Option<MovementTarget>>,
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: SetMovementDirection>,
	{
		for (entity, movement, transform) in &movements {
			let key = Animations { entity };

			let Some(forward) = get_forward_direction(movement, transform) else {
				continue;
			};
			let Some(mut animations) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			animations.set_movement_direction(forward);
		}
	}
}

fn get_forward_direction<TMovement>(movement: &TMovement, transform: &Transform) -> Option<Dir3>
where
	TMovement: GetProperty<Option<MovementTarget>>,
{
	match movement.get_property()? {
		MovementTarget::Dir(direction) => Some(direction),
		MovementTarget::Point(point) => Dir3::try_from(point - transform.translation).ok(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Movement(MovementTarget);

	impl GetProperty<Option<MovementTarget>> for _Movement {
		fn get_property(&self) -> Option<MovementTarget> {
			Some(self.0)
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Animations {
		mock: Mock_Animations,
	}

	#[automock]
	impl SetMovementDirection for _Animations {
		fn set_movement_direction(&mut self, direction: Dir3) {
			self.mock.set_movement_direction(direction);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			MovementDefinition::animate_movement_forward::<_Movement, Query<Mut<_Animations>>>,
		);

		app
	}

	#[test]
	fn set_forward_from_direction() {
		let mut app = setup();
		app.world_mut().spawn((
			_Movement(MovementTarget::Dir(Dir3::NEG_X)),
			_Animations::new().with_mock(|mock| {
				mock.expect_set_movement_direction()
					.times(1)
					.with(eq(Dir3::NEG_X))
					.return_const(());
			}),
			Transform::default(),
		));

		app.update();
	}

	#[test]
	fn set_forward_from_target_point() {
		let mut app = setup();
		app.world_mut().spawn((
			_Movement(MovementTarget::Point(Vec3::new(1., 2., 3.))),
			_Animations::new().with_mock(|mock| {
				mock.expect_set_movement_direction()
					.times(1)
					.with(eq(Dir3::NEG_X))
					.return_const(());
			}),
			Transform::from_xyz(2., 2., 3.),
		));

		app.update();
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		app.world_mut().spawn((
			_Movement(MovementTarget::Dir(Dir3::NEG_X)),
			_Animations::new().with_mock(|mock| {
				mock.expect_set_movement_direction()
					.times(1)
					.return_const(());
			}),
			Transform::default(),
		));

		app.update();
		app.update();
	}

	#[test]
	fn act_again_if_movement_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Movement(MovementTarget::Dir(Dir3::NEG_X)),
				_Animations::new().with_mock(|mock| {
					mock.expect_set_movement_direction()
						.times(2)
						.return_const(());
				}),
				Transform::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Movement>()
			.as_deref_mut();
		app.update();
	}
}
