use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	components::persistent_entity::PersistentEntity,
	resources::persistent_entities::{GetPersistentEntity, PersistentEntities},
	tools::UnitsPerSecond,
	traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct SetVelocityForward {
	pub(crate) rotation: PersistentEntity,
	pub(crate) speed: UnitsPerSecond,
}

impl SetVelocityForward {
	pub(crate) fn system(
		commands: Commands,
		set_velocities: Query<(Entity, &Self)>,
		transforms: Query<&Transform>,
		persistent_entities: ResMut<PersistentEntities>,
	) {
		Self::system_internal(commands, set_velocities, transforms, persistent_entities)
	}

	fn system_internal<TPersistentEntities>(
		mut commands: Commands,
		set_velocities: Query<(Entity, &Self)>,
		transforms: Query<&Transform>,
		mut persistent_entities: ResMut<TPersistentEntities>,
	) where
		TPersistentEntities: Resource + GetPersistentEntity,
	{
		for (entity, set_velocity) in &set_velocities {
			let Some(rotation) = persistent_entities.get_entity(&set_velocity.rotation) else {
				continue;
			};
			let Ok(rotation) = transforms.get(rotation) else {
				continue;
			};
			let movement = rotation.forward() * *set_velocity.speed;
			commands.try_insert_on(entity, Velocity::linear(movement));
			commands.try_remove_from::<SetVelocityForward>(entity);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use bevy_rapier3d::prelude::Velocity;
	use common::{
		assert_eq_approx,
		test_tools::utils::SingleThreadedApp,
		traits::{clamp_zero_positive::ClampZeroPositive, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Resource, NestedMocks)]
	struct _PersistentEntities {
		mock: Mock_PersistentEntities,
	}

	#[automock]
	impl GetPersistentEntity for _PersistentEntities {
		fn get_entity(&mut self, id: &PersistentEntity) -> Option<Entity> {
			self.mock.get_entity(id)
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Movement;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn use_correct_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		let rotation = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y))
			.id();
		app.world_mut()
			.insert_resource(_PersistentEntities::new().with_mock(|mock| {
				mock.expect_get_entity()
					.times(1)
					.with(eq(persistent_entity))
					.return_const(rotation);
			}));
		app.world_mut().spawn(SetVelocityForward {
			rotation: persistent_entity,
			speed: UnitsPerSecond::new(1.),
		});

		app.world_mut()
			.run_system_once(SetVelocityForward::system_internal::<_PersistentEntities>)
			.map(|_| {})
	}

	#[test]
	fn insert_velocity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		let rotation = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y))
			.id();
		app.world_mut()
			.insert_resource(_PersistentEntities::new().with_mock(|mock| {
				mock.expect_get_entity().return_const(rotation);
			}));
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward {
				rotation: persistent_entity,
				speed: UnitsPerSecond::new(1.),
			})
			.id();

		app.world_mut()
			.run_system_once(SetVelocityForward::system_internal::<_PersistentEntities>)?;

		assert_eq_approx!(
			Some(&Velocity::linear(Vec3::new(1., 2., 3.).normalize())),
			app.world().entity(entity).get::<Velocity>(),
			0.00001
		);
		Ok(())
	}

	#[test]
	fn insert_velocity_scaled_by_speed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		let rotation = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y))
			.id();
		app.world_mut()
			.insert_resource(_PersistentEntities::new().with_mock(|mock| {
				mock.expect_get_entity().return_const(rotation);
			}));
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward {
				rotation: persistent_entity,
				speed: UnitsPerSecond::new(10.),
			})
			.id();

		app.world_mut()
			.run_system_once(SetVelocityForward::system_internal::<_PersistentEntities>)?;

		assert_eq_approx!(
			Some(&Velocity::linear(Vec3::new(1., 2., 3.).normalize() * 10.)),
			app.world().entity(entity).get::<Velocity>(),
			0.00001
		);
		Ok(())
	}

	#[test]
	fn remove_velocity_setter() -> Result<(), RunSystemError> {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		let rotation = app
			.world_mut()
			.spawn(Transform::default().looking_to(Vec3::new(1., 2., 3.), Vec3::Y))
			.id();
		app.world_mut()
			.insert_resource(_PersistentEntities::new().with_mock(|mock| {
				mock.expect_get_entity().return_const(rotation);
			}));
		let entity = app
			.world_mut()
			.spawn(SetVelocityForward {
				rotation: persistent_entity,
				speed: UnitsPerSecond::new(10.),
			})
			.id();

		app.world_mut()
			.run_system_once(SetVelocityForward::system_internal::<_PersistentEntities>)?;

		assert_eq!(None, app.world().entity(entity).get::<SetVelocityForward>());
		Ok(())
	}
}
