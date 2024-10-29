use crate::traits::MoveTogether;
use bevy::prelude::*;

pub(crate) fn move_with_target<TFollow: MoveTogether + Component>(
	targets: Query<&Transform, Without<TFollow>>,
	mut follower: Query<(&mut Transform, &mut TFollow)>,
) {
	for (mut transform, mut follower) in &mut follower {
		let Some(target) = follower.entity() else {
			continue;
		};
		let Ok(target) = targets.get(target) else {
			continue;
		};

		follower.move_together_with(transform.as_mut(), target.translation);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, NestedMocks)]
	struct _Move {
		pub mock: Mock_Move,
	}

	#[automock]
	impl MoveTogether for _Move {
		fn entity(&self) -> Option<Entity> {
			self.mock.entity()
		}
		fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3) {
			self.mock.move_together_with(transform, new_position)
		}
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn do_follow() {
		let mut app = setup();
		let target = app
			.world_mut()
			.spawn(Transform::from_translation(Vec3::new(1., 2., 3.)))
			.id();
		app.world_mut().spawn((
			_Move::new().with_mock(|mock| {
				mock.expect_entity().return_const(target);
				mock.expect_move_together_with()
					.with(
						eq(Transform::from_xyz(10., 10., 10.)),
						eq(Vec3::new(1., 2., 3.)),
					)
					.times(1)
					.return_const(());
			}),
			Transform::from_xyz(10., 10., 10.),
		));

		app.world_mut().run_system_once(move_with_target::<_Move>);
	}

	#[test]
	fn do_follow_for_multiple_followers() {
		let mut app = setup();
		let target = app
			.world_mut()
			.spawn(Transform::from_translation(Vec3::new(1., 2., 3.)))
			.id();
		app.world_mut().spawn((
			_Move::new().with_mock(|mock| {
				mock.expect_entity().return_const(target);
				mock.expect_move_together_with()
					.with(
						eq(Transform::from_xyz(10., 10., 10.)),
						eq(Vec3::new(1., 2., 3.)),
					)
					.times(1)
					.return_const(());
			}),
			Transform::from_xyz(10., 10., 10.),
		));
		app.world_mut().spawn((
			_Move::new().with_mock(|mock| {
				mock.expect_entity().return_const(target);
				mock.expect_move_together_with()
					.with(
						eq(Transform::from_xyz(11., 11., 11.)),
						eq(Vec3::new(1., 2., 3.)),
					)
					.times(1)
					.return_const(());
			}),
			Transform::from_xyz(11., 11., 11.),
		));

		app.world_mut().run_system_once(move_with_target::<_Move>);
	}
}
