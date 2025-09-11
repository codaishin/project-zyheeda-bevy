use super::Movement;
use crate::{components::movement::MotionTarget, traits::MovementUpdate};
use bevy::prelude::*;
use common::{
	components::immobilized::Immobilized,
	tools::{Done, speed::Speed},
	traits::{
		accessors::get::{RefAs, RefInto},
		handles_physics::LinearMotion,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use std::marker::PhantomData;

#[derive(PartialEq, Debug)]
pub struct Physical<TMotion>(PhantomData<TMotion>)
where
	TMotion: ThreadSafe;

impl<TMotion> MovementUpdate for Movement<Physical<TMotion>>
where
	TMotion: ThreadSafe
		+ From<LinearMotion>
		+ for<'a> RefInto<'a, Done>
		+ for<'a> RefInto<'a, LinearMotion>
		+ Component,
{
	type TComponents<'a> = Option<&'a TMotion>;
	type TConstraint = Without<Immobilized>;

	fn update(
		&self,
		agent: &mut ZyheedaEntityCommands,
		motion: Option<&TMotion>,
		speed: Speed,
	) -> Done {
		let new_motion = match self.target {
			Some(MotionTarget::Vec(target)) => LinearMotion::ToTarget { target, speed },
			Some(MotionTarget::Dir(direction)) => LinearMotion::Direction { direction, speed },
			None => LinearMotion::Stop,
		};

		match motion {
			Some(motion) if motion.ref_as::<LinearMotion>() == new_motion => {
				motion.ref_as::<Done>()
			}
			_ => {
				agent.try_insert(TMotion::from(new_motion));
				Done::from(false)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{
			component::Component,
			entity::Entity,
			system::{Query, ScheduleSystem},
		},
	};
	use common::{
		tools::UnitsPerSecond,
		traits::accessors::get::TryApplyOn,
		zyheeda_commands::ZyheedaCommands,
	};
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Result(Done);

	#[derive(Component)]
	struct _UpdateParams((Option<_Motion>, Speed));

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	enum _Motion {
		NotDone(LinearMotion),
		Done(LinearMotion),
	}

	impl From<&_Motion> for Done {
		fn from(motion: &_Motion) -> Self {
			Done::when(matches!(motion, _Motion::Done(..)))
		}
	}

	impl From<LinearMotion> for _Motion {
		fn from(linear: LinearMotion) -> Self {
			Self::NotDone(linear)
		}
	}

	impl From<&_Motion> for LinearMotion {
		fn from(motion: &_Motion) -> Self {
			match motion {
				_Motion::NotDone(linear_motion) => *linear_motion,
				_Motion::Done(linear_motion) => *linear_motion,
			}
		}
	}

	#[allow(clippy::type_complexity)]
	fn call_update(
		mut commands: ZyheedaCommands,
		agents: Query<
			(Entity, &Movement<Physical<_Motion>>, &_UpdateParams),
			<Movement<Physical<_Motion>> as MovementUpdate>::TConstraint,
		>,
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
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((None, speed)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion::from(LinearMotion::ToTarget { speed, target })),
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
				Movement::<Physical<_Motion>>::to(direction),
				_UpdateParams((None, speed)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion::from(LinearMotion::Direction { speed, direction })),
			app.world().entity(agent).get::<_Motion>()
		);
	}

	#[test]
	fn update_applies_stop_motion() {
		let mut app = setup(call_update);
		let agent = app
			.world_mut()
			.spawn((
				Movement::<Physical<_Motion>> {
					target: None,
					_m: PhantomData,
				},
				_UpdateParams((None, Speed(UnitsPerSecond::from(11.)))),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion::from(LinearMotion::Stop)),
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
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((
					Some(_Motion::NotDone(LinearMotion::ToTarget {
						speed: Speed(UnitsPerSecond::from(42.)),
						target: Vec3::new(1., 2., 3.),
					})),
					speed,
				)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion::from(LinearMotion::ToTarget { speed, target })),
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
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((
					Some(_Motion::Done(LinearMotion::ToTarget { speed, target })),
					speed,
				)),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<_Motion>());
	}

	#[test]
	fn movement_constraint_excludes_immobilized() {
		let mut app = setup(call_update);
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((None, speed)),
				Immobilized,
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
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((
					Some(_Motion::from(LinearMotion::ToTarget {
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
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((
					Some(_Motion::from(LinearMotion::Direction {
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
				Movement::<Physical<_Motion>>::to(target),
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
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((
					Some(_Motion::Done(LinearMotion::ToTarget { speed, target })),
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
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((
					Some(_Motion::Done(LinearMotion::ToTarget {
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
