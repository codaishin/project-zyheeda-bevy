use crate::{
	components::{Chase, movement::Movement},
	systems::movement::insert_process_component::StopMovement,
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{GetMut, TryApplyOn},
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> ChaseSystem for T where T: Component {}

pub(crate) trait ChaseSystem: Component + Sized {
	fn chase<TMotion>(
		mut commands: ZyheedaCommands,
		mut removed_chasers: RemovedComponents<Chase>,
		chasers: Query<(Entity, &Chase), With<Self>>,
		transforms: Query<&GlobalTransform>,
	) where
		TMotion: ThreadSafe,
	{
		for entity in removed_chasers.read() {
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(Movement::<TMotion>::stop());
			});
		}

		for (entity, Chase(target)) in &chasers {
			let Some(target) = commands.get_mut(target) else {
				continue;
			};
			let Ok(target) = transforms.get(target.id()) else {
				continue;
			};
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(Movement::<TMotion>::to(target.translation()));
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::SingleThreadedApp;

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
	fn stop_movement_when_stopped_chasing() {
		let (mut app, target) = setup(Vec3::new(1., 2., 3.));
		let chaser = app
			.world_mut()
			.spawn((_Agent, GlobalTransform::from_xyz(3., 2., 1.), Chase(target)))
			.id();

		app.update();
		app.world_mut().entity_mut(chaser).remove::<Chase>();
		app.update();

		assert_eq!(
			Some(&Movement::stop()),
			app.world()
				.entity(chaser)
				.get::<Movement<_MovementMethod>>()
		);
	}
}
