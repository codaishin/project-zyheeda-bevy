use crate::components::{Chase, movement::Movement};
use bevy::prelude::*;
use common::traits::{
	thread_safe::ThreadSafe,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};

impl<T> ChaseSystem for T {}

pub(crate) trait ChaseSystem {
	fn chase<TMovementMethod>(
		mut commands: Commands,
		chasers: Query<(Entity, &Chase), With<Self>>,
		mut removed_chasers: RemovedComponents<Chase>,
		transforms: Query<&GlobalTransform>,
	) where
		Self: Component + Sized,
		TMovementMethod: ThreadSafe + Default,
	{
		for entity in removed_chasers.read() {
			commands.try_remove_from::<Movement<TMovementMethod>>(entity);
		}

		for (entity, Chase(target)) in &chasers {
			let Ok(target) = transforms.get(*target) else {
				continue;
			};
			commands.try_insert_on(
				entity,
				Movement {
					target: target.translation(),
					cstr: TMovementMethod::default,
				},
			);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq, Default)]
	struct _MovementMethod;

	fn setup(target_position: Vec3) -> (App, Entity) {
		let mut app = App::new();
		app.add_systems(Update, _Agent::chase::<_MovementMethod>);
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_translation(target_position))
			.id();

		(app, target)
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
			Some(&Movement {
				target: target_position,
				cstr: _MovementMethod::default
			}),
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
