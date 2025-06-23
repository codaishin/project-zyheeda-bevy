use crate::traits::move_together::MoveTogether;
use bevy::{ecs::component::Mutable, prelude::*};
use common::resources::persistent_entities::PersistentEntities;

pub(crate) fn move_with_target<TFollow>(
	targets: Query<&Transform, Without<TFollow>>,
	mut follower: Query<(&mut Transform, &mut TFollow)>,
	mut persistent_entities: ResMut<PersistentEntities>,
) where
	TFollow: MoveTogether + Component<Mutability = Mutable>,
{
	for (mut transform, mut follower) in &mut follower {
		let Some(target) = follower.entity() else {
			continue;
		};
		let Some(target) = persistent_entities.get_entity(&target) else {
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
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		components::persistent_entity::PersistentEntity,
		test_tools::utils::SingleThreadedApp,
		traits::{
			nested_mock::NestedMocks,
			register_persistent_entities::RegisterPersistentEntities,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, NestedMocks)]
	struct _Move {
		pub mock: Mock_Move,
	}

	#[automock]
	impl MoveTogether for _Move {
		fn entity(&self) -> Option<PersistentEntity> {
			self.mock.entity()
		}
		fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3) {
			self.mock.move_together_with(transform, new_position)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();

		app
	}

	#[test]
	fn do_follow() -> Result<(), RunSystemError> {
		let mut app = setup();
		let target = PersistentEntity::default();
		app.world_mut()
			.spawn((Transform::from_translation(Vec3::new(1., 2., 3.)), target));
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

		app.world_mut().run_system_once(move_with_target::<_Move>)
	}

	#[test]
	fn do_follow_for_multiple_followers() -> Result<(), RunSystemError> {
		let mut app = setup();
		let target = PersistentEntity::default();
		app.world_mut()
			.spawn((Transform::from_translation(Vec3::new(1., 2., 3.)), target));
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

		app.world_mut().run_system_once(move_with_target::<_Move>)
	}
}
