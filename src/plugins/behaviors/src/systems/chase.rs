use crate::components::{Chase, movement::Movement};
use bevy::prelude::*;
use common::{
	resources::persistent_entities::{GetPersistentEntity, PersistentEntities},
	traits::{thread_safe::ThreadSafe, try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};

impl<T> ChaseSystem for T {}

pub(crate) trait ChaseSystem {
	fn chase<TMovementMethod>(
		commands: Commands,
		chasers: Query<(Entity, &Chase), With<Self>>,
		removed_chasers: RemovedComponents<Chase>,
		transforms: Query<&GlobalTransform>,
		persistent_entities: ResMut<PersistentEntities>,
	) where
		Self: Component + Sized,
		TMovementMethod: ThreadSafe + Default,
	{
		chase::<TMovementMethod, Self, PersistentEntities>(
			commands,
			chasers,
			removed_chasers,
			transforms,
			persistent_entities,
		);
	}
}

fn chase<TMovementMethod, TAgent, TPersistentEntities>(
	mut commands: Commands,
	chasers: Query<(Entity, &Chase), With<TAgent>>,
	mut removed_chasers: RemovedComponents<Chase>,
	transforms: Query<&GlobalTransform>,
	mut persistent_entities: ResMut<TPersistentEntities>,
) where
	TAgent: Component + Sized,
	TMovementMethod: ThreadSafe + Default,
	TPersistentEntities: Resource + GetPersistentEntity,
{
	for entity in removed_chasers.read() {
		commands.try_remove_from::<Movement<TMovementMethod>>(entity);
	}

	for (entity, Chase(target)) in &chasers {
		let Some(target) = persistent_entities.get_entity(target) else {
			continue;
		};
		let Ok(target) = transforms.get(target) else {
			continue;
		};
		commands.try_insert_on(
			entity,
			Movement::<TMovementMethod>::to(target.translation()),
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::nested_mock::NestedMocks,
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

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq, Default)]
	struct _MovementMethod;

	fn setup(target_position: Vec3) -> (App, PersistentEntity) {
		let mut app = App::new();
		app.add_systems(
			Update,
			chase::<_MovementMethod, _Agent, _PersistentEntities>,
		);
		let persistent_target = PersistentEntity::default();
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_translation(target_position))
			.id();
		app.insert_resource(_PersistentEntities::new().with_mock(|mock| {
			mock.expect_get_entity()
				.with(eq(persistent_target))
				.return_const(target);
		}));

		(app, persistent_target)
	}

	#[test]
	fn set_movement_to_follow_target_when_chasing() {
		let target_position = Vec3::new(1., 2., 3.);
		let (mut app, target) = setup(target_position);
		let chaser = app
			.world_mut()
			.spawn((_Agent, GlobalTransform::default(), Chase(target)))
			.id();

		app.update();

		assert_eq!(
			Some(&Movement::<_MovementMethod>::to(target_position)),
			app.world()
				.entity(chaser)
				.get::<Movement<_MovementMethod>>()
		);
	}

	#[test]
	fn remove_movement_when_not_chasing() {
		let (mut app, target) = setup(Vec3::new(1., 2., 3.));
		let chaser = app
			.world_mut()
			.spawn((_Agent, GlobalTransform::from_xyz(3., 2., 1.), Chase(target)))
			.id();

		app.update();
		app.world_mut().entity_mut(chaser).remove::<Chase>();
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(chaser)
				.get::<Movement<_MovementMethod>>()
		);
	}
}
