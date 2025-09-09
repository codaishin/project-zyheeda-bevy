pub(crate) mod path_or_wasd;
pub(crate) mod physical;

mod dto;

use std::marker::PhantomData;

use super::SetFace;
use crate::{
	components::movement::dto::MovementDto,
	systems::movement::insert_process_component::StopMovement,
};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, handles_orientation::Face, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaCommands,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug)]
#[require(GlobalTransform)]
#[savable_component(dto = MovementDto<TMovement>)]
pub(crate) struct Movement<TMovement>
where
	TMovement: ThreadSafe,
{
	pub(crate) target: Option<MotionTarget>,
	_m: PhantomData<TMovement>,
}

impl<TMovement> Movement<TMovement>
where
	TMovement: ThreadSafe,
{
	#[cfg(test)]
	pub(crate) fn to_none() -> Self {
		Self {
			target: None,
			_m: PhantomData,
		}
	}

	pub(crate) fn to<T>(target: T) -> Self
	where
		T: Into<MotionTarget>,
	{
		Self {
			target: Some(target.into()),
			_m: PhantomData,
		}
	}

	pub(crate) fn set_faces(
		mut commands: ZyheedaCommands,
		mut removed: RemovedComponents<Self>,
		changed: Query<(Entity, &Self), Changed<Self>>,
	) {
		for entity in removed.read() {
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<SetFace>();
			});
		}

		for (entity, movement) in &changed {
			let set_face = match &movement.target {
				Some(MotionTarget::Vec(vec3)) => SetFace(Face::Translation(*vec3)),
				Some(MotionTarget::Dir(dir3)) => SetFace(Face::Direction(*dir3)),
				_ => continue,
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(set_face);
			});
		}
	}
}

impl<TMovement> StopMovement for Movement<TMovement>
where
	TMovement: ThreadSafe,
{
	fn stop() -> Self {
		Self {
			target: None,
			_m: PhantomData,
		}
	}
}

impl<TMovement> PartialEq for Movement<TMovement>
where
	TMovement: ThreadSafe,
{
	fn eq(&self, other: &Self) -> bool {
		self.target == other.target
	}
}

impl<TMovement> Clone for Movement<TMovement>
where
	TMovement: ThreadSafe,
{
	fn clone(&self) -> Self {
		Self {
			target: self.target,
			_m: PhantomData,
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum MotionTarget {
	Vec(Vec3),
	Dir(Dir3),
}

impl From<Vec3> for MotionTarget {
	fn from(value: Vec3) -> Self {
		Self::Vec(value)
	}
}

impl From<Dir3> for MotionTarget {
	fn from(value: Dir3) -> Self {
		Self::Dir(value)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::ScheduleSystem;
	use testing::{ApproxEqual, SingleThreadedApp};

	impl ApproxEqual<f32> for MotionTarget {
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			match (self, other) {
				(MotionTarget::Vec(a), MotionTarget::Vec(b)) => a.approx_equal(b, tolerance),
				(MotionTarget::Dir(a), MotionTarget::Dir(b)) => a.approx_equal(b, tolerance),
				_ => false,
			}
		}
	}

	impl<TMovement> ApproxEqual<f32> for Movement<TMovement>
	where
		TMovement: ThreadSafe,
	{
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			self.target.approx_equal(&other.target, tolerance)
		}
	}

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
		movement.target = Some(Vec3::new(3., 4., 5.).into());
		app.update();

		assert_eq!(
			Some(&SetFace(Face::Translation(Vec3::new(3., 4., 5.)))),
			app.world().entity(entity).get::<SetFace>()
		);
	}

	#[test]
	fn set_to_face_direction_on_update_when_changed() {
		let mut app = setup(Movement::<_T>::set_faces);
		let entity = app.world_mut().spawn(Movement::<_T>::to(Dir3::NEG_X)).id();

		app.update();
		let mut movement = app.world_mut().entity_mut(entity);
		let mut movement = movement.get_mut::<Movement<_T>>().unwrap();
		movement.target = Some(Dir3::NEG_Z.into());
		app.update();

		assert_eq!(
			Some(&SetFace(Face::Direction(Dir3::NEG_Z))),
			app.world().entity(entity).get::<SetFace>()
		);
	}

	#[test]
	fn remove_set_face_on_update_when_removed() {
		let mut app = setup(Movement::<_T>::set_faces);
		let entity = app
			.world_mut()
			.spawn((Movement::<_T>::to(Dir3::NEG_X), SetFace(Face::Target)))
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
			.spawn((Movement::<_T>::to(Dir3::NEG_X), SetFace(Face::Target)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<Movement<_T>>()
			.insert(Movement::<_T>::to(Dir3::NEG_X));
		app.update();

		assert_eq!(
			Some(&SetFace(Face::Direction(Dir3::NEG_X))),
			app.world().entity(entity).get::<SetFace>()
		);
	}
}
