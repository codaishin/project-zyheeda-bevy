pub(crate) mod path_or_wasd;

mod dto;

use super::SetFace;
use crate::{components::movement::dto::MovementDto, traits::MovementUpdate};
use bevy::prelude::*;
use common::{
	components::immobilized::Immobilized,
	tools::Done,
	traits::{
		accessors::get::{DynProperty, GetProperty, TryApplyOn},
		animation::GetMovementDirection,
		handles_movement_behavior::{MotionSpec, PathMotionSpec},
		handles_orientation::Face,
		handles_physics::LinearMotionSpec,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};
use macros::SavableComponent;
use std::marker::PhantomData;

#[derive(Component, SavableComponent, Debug, Copy)]
#[require(GlobalTransform)]
#[savable_component(dto = MovementDto<TMotion>)]
pub(crate) struct Movement<TMotion>
where
	TMotion: ThreadSafe,
{
	pub(crate) spec: PathMotionSpec,
	_m: PhantomData<TMotion>,
}

impl<TMotion> Movement<TMotion>
where
	TMotion: ThreadSafe,
{
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
			let set_face = match &movement.spec {
				PathMotionSpec(MotionSpec::ToTarget { target, .. }) => {
					SetFace(Face::Translation(*target))
				}
				PathMotionSpec(MotionSpec::Direction { direction, .. }) => {
					SetFace(Face::Direction(*direction))
				}
				_ => continue,
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(set_face);
			});
		}
	}

	pub(crate) fn stop() -> Self {
		Self {
			spec: PathMotionSpec(MotionSpec::Stop),
			_m: PhantomData,
		}
	}
}

impl<TMotion> From<PathMotionSpec> for Movement<TMotion>
where
	TMotion: ThreadSafe,
{
	fn from(target: PathMotionSpec) -> Self {
		Self {
			spec: target,
			_m: PhantomData,
		}
	}
}

impl<TMotion> PartialEq for Movement<TMotion>
where
	TMotion: ThreadSafe,
{
	fn eq(&self, other: &Self) -> bool {
		self.spec == other.spec
	}
}

impl<TMotion> Clone for Movement<TMotion>
where
	TMotion: ThreadSafe,
{
	fn clone(&self) -> Self {
		Self {
			spec: self.spec,
			_m: PhantomData,
		}
	}
}

impl<TMotion> GetProperty<PathMotionSpec> for Movement<TMotion>
where
	TMotion: ThreadSafe,
{
	fn get_property(&self) -> MotionSpec {
		self.spec.0
	}
}

impl<TMotion> MovementUpdate for Movement<TMotion>
where
	TMotion: ThreadSafe
		+ From<LinearMotionSpec>
		+ GetProperty<Done>
		+ GetProperty<LinearMotionSpec>
		+ Component,
{
	type TComponents<'a> = Option<&'a TMotion>;
	type TConstraint = Without<Immobilized>;

	fn update(&self, agent: &mut ZyheedaEntityCommands, motion: Option<&TMotion>) -> Done {
		let PathMotionSpec(target) = self.spec;

		match motion {
			Some(motion) if motion.dyn_property::<LinearMotionSpec>() == target => {
				Done::when(motion.dyn_property::<Done>())
			}
			_ => {
				agent.try_insert(TMotion::from(LinearMotionSpec(target)));
				Done(false)
			}
		}
	}
}

impl<TMotion> GetMovementDirection for Movement<TMotion>
where
	TMotion: ThreadSafe,
{
	fn movement_direction(&self, transform: &GlobalTransform) -> Option<Dir3> {
		match self.spec {
			PathMotionSpec(MotionSpec::ToTarget { target, .. }) => {
				Dir3::try_from(target - transform.translation()).ok()
			}
			PathMotionSpec(MotionSpec::Direction { direction, .. }) => Some(direction),
			PathMotionSpec(MotionSpec::Stop) => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::ScheduleSystem;
	use common::tools::speed::Speed;
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	enum _Motion {
		NotDone(LinearMotionSpec),
		Done(LinearMotionSpec),
	}

	impl From<LinearMotionSpec> for _Motion {
		fn from(linear: LinearMotionSpec) -> Self {
			Self::NotDone(linear)
		}
	}

	impl GetProperty<Done> for _Motion {
		fn get_property(&self) -> bool {
			matches!(self, _Motion::Done(..))
		}
	}

	impl GetProperty<LinearMotionSpec> for _Motion {
		fn get_property(&self) -> MotionSpec {
			match self {
				_Motion::NotDone(LinearMotionSpec(target)) => *target,
				_Motion::Done(LinearMotionSpec(target)) => *target,
			}
		}
	}

	mod set_face {
		use super::*;

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
				.spawn(Movement::<_Motion>::from(PathMotionSpec(
					MotionSpec::ToTarget {
						target: Vec3::new(1., 2., 3.),
						speed: Speed::ZERO,
					},
				)))
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
				.spawn(Movement::<_Motion>::from(PathMotionSpec(
					MotionSpec::ToTarget {
						target: Vec3::new(1., 2., 3.),
						speed: Speed::ZERO,
					},
				)))
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
				.spawn(Movement::<_Motion>::from(PathMotionSpec(
					MotionSpec::ToTarget {
						target: Vec3::new(1., 2., 3.),
						speed: Speed::ZERO,
					},
				)))
				.id();

			app.update();
			let mut movement = app.world_mut().entity_mut(entity);
			let mut movement = movement.get_mut::<Movement<_Motion>>().unwrap();
			movement.spec = PathMotionSpec(MotionSpec::ToTarget {
				speed: Speed::ZERO,
				target: Vec3::new(3., 4., 5.),
			});
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
				.spawn(Movement::<_Motion>::from(PathMotionSpec(
					MotionSpec::Direction {
						direction: Dir3::NEG_X,
						speed: Speed::ZERO,
					},
				)))
				.id();

			app.update();
			let mut movement = app.world_mut().entity_mut(entity);
			let mut movement = movement.get_mut::<Movement<_Motion>>().unwrap();
			movement.spec = PathMotionSpec(MotionSpec::Direction {
				direction: Dir3::NEG_Z,
				speed: Speed::ZERO,
			});
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
				.spawn((
					Movement::<_Motion>::from(PathMotionSpec(MotionSpec::Direction {
						direction: Dir3::NEG_X,
						speed: Speed::ZERO,
					})),
					SetFace(Face::Target),
				))
				.id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.remove::<Movement<_Motion>>();
			app.update();

			assert_eq!(None, app.world().entity(entity).get::<SetFace>());
		}

		#[test]
		fn when_movement_inserted_after_removal_in_same_frame_add_face() {
			let mut app = setup(Movement::<_Motion>::set_faces);
			let entity = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::from(PathMotionSpec(MotionSpec::Direction {
						direction: Dir3::NEG_X,
						speed: Speed::ZERO,
					})),
					SetFace(Face::Target),
				))
				.id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.remove::<Movement<_Motion>>()
				.insert(Movement::<_Motion>::from(PathMotionSpec(
					MotionSpec::Direction {
						direction: Dir3::NEG_X,
						speed: Speed::ZERO,
					},
				)));
			app.update();

			assert_eq!(
				Some(&SetFace(Face::Direction(Dir3::NEG_X))),
				app.world().entity(entity).get::<SetFace>()
			);
		}
	}

	mod get_direction {
		use super::*;

		#[test]
		fn get_movement_from_translation() {
			let target = Vec3::new(1., 2., 3.);
			let position = Vec3::new(4., 7., -1.);
			let movement = Movement::<_Motion>::from(PathMotionSpec(MotionSpec::ToTarget {
				target,
				speed: Speed::ZERO,
			}));

			let direction =
				movement.movement_direction(&GlobalTransform::from_translation(position));

			assert_eq!(Some(Dir3::try_from(target - position).unwrap()), direction);
		}

		#[test]
		fn get_no_movement_direction_when_target_is_position() {
			let target = Vec3::new(1., 2., 3.);
			let position = target;
			let movement = Movement::<_Motion>::from(PathMotionSpec(MotionSpec::ToTarget {
				target,
				speed: Speed::ZERO,
			}));

			let direction =
				movement.movement_direction(&GlobalTransform::from_translation(position));

			assert_eq!(None, direction);
		}

		#[test]
		fn get_movement_from_direction() {
			let direction = Dir3::NEG_Z;
			let movement = Movement::<_Motion>::from(PathMotionSpec(MotionSpec::Direction {
				direction,
				speed: Speed::ZERO,
			}));

			let direction = movement
				.movement_direction(&GlobalTransform::from_translation(Vec3::new(4., 7., -1.)));

			assert_eq!(Some(Dir3::NEG_Z), direction);
		}
	}

	mod movement_update {
		use super::*;
		use common::tools::UnitsPerSecond;

		#[derive(Component, Debug, PartialEq)]
		struct _Result(Done);

		#[derive(Component)]
		struct _UpdateParams(Option<_Motion>);

		#[allow(clippy::type_complexity)]
		fn call_update(
			mut commands: ZyheedaCommands,
			agents: Query<
				(Entity, &Movement<_Motion>, &_UpdateParams),
				<Movement<_Motion> as MovementUpdate>::TConstraint,
			>,
		) {
			for (entity, movement, params) in &agents {
				commands.try_apply_on(&entity, |mut e| {
					let _UpdateParams(motion) = *params;
					let result = movement.update(&mut e, motion.as_ref());
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
		fn update_applies_motion_target() {
			let mut app = setup(call_update);
			let target = MotionSpec::ToTarget {
				speed: Speed(UnitsPerSecond::from(11.)),
				target: Vec3::new(10., 0., 7.),
			};
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::from(PathMotionSpec(target)),
					_UpdateParams(None),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(LinearMotionSpec(target))),
				app.world().entity(agent).get::<_Motion>()
			);
		}

		#[test]
		fn update_applies_motion_when_different_motion_present() {
			let mut app = setup(call_update);
			let target = MotionSpec::ToTarget {
				speed: Speed(UnitsPerSecond::from(11.)),
				target: Vec3::new(10., 0., 7.),
			};
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::from(PathMotionSpec(target)),
					_UpdateParams(Some(_Motion::NotDone(LinearMotionSpec(
						MotionSpec::ToTarget {
							speed: Speed(UnitsPerSecond::from(42.)),
							target: Vec3::new(1., 2., 3.),
						},
					)))),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(LinearMotionSpec(target))),
				app.world().entity(agent).get::<_Motion>()
			);
		}

		#[test]
		fn update_applies_no_motion_when_same_motion_present() {
			let mut app = setup(call_update);
			let target = MotionSpec::ToTarget {
				speed: Speed(UnitsPerSecond::from(11.)),
				target: Vec3::new(10., 0., 7.),
			};
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::from(PathMotionSpec(target)),
					_UpdateParams(Some(_Motion::NotDone(LinearMotionSpec(target)))),
				))
				.id();

			app.update();

			assert_eq!(None, app.world().entity(agent).get::<_Motion>());
		}

		#[test]
		fn movement_constraint_excludes_immobilized() {
			let mut app = setup(call_update);
			let target = MotionSpec::ToTarget {
				speed: Speed(UnitsPerSecond::from(11.)),
				target: Vec3::new(10., 0., 7.),
			};
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::from(PathMotionSpec(target)),
					_UpdateParams(None),
					Immobilized,
				))
				.id();

			app.update();

			assert_eq!(None, app.world().entity(agent).get::<_Motion>());
		}

		#[test]
		fn update_returns_not_done_when_target_motion_present() {
			let mut app = setup(call_update);
			let target = MotionSpec::ToTarget {
				speed: Speed(UnitsPerSecond::from(11.)),
				target: Vec3::new(10., 0., 7.),
			};
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::from(PathMotionSpec(target)),
					_UpdateParams(Some(_Motion::from(LinearMotionSpec(
						MotionSpec::ToTarget {
							speed: Speed::default(),
							target: Vec3::default(),
						},
					)))),
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
			let target = MotionSpec::ToTarget {
				speed: Speed(UnitsPerSecond::from(11.)),
				target: Vec3::new(10., 0., 7.),
			};
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::from(PathMotionSpec(target)),
					_UpdateParams(None),
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
			let target = MotionSpec::ToTarget {
				speed: Speed(UnitsPerSecond::from(11.)),
				target: Vec3::new(10., 0., 7.),
			};
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::from(PathMotionSpec(target)),
					_UpdateParams(Some(_Motion::Done(LinearMotionSpec(target)))),
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
			let target = MotionSpec::ToTarget {
				speed: Speed(UnitsPerSecond::from(11.)),
				target: Vec3::new(10., 0., 7.),
			};
			let agent = app
				.world_mut()
				.spawn((
					Movement::<_Motion>::from(PathMotionSpec(target)),
					_UpdateParams(Some(_Motion::Done(LinearMotionSpec(
						MotionSpec::ToTarget {
							speed: Speed(UnitsPerSecond::from(42.)),
							target: Vec3::new(11., 1., 8.),
						},
					)))),
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
