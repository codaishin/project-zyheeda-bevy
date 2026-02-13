pub(crate) mod path_or_direction;

mod dto;

use crate::{
	components::{facing::SetFace, movement::dto::MovementDto},
	traits::movement_update::MovementUpdate,
};
use bevy::prelude::*;
use common::{
	tools::{Done, speed::Speed},
	traits::{
		accessors::get::{DynProperty, GetProperty, TryApplyOn},
		handles_movement::MovementTarget,
		handles_orientation::Face,
		handles_physics::CharacterMotion,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};
use macros::SavableComponent;
use std::marker::PhantomData;

#[derive(Component, SavableComponent, Debug)]
#[require(GlobalTransform)]
#[savable_component(dto = MovementDto)]
pub struct Movement<TMotion> {
	pub(crate) target: Option<MovementTarget>,
	_m: PhantomData<fn() -> TMotion>,
}

impl<TMotion> Movement<TMotion> {
	pub(crate) fn stop() -> Self {
		Self {
			target: None,
			_m: PhantomData,
		}
	}

	pub(crate) fn to<T>(target: T) -> Self
	where
		T: Into<MovementTarget>,
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
	) where
		Self: Component,
	{
		for entity in removed.read() {
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<SetFace>();
			});
		}

		for (entity, movement) in &changed {
			commands.try_apply_on(&entity, |mut e| {
				match &movement.target {
					Some(MovementTarget::Point(vec3)) => {
						e.try_insert(SetFace(Face::Translation(*vec3)));
					}
					Some(MovementTarget::Dir(dir3)) => {
						e.try_insert(SetFace(Face::Direction(*dir3)));
					}
					None => {
						e.try_remove::<SetFace>();
					}
				};
			});
		}
	}
}

impl<TMotion> PartialEq for Movement<TMotion> {
	fn eq(&self, other: &Self) -> bool {
		self.target == other.target
	}
}

impl<TMotion> Clone for Movement<TMotion> {
	fn clone(&self) -> Self {
		Self {
			target: self.target,
			_m: PhantomData,
		}
	}
}

impl<TMotion> GetProperty<Option<MovementTarget>> for Movement<TMotion> {
	fn get_property(&self) -> Option<MovementTarget> {
		self.target
	}
}

impl<TMotion> MovementUpdate for Movement<TMotion>
where
	TMotion: From<CharacterMotion> + GetProperty<Done> + GetProperty<CharacterMotion> + Component,
{
	type TComponents<'a> = Option<&'a TMotion>;
	type TConstraint = ();

	fn update(
		&self,
		agent: &mut ZyheedaEntityCommands,
		motion: Option<&TMotion>,
		speed: Speed,
	) -> Done {
		let new_motion = match self.target {
			Some(MovementTarget::Point(target)) => CharacterMotion::ToTarget { target, speed },
			Some(MovementTarget::Dir(direction)) => CharacterMotion::Direction { direction, speed },
			None => CharacterMotion::Stop,
		};

		match motion {
			Some(motion) if motion.dyn_property::<CharacterMotion>() == new_motion => {
				Done::when(motion.dyn_property::<Done>())
			}
			_ => {
				agent.try_insert(TMotion::from(new_motion));
				Done(false)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::ecs::system::ScheduleSystem;
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	enum _Motion {
		NotDone(CharacterMotion),
		Done(CharacterMotion),
	}

	impl From<CharacterMotion> for _Motion {
		fn from(linear: CharacterMotion) -> Self {
			Self::NotDone(linear)
		}
	}

	impl GetProperty<Done> for _Motion {
		fn get_property(&self) -> bool {
			matches!(self, _Motion::Done(..))
		}
	}

	impl GetProperty<CharacterMotion> for _Motion {
		fn get_property(&self) -> CharacterMotion {
			match self {
				_Motion::NotDone(motion) => *motion,
				_Motion::Done(motion) => *motion,
			}
		}
	}

	mod set_face {
		use super::*;
		use testing::ApproxEqual;

		impl<TMotion> ApproxEqual<f32> for Movement<TMotion> {
			fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
				let (a, b) = match (self.target, other.target) {
					(Some(a), Some(b)) => (a, b),
					(None, None) => return true,
					_ => return false,
				};

				match (a, b) {
					(MovementTarget::Point(a), MovementTarget::Point(b)) => {
						a.approx_equal(&b, tolerance)
					}
					(MovementTarget::Dir(a), MovementTarget::Dir(b)) => {
						a.approx_equal(&b, tolerance)
					}
					_ => false,
				}
			}
		}

		fn setup<TMarker>(system: impl IntoScheduleConfigs<ScheduleSystem, TMarker>) -> App {
			let mut app = App::new().single_threaded(Update);

			app.add_systems(Update, system);

			app
		}

		#[test]
		fn set_to_face_translation_on_update() {
			let mut app = setup(Movement::<_Motion>::set_faces);
			let entity = app
				.world_mut()
				.spawn(Movement::<_Motion>::to(Vec3::new(1., 2., 3.)))
				.id();

			app.update();

			assert_eq!(
				Some(&SetFace(Face::Translation(Vec3::new(1., 2., 3.)))),
				app.world().entity(entity).get::<SetFace>()
			);
		}

		#[test]
		fn do_not_set_to_face_translation_on_update_when_not_added() {
			let mut app = setup(Movement::<_Motion>::set_faces);
			let entity = app
				.world_mut()
				.spawn(Movement::<_Motion>::to(Vec3::new(1., 2., 3.)))
				.id();

			app.update();
			app.world_mut().entity_mut(entity).remove::<SetFace>();
			app.update();

			assert_eq!(None, app.world().entity(entity).get::<SetFace>());
		}

		#[test]
		fn set_to_face_translation_on_update_when_changed() {
			let mut app = setup(Movement::<_Motion>::set_faces);
			let entity = app
				.world_mut()
				.spawn(Movement::<_Motion>::to(Vec3::new(1., 2., 3.)))
				.id();

			app.update();
			let mut movement = app.world_mut().entity_mut(entity);
			let mut movement = movement.get_mut::<Movement<_Motion>>().unwrap();
			movement.target = Some(Vec3::new(3., 4., 5.).into());
			app.update();

			assert_eq!(
				Some(&SetFace(Face::Translation(Vec3::new(3., 4., 5.)))),
				app.world().entity(entity).get::<SetFace>()
			);
		}

		#[test]
		fn set_to_face_direction_on_update_when_changed() {
			let mut app = setup(Movement::<_Motion>::set_faces);
			let entity = app
				.world_mut()
				.spawn(Movement::<_Motion>::to(Dir3::NEG_X))
				.id();

			app.update();
			let mut movement = app.world_mut().entity_mut(entity);
			let mut movement = movement.get_mut::<Movement<_Motion>>().unwrap();
			movement.target = Some(Dir3::NEG_Z.into());
			app.update();

			assert_eq!(
				Some(&SetFace(Face::Direction(Dir3::NEG_Z))),
				app.world().entity(entity).get::<SetFace>()
			);
		}

		#[test]
		fn remove_set_face_on_update_when_removed() {
			let mut app = setup(Movement::<_Motion>::set_faces);
			let entity = app
				.world_mut()
				.spawn((Movement::<_Motion>::to(Dir3::NEG_X), SetFace(Face::Target)))
				.id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.remove::<Movement<_Motion>>();
			app.update();

			assert_eq!(None, app.world().entity(entity).get::<SetFace>());
		}

		#[test]
		fn remove_set_face_on_update_when_set_to_stop() {
			let mut app = setup(Movement::<_Motion>::set_faces);
			let entity = app.world_mut().spawn(SetFace(Face::Target)).id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.insert(Movement::<_Motion>::stop());
			app.update();

			assert_eq!(None, app.world().entity(entity).get::<SetFace>());
		}

		#[test]
		fn when_movement_inserted_after_removal_in_same_frame_add_face() {
			let mut app = setup(Movement::<_Motion>::set_faces);
			let entity = app
				.world_mut()
				.spawn((Movement::<_Motion>::to(Dir3::NEG_X), SetFace(Face::Target)))
				.id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.remove::<Movement<_Motion>>()
				.insert(Movement::<_Motion>::to(Dir3::NEG_X));
			app.update();

			assert_eq!(
				Some(&SetFace(Face::Direction(Dir3::NEG_X))),
				app.world().entity(entity).get::<SetFace>()
			);
		}
	}

	mod movement_update {
		use super::*;
		use common::tools::UnitsPerSecond;

		#[derive(Component, Debug, PartialEq)]
		struct _Result(Done);

		#[derive(Component)]
		struct _UpdateParams((Option<_Motion>, Speed));

		#[allow(clippy::type_complexity)]
		fn call_update(
			mut commands: ZyheedaCommands,
			agents: Query<(Entity, &Movement<_Motion>, &_UpdateParams)>,
		) {
			for (entity, movement, params) in &agents {
				commands.try_apply_on(&entity, |mut e| {
					let _UpdateParams((motion, speed)) = *params;
					let result = movement.update(&mut e, motion.as_ref(), speed);
					e.try_insert(_Result(result));
				});
			}
		}

		fn setup<TMarker>(system: impl IntoScheduleConfigs<ScheduleSystem, TMarker>) -> App {
			let mut app = App::new().single_threaded(Update);

			app.add_systems(Update, system.chain());

			app
		}

		#[test]
		fn update_applies_target_motion() {
			let mut app = setup(call_update);
			let target = Vec3::new(10., 0., 7.);
			let speed = Speed(UnitsPerSecond::from(11.));
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::to(target),
					_UpdateParams((None, speed)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(CharacterMotion::ToTarget { speed, target })),
				app.world().entity(agent).get::<_Motion>()
			);
		}
		#[test]
		fn update_applies_directional_motion() {
			let mut app = setup(call_update);
			let direction = Dir3::NEG_X;
			let speed = Speed(UnitsPerSecond::from(11.));
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::to(direction),
					_UpdateParams((None, speed)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(CharacterMotion::Direction {
					speed,
					direction
				})),
				app.world().entity(agent).get::<_Motion>()
			);
		}

		#[test]
		fn update_applies_stop_motion() {
			let mut app = setup(call_update);
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion> {
						target: None,
						_m: PhantomData,
					},
					_UpdateParams((None, Speed(UnitsPerSecond::from(11.)))),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(CharacterMotion::Stop)),
				app.world().entity(agent).get::<_Motion>()
			);
		}

		#[test]
		fn update_applies_motion_when_different_motion_present() {
			let mut app = setup(call_update);
			let target = Vec3::new(10., 0., 7.);
			let speed = Speed(UnitsPerSecond::from(11.));
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::to(target),
					_UpdateParams((
						Some(_Motion::NotDone(CharacterMotion::ToTarget {
							speed: Speed(UnitsPerSecond::from(42.)),
							target: Vec3::new(1., 2., 3.),
						})),
						speed,
					)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(CharacterMotion::ToTarget { speed, target })),
				app.world().entity(agent).get::<_Motion>()
			);
		}

		#[test]
		fn update_applies_no_motion_when_same_motion_present() {
			let mut app = setup(call_update);
			let target = Vec3::new(10., 0., 7.);
			let speed = Speed(UnitsPerSecond::from(11.));
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::to(target),
					_UpdateParams((
						Some(_Motion::Done(CharacterMotion::ToTarget { speed, target })),
						speed,
					)),
				))
				.id();

			app.update();

			assert_eq!(None, app.world().entity(agent).get::<_Motion>());
		}

		#[test]
		fn update_returns_not_done_when_target_motion_present() {
			let mut app = setup(call_update);
			let target = Vec3::new(10., 0., 7.);
			let speed = Speed(UnitsPerSecond::from(11.));
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::to(target),
					_UpdateParams((
						Some(_Motion::from(CharacterMotion::ToTarget {
							speed: Speed::default(),
							target: Vec3::default(),
						})),
						speed,
					)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Result(Done::from(false))),
				app.world().entity(agent).get::<_Result>()
			);
		}

		#[test]
		fn update_returns_not_done_when_directional_motion_present() {
			let mut app = setup(call_update);
			let target = Vec3::new(10., 0., 7.);
			let speed = Speed(UnitsPerSecond::from(11.));
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::to(target),
					_UpdateParams((
						Some(_Motion::from(CharacterMotion::Direction {
							speed: Speed::default(),
							direction: Dir3::NEG_X,
						})),
						speed,
					)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Result(Done::from(false))),
				app.world().entity(agent).get::<_Result>()
			);
		}

		#[test]
		fn update_returns_not_done_when_no_motion_present() {
			let mut app = setup(call_update);
			let target = Vec3::new(10., 0., 7.);
			let speed = Speed(UnitsPerSecond::from(11.));
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::to(target),
					_UpdateParams((None, speed)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Result(Done::from(false))),
				app.world().entity(agent).get::<_Result>()
			);
		}

		#[test]
		fn update_returns_done_when_motion_done() {
			let mut app = setup(call_update);
			let target = Vec3::new(10., 0., 7.);
			let speed = Speed(UnitsPerSecond::from(11.));
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::to(target),
					_UpdateParams((
						Some(_Motion::Done(CharacterMotion::ToTarget { speed, target })),
						speed,
					)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Result(Done::from(true))),
				app.world().entity(agent).get::<_Result>()
			);
		}

		#[test]
		fn update_returns_not_done_when_inserting_new_motion_done() {
			let mut app = setup(call_update);
			let target = Vec3::new(10., 0., 7.);
			let speed = Speed(UnitsPerSecond::from(11.));
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::to(target),
					_UpdateParams((
						Some(_Motion::Done(CharacterMotion::ToTarget {
							speed: Speed(UnitsPerSecond::from(42.)),
							target: Vec3::new(11., 1., 8.),
						})),
						speed,
					)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Result(Done::from(false))),
				app.world().entity(agent).get::<_Result>()
			);
		}
	}
}
