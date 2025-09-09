use super::{Movement, OnMovementRemoved};
use crate::traits::{IsDone, MovementUpdate, change_per_frame::MinDistance};
use bevy::prelude::*;
use common::{
	components::immobilized::Immobilized,
	tools::speed::Speed,
	traits::{
		animation::GetMovementDirection,
		handles_physics::LinearMotion,
		thread_safe::ThreadSafe,
	},
};
use std::{cmp::Ordering, marker::PhantomData, time::Duration};

#[derive(PartialEq, Debug)]
pub struct Physical<TMotion>(PhantomData<TMotion>)
where
	TMotion: ThreadSafe;

impl<TMotion> Default for Physical<TMotion>
where
	TMotion: ThreadSafe,
{
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<TMotion> MinDistance for Physical<TMotion>
where
	TMotion: ThreadSafe,
{
	fn min_distance(speed: Speed, delta: Duration) -> f32 {
		delta.as_secs_f32() * *speed
	}
}

impl<TMotion> MovementUpdate for Movement<Physical<TMotion>>
where
	TMotion: ThreadSafe + From<LinearMotion> + Component,
{
	type TComponents<'a> = &'a GlobalTransform;
	type TConstraint = Without<Immobilized>;

	fn update(
		&self,
		agent: &mut EntityCommands,
		transform: &GlobalTransform,
		speed: Speed,
		delta: Duration,
	) -> IsDone {
		let direction = self.target - transform.translation();
		let min_distance = Physical::<TMotion>::min_distance(speed, delta);

		match direction.length().partial_cmp(&min_distance) {
			Some(Ordering::Less | Ordering::Equal) | None => {
				agent.try_insert(TMotion::from(LinearMotion::ZERO));
				IsDone(true)
			}
			_ => {
				agent.try_insert(TMotion::from(LinearMotion(direction.normalize() * *speed)));
				IsDone(false)
			}
		}
	}
}

impl<TMotion> OnMovementRemoved for Movement<Physical<TMotion>>
where
	TMotion: Component + From<LinearMotion>,
{
	type TConstraint = Without<Immobilized>;

	fn on_movement_removed(entity: &mut EntityCommands) {
		entity.try_insert(TMotion::from(LinearMotion::ZERO));
	}
}

impl<TMotion> GetMovementDirection for Movement<Physical<TMotion>>
where
	TMotion: ThreadSafe,
{
	fn movement_direction(&self, transform: &GlobalTransform) -> Option<Dir3> {
		Dir3::try_from(self.target - transform.translation()).ok()
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
			system::{Commands, Query, ScheduleSystem},
		},
	};
	use common::tools::UnitsPerSecond;
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Result(IsDone);

	#[derive(Component)]
	struct _UpdateParams((GlobalTransform, Speed));

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Motion(LinearMotion);

	impl From<LinearMotion> for _Motion {
		fn from(linear: LinearMotion) -> Self {
			Self(linear)
		}
	}

	#[allow(clippy::type_complexity)]
	fn call_update(
		delta: Duration,
	) -> impl Fn(
		Commands,
		Query<
			(Entity, &Movement<Physical<_Motion>>, &_UpdateParams),
			<Movement<Physical<_Motion>> as MovementUpdate>::TConstraint,
		>,
	) {
		move |mut commands, agents| {
			for (entity, movement, params) in &agents {
				let entity = &mut commands.entity(entity);
				let _UpdateParams((position, speed)) = *params;
				let result = movement.update(entity, &position, speed, delta);
				entity.insert(_Result(result));
			}
		}
	}

	struct _OnRemoveCalled;

	fn call_on_remove(
		mut commands: Commands,
		entities: Query<Entity, <Movement<Physical<_Motion>> as OnMovementRemoved>::TConstraint>,
	) {
		for entity in &entities {
			let entity = &mut commands.entity(entity);
			Movement::<Physical<_Motion>>::on_movement_removed(entity);
		}
	}

	fn setup<TMarker>(system: impl IntoScheduleConfigs<ScheduleSystem, TMarker>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, system);

		app
	}

	#[test]
	fn update_applies_velocity() {
		let mut app = setup(call_update(Duration::from_millis(100)));
		let transform = GlobalTransform::from_xyz(3., 0., 2.);
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((transform, speed)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion(LinearMotion(
				(target - transform.translation()).normalize() * *speed
			))),
			app.world().entity(agent).get::<_Motion>()
		);
	}

	#[test]
	fn movement_constraint_excludes_immobilized() {
		let mut app = setup(call_update(Duration::from_millis(100)));
		let transform = GlobalTransform::from_xyz(3., 0., 2.);
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((transform, speed)),
				Immobilized,
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<_Motion>());
	}

	#[test]
	fn update_returns_not_done() {
		let mut app = setup(call_update(Duration::from_millis(100)));
		let transform = GlobalTransform::from_xyz(3., 0., 2.);
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((transform, speed)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Result(IsDone(false))),
			app.world().entity(agent).get::<_Result>()
		);
	}

	#[test]
	fn update_zeros_motion_when_direction_length_zero() {
		let mut app = setup(call_update(Duration::from_millis(100)));
		let transform = GlobalTransform::from_xyz(10., 0., 7.);
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((transform, speed)),
				_Motion(LinearMotion(Vec3::new(1., 2., 3.))),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion(LinearMotion::ZERO)),
			app.world().entity(agent).get::<_Motion>()
		);
	}

	#[test]
	fn update_zeros_motion_when_direction_length_not_computable() {
		let mut app = setup(call_update(Duration::from_millis(100)));
		let transform = GlobalTransform::from_xyz(10., 0., 7.);
		let target = Vec3::new(f32::NAN, 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((transform, speed)),
				_Motion(LinearMotion(Vec3::new(1., 2., 3.))),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion(LinearMotion::ZERO)),
			app.world().entity(agent).get::<_Motion>()
		);
	}

	#[test]
	fn update_returns_done_when_direction_length_zero() {
		let mut app = setup(
			call_update(Duration::from_millis(0)), // causes min_distance to become zero
		);
		let transform = GlobalTransform::from_xyz(10., 0., 7.);
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((transform, speed)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Result(IsDone(true))),
			app.world().entity(agent).get::<_Result>()
		);
	}

	#[test]
	fn update_returns_done_when_direction_lower_than_min_distance() {
		let delta = 4.;
		let speed = 11.;
		let mut app = setup(call_update(Duration::from_secs(delta as u64)));
		let transform = GlobalTransform::from_xyz(10., 0., 7.);
		let target = transform.translation() + Vec3::X * (speed * delta - 1.);
		let speed = Speed(UnitsPerSecond::from(speed));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<Physical<_Motion>>::to(target),
				_UpdateParams((transform, speed)),
				_Motion::default(),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);
		assert_eq!(
			(Some(&_Motion::default()), Some(&_Result(IsDone(true)))),
			(agent.get::<_Motion>(), agent.get::<_Result>())
		);
	}

	#[test]
	fn set_velocity_zero_when_calling_on_remove() {
		let mut app = setup(call_on_remove);
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(
			Some(&_Motion(LinearMotion::ZERO)),
			app.world().entity(entity).get::<_Motion>()
		);
	}

	#[test]
	fn do_not_set_velocity_zero_when_calling_on_remove_and_immobilized() {
		let mut app = setup(call_on_remove);
		let entity = app.world_mut().spawn(Immobilized).id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Motion>());
	}

	#[test]
	fn get_movement_direction() {
		let target = Vec3::new(1., 2., 3.);
		let position = Vec3::new(4., 7., -1.);
		let movement = Movement::<Physical<_Motion>>::to(target);

		let direction = movement.movement_direction(&GlobalTransform::from_translation(position));

		assert_eq!(Some(Dir3::try_from(target - position).unwrap()), direction);
	}

	#[test]
	fn get_no_movement_direction_when_target_is_position() {
		let target = Vec3::new(1., 2., 3.);
		let position = target;
		let movement = Movement::<Physical<_Motion>>::to(target);

		let direction = movement.movement_direction(&GlobalTransform::from_translation(position));

		assert_eq!(None, direction);
	}
}
