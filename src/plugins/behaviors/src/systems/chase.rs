use crate::components::{Chase, movement::Movement};
use bevy::prelude::*;
use common::{
	resources::persistent_entities::PersistentEntities,
	traits::{thread_safe::ThreadSafe, try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};

impl<T> ChaseSystem for T {}

pub(crate) trait ChaseSystem {
	fn chase<TMovementMethod>(
		mut commands: Commands,
		mut persistent_entities: ResMut<PersistentEntities>,
		mut removed_chasers: RemovedComponents<Chase>,
		chasers: Query<(Entity, &Chase), With<Self>>,
		transforms: Query<&GlobalTransform>,
	) where
		Self: Component + Sized,
		TMovementMethod: ThreadSafe + Default,
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
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		components::persistent_entity::PersistentEntity,
		test_tools::utils::SingleThreadedApp,
		traits::register_persistent_entities::RegisterPersistentEntities,
	};

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq, Default)]
	struct _MovementMethod;

	fn setup(target_position: Vec3) -> (App, PersistentEntity) {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_systems(Update, _Agent::chase::<_MovementMethod>);

		let persistent_target = PersistentEntity::default();
		app.world_mut().spawn((
			GlobalTransform::from_translation(target_position),
			persistent_target,
		));

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
