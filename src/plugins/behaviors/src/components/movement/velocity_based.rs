use super::{Movement, OnMovementRemoved};
use crate::traits::{IsDone, MovementUpdate, change_per_frame::MinDistance};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	components::immobilized::Immobilized,
	tools::speed::Speed,
	traits::animation::GetMovementDirection,
};
use std::time::Duration;

#[derive(PartialEq, Debug, Default)]
pub struct VelocityBased;

impl VelocityBased {
	const SENSITIVITY: f32 = 2.;
}

impl MinDistance for VelocityBased {
	fn min_distance(speed: Speed, delta: Duration) -> f32 {
		delta.as_secs_f32() * *speed * Self::SENSITIVITY
	}
}

impl MovementUpdate for Movement<VelocityBased> {
	type TComponents<'a> = &'a GlobalTransform;
	type TConstraint = Without<Immobilized>;

	fn update(
		&self,
		agent: &mut EntityCommands,
		transform: &GlobalTransform,
		speed: Speed,
		delta: Duration,
	) -> IsDone {
		if self.target == transform.translation() {
			return IsDone(true);
		}

		let direction = self.target - transform.translation();

		if direction.length() < VelocityBased::min_distance(speed, delta) {
			return IsDone(true);
		}

		agent.try_insert(Velocity::linear(direction.normalize() * *speed));
		IsDone(false)
	}
}

impl OnMovementRemoved for Movement<VelocityBased> {
	type TConstraint = Without<Immobilized>;

	fn on_movement_removed(entity: &mut EntityCommands) {
		entity.try_insert(Velocity::zero());
	}
}

impl GetMovementDirection for Movement<VelocityBased> {
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
	use bevy_rapier3d::dynamics::Velocity;
	use common::{tools::UnitsPerSecond, traits::clamp_zero_positive::ClampZeroPositive};
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Result(IsDone);

	#[derive(Component)]
	struct _UpdateParams((GlobalTransform, Speed));

	#[allow(clippy::type_complexity)]
	fn call_update(
		delta: Duration,
	) -> impl Fn(
		Commands,
		Query<
			(Entity, &Movement<VelocityBased>, &_UpdateParams),
			<Movement<VelocityBased> as MovementUpdate>::TConstraint,
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
		entities: Query<Entity, <Movement<VelocityBased> as OnMovementRemoved>::TConstraint>,
	) {
		for entity in &entities {
			let entity = &mut commands.entity(entity);
			Movement::<VelocityBased>::on_movement_removed(entity);
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
		let speed = Speed(UnitsPerSecond::new(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
				_UpdateParams((transform, speed)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Velocity::linear(
				(target - transform.translation()).normalize() * *speed
			)),
			app.world().entity(agent).get::<Velocity>()
		);
	}

	#[test]
	fn movement_constraint_excludes_immobilized() {
		let mut app = setup(call_update(Duration::from_millis(100)));
		let transform = GlobalTransform::from_xyz(3., 0., 2.);
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::new(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
				_UpdateParams((transform, speed)),
				Immobilized,
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<Velocity>());
	}

	#[test]
	fn update_returns_not_done() {
		let mut app = setup(call_update(Duration::from_millis(100)));
		let transform = GlobalTransform::from_xyz(3., 0., 2.);
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::new(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
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
	fn update_removes_velocity_when_direction_length_zero() {
		let mut app = setup(call_update(Duration::from_millis(100)));
		let transform = GlobalTransform::from_xyz(10., 0., 7.);
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::new(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
				_UpdateParams((transform, speed)),
				Velocity::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Velocity::default()),
			app.world().entity(agent).get::<Velocity>()
		);
	}

	#[test]
	fn update_returns_done_when_direction_length_zero() {
		let mut app = setup(
			call_update(Duration::from_millis(0)), // causes min_distance to become zero
		);
		let transform = GlobalTransform::from_xyz(10., 0., 7.);
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::new(11.));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
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
		let target =
			transform.translation() + Vec3::X * (VelocityBased::SENSITIVITY * speed * delta - 1.);
		let speed = Speed(UnitsPerSecond::new(speed));
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
				_UpdateParams((transform, speed)),
				Velocity::default(),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);
		assert_eq!(
			(Some(&Velocity::default()), Some(&_Result(IsDone(true)))),
			(agent.get::<Velocity>(), agent.get::<_Result>())
		);
	}

	#[test]
	fn set_velocity_zero_when_calling_on_remove() {
		let mut app = setup(call_on_remove);
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(
			Some(&Velocity::zero()),
			app.world().entity(entity).get::<Velocity>()
		);
	}

	#[test]
	fn do_not_set_velocity_zero_when_calling_on_remove_and_immobilized() {
		let mut app = setup(call_on_remove);
		let entity = app.world_mut().spawn(Immobilized).id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<Velocity>());
	}

	#[test]
	fn get_movement_direction() {
		let target = Vec3::new(1., 2., 3.);
		let position = Vec3::new(4., 7., -1.);
		let movement = Movement::<VelocityBased>::to(target);

		let direction = movement.movement_direction(&GlobalTransform::from_translation(position));

		assert_eq!(Some(Dir3::try_from(target - position).unwrap()), direction);
	}

	#[test]
	fn get_no_movement_direction_when_target_is_position() {
		let target = Vec3::new(1., 2., 3.);
		let position = target;
		let movement = Movement::<VelocityBased>::to(target);

		let direction = movement.movement_direction(&GlobalTransform::from_translation(position));

		assert_eq!(None, direction);
	}
}
