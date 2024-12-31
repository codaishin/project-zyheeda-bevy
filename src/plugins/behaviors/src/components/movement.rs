pub(crate) mod velocity_based;

use super::SetFace;
use crate::traits::Cleanup;
use bevy::prelude::*;
use common::{
	test_tools::utils::ApproxEqual,
	traits::{handles_orientation::Face, try_insert_on::TryInsertOn},
};
use std::marker::PhantomData;

#[derive(Component, Clone, PartialEq, Debug, Default)]
pub(crate) struct Movement<TMovement> {
	pub(crate) target: Vec3,
	phantom_data: PhantomData<TMovement>,
}

impl<TMovement> Movement<TMovement> {
	pub(crate) fn to(target: Vec3) -> Self {
		Self {
			target,
			phantom_data: PhantomData,
		}
	}

	pub(crate) fn update(mut commands: Commands, movements: Query<(Entity, &Self), Changed<Self>>)
	where
		TMovement: Sync + Send + 'static,
	{
		for (entity, movement) in &movements {
			commands.try_insert_on(entity, SetFace(Face::Translation(movement.target)));
		}
	}
}

impl<T> Cleanup for Movement<T> {
	fn cleanup(&self, agent: &mut EntityCommands) {
		agent.remove::<SetFace>();
	}
}

impl<TMovement> ApproxEqual<f32> for Movement<TMovement> {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.target.approx_equal(&other.target, tolerance)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::test_tools::utils::SingleThreadedApp;

	struct _T;

	fn call_cleanup(mut commands: Commands, query: Query<(Entity, &Movement<_T>)>) {
		for (id, movement) in &query {
			movement.cleanup(&mut commands.entity(id));
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn use_cleanup() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((Movement::<_T>::to(default()), SetFace(Face::Cursor)))
			.id();

		app.world_mut().run_system_once(call_cleanup)?;

		assert_eq!(None, app.world().entity(entity).get::<SetFace>());
		Ok(())
	}

	#[test]
	fn set_to_face_translation_on_update() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Movement::<_T>::to(Vec3::new(1., 2., 3.)))
			.id();

		app.world_mut().run_system_once(Movement::<_T>::update)?;

		assert_eq!(
			Some(&SetFace(Face::Translation(Vec3::new(1., 2., 3.)))),
			app.world().entity(entity).get::<SetFace>()
		);
		Ok(())
	}

	#[test]
	fn do_not_set_to_face_translation_on_update_when_not_added() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Movement::<_T>::to(Vec3::new(1., 2., 3.)))
			.id();

		app.add_systems(Update, Movement::<_T>::update);
		app.update();
		app.world_mut().entity_mut(entity).remove::<SetFace>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SetFace>());
	}

	#[test]
	fn set_to_face_translation_on_update_when_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Movement::<_T>::to(Vec3::new(1., 2., 3.)))
			.id();

		app.add_systems(Update, Movement::<_T>::update);
		app.update();
		let mut movement = app.world_mut().entity_mut(entity);
		let mut movement = movement.get_mut::<Movement<_T>>().unwrap();
		movement.target = Vec3::new(3., 4., 5.);
		app.update();

		assert_eq!(
			Some(&SetFace(Face::Translation(Vec3::new(3., 4., 5.)))),
			app.world().entity(entity).get::<SetFace>()
		);
	}
}
