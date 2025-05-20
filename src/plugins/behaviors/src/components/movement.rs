pub(crate) mod path_or_wasd;
pub(crate) mod velocity_based;

use super::SetFace;
use bevy::{ecs::query::QueryFilter, prelude::*};
use common::{
	test_tools::utils::ApproxEqual,
	traits::{
		handles_orientation::Face,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

#[derive(Component, Clone, PartialEq, Debug)]
#[require(GlobalTransform)]
pub(crate) struct Movement<TMovement> {
	pub(crate) target: Vec3,
	method_cstr: fn() -> TMovement,
}

impl<TMovement> Movement<TMovement> {
	#[cfg(test)]
	pub(crate) fn new(target: Vec3, method_cstr: fn() -> TMovement) -> Self {
		Self {
			target,
			method_cstr,
		}
	}

	pub(crate) fn to(target: Vec3) -> Self
	where
		TMovement: Default,
	{
		Self {
			target,
			method_cstr: TMovement::default,
		}
	}

	pub(crate) fn new_movement(&self) -> TMovement {
		(self.method_cstr)()
	}

	pub(crate) fn set_faces(
		mut commands: Commands,
		mut removed: RemovedComponents<Self>,
		changed: Query<(Entity, &Self), Changed<Self>>,
	) where
		TMovement: Sync + Send + 'static,
	{
		for entity in removed.read() {
			commands.try_remove_from::<SetFace>(entity);
		}

		for (entity, movement) in &changed {
			commands.try_insert_on(entity, SetFace(Face::Translation(movement.target)));
		}
	}

	pub(crate) fn cleanup(
		mut commands: Commands,
		mut removed: RemovedComponents<Self>,
		valid_entities: Query<(), <Movement<TMovement> as OnMovementRemoved>::TConstraint>,
	) where
		Movement<TMovement>: OnMovementRemoved + Sync + Send + 'static,
	{
		let matches_constraint = |entity: &Entity| valid_entities.contains(*entity);

		for entity in removed.read().filter(matches_constraint) {
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			Movement::<TMovement>::on_movement_removed(&mut entity);
		}
	}
}

impl<TMovement> Default for Movement<TMovement>
where
	TMovement: Default,
{
	fn default() -> Self {
		Self {
			target: Vec3::default(),
			method_cstr: TMovement::default,
		}
	}
}

impl<TMovement> ApproxEqual<f32> for Movement<TMovement> {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.target.approx_equal(&other.target, tolerance)
	}
}

pub(crate) trait OnMovementRemoved {
	type TConstraint: QueryFilter;

	fn on_movement_removed(entity: &mut EntityCommands);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::ScheduleSystem;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Default)]
	struct _T;

	fn setup<TMarker>(system: impl IntoScheduleConfigs<ScheduleSystem, TMarker>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, system);

		app
	}

	#[test]
	fn set_to_face_translation_on_update() {
		let mut app = setup(Movement::<_T>::set_faces);
		let entity = app
			.world_mut()
			.spawn(Movement::<_T>::to(Vec3::new(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			Some(&SetFace(Face::Translation(Vec3::new(1., 2., 3.)))),
			app.world().entity(entity).get::<SetFace>()
		);
	}

	#[test]
	fn do_not_set_to_face_translation_on_update_when_not_added() {
		let mut app = setup(Movement::<_T>::set_faces);
		let entity = app
			.world_mut()
			.spawn(Movement::<_T>::to(Vec3::new(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<SetFace>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SetFace>());
	}

	#[test]
	fn set_to_face_translation_on_update_when_changed() {
		let mut app = setup(Movement::<_T>::set_faces);
		let entity = app
			.world_mut()
			.spawn(Movement::<_T>::to(Vec3::new(1., 2., 3.)))
			.id();

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

	#[test]
	fn remove_set_face_on_update_when_removed() {
		let mut app = setup(Movement::<_T>::set_faces);
		let entity = app
			.world_mut()
			.spawn((Movement::<_T>::default(), SetFace(Face::Cursor)))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<Movement<_T>>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SetFace>());
	}

	#[test]
	fn when_movement_inserted_after_removal_in_same_frame_add_face() {
		let mut app = setup(Movement::<_T>::set_faces);
		let entity = app
			.world_mut()
			.spawn((Movement::<_T>::default(), SetFace(Face::Cursor)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<Movement<_T>>()
			.insert(Movement::<_T>::default());
		app.update();

		assert_eq!(
			Some(&SetFace(Face::Translation(Vec3::default()))),
			app.world().entity(entity).get::<SetFace>()
		);
	}

	impl OnMovementRemoved for Movement<_T> {
		type TConstraint = Without<_DoNotCallOnRemove>;

		fn on_movement_removed(entity: &mut EntityCommands) {
			entity.insert(_OnRemoveCalled);
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _OnRemoveCalled;

	#[derive(Component)]
	struct _DoNotCallOnRemove;

	#[test]
	fn cleanup_calls_on_remove() {
		let mut app = setup(Movement::<_T>::cleanup);
		let entity = app
			.world_mut()
			.spawn(Movement {
				target: default(),
				method_cstr: || _T,
			})
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<Movement<_T>>();
		app.update();

		assert_eq!(
			Some(&_OnRemoveCalled),
			app.world().entity(entity).get::<_OnRemoveCalled>()
		);
	}

	#[test]
	fn cleanup_does_not_call_on_remove_when_filter_not_satisfied() {
		let mut app = setup(Movement::<_T>::cleanup);
		let entity = app
			.world_mut()
			.spawn((Movement::<_T>::default(), _DoNotCallOnRemove))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<Movement<_T>>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_OnRemoveCalled>());
	}
}
