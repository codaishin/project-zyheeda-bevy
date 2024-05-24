use crate::{
	components::Immobilized,
	systems::idle::SetToIdle,
	traits::{MovementData, MovementVelocityBased},
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::Without,
		system::{Commands, Query},
	},
	transform::components::GlobalTransform,
};

pub(crate) fn execute_move_velocity_based<
	TMovementConfig: Component + MovementData,
	TMovement: Component + MovementVelocityBased,
>(
	mut commands: Commands,
	agents: Query<(Entity, &GlobalTransform, &TMovementConfig, &TMovement), Without<Immobilized>>,
) -> SetToIdle<TMovement> {
	let done_entities = agents
		.iter()
		.filter_map(|(id, transform, config, movement)| {
			let entity = &mut commands.get_entity(id)?;
			let (speed, ..) = config.get_movement_data();
			let position = transform.translation();

			match movement.update(entity, position, speed).is_done() {
				true => Some(entity.id()),
				false => None,
			}
		})
		.collect();

	SetToIdle::new(done_entities)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::MovementMode, traits::IsDone};
	use bevy::{
		app::{App, Update},
		ecs::system::{EntityCommands, In, IntoSystem},
		math::Vec3,
		transform::components::Transform,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::UnitsPerSecond,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[derive(Component)]
	struct ConfigFast;

	#[derive(Component)]
	struct ConfigSlow;

	#[derive(Component, Default, Debug)]
	struct _Movement(IsDone);

	#[derive(Component, PartialEq, Debug)]
	struct _MoveParams((Vec3, UnitsPerSecond));

	#[derive(Component, Debug, PartialEq)]
	struct _Idle;

	impl MovementData for ConfigFast {
		fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
			(UnitsPerSecond::new(11.), MovementMode::Fast)
		}
	}

	impl MovementData for ConfigSlow {
		fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
			(UnitsPerSecond::new(2.), MovementMode::Slow)
		}
	}

	impl MovementVelocityBased for _Movement {
		fn update(
			&self,
			agent: &mut EntityCommands,
			position: Vec3,
			speed: UnitsPerSecond,
		) -> IsDone {
			agent.insert(_MoveParams((position, speed)));
			self.0
		}
	}

	fn idle(set_to_idle: In<SetToIdle<_Movement>>, mut commands: Commands) {
		for entity in set_to_idle.entities.iter() {
			commands.entity(*entity).insert(_Idle);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(
				execute_move_velocity_based::<ConfigFast, _Movement>.pipe(idle),
				execute_move_velocity_based::<ConfigSlow, _Movement>.pipe(idle),
			),
		);

		app
	}

	#[test]
	fn apply_speed_fast() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let movement = _Movement(false.into());
		let agent = app
			.world
			.spawn((config, GlobalTransform::from(transform), movement))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_MoveParams((
				Vec3::new(1., 2., 3.),
				UnitsPerSecond::new(11.)
			))),
			agent.get::<_MoveParams>()
		);
	}

	#[test]
	fn apply_speed_slow() {
		let mut app = setup();
		let transform = Transform::from_xyz(4., 5., 6.);
		let config = ConfigSlow;
		let movement = _Movement(false.into());
		let agent = app
			.world
			.spawn((config, GlobalTransform::from(transform), movement))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_MoveParams((
				Vec3::new(4., 5., 6.),
				UnitsPerSecond::new(2.)
			))),
			agent.get::<_MoveParams>()
		);
	}

	#[test]
	fn return_entity_to_idle_when_done() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let movement = _Movement(true.into());

		let agent = app
			.world
			.spawn((config, GlobalTransform::from(transform), movement))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Idle), agent.get::<_Idle>());
	}

	#[test]
	fn do_not_return_entity_to_idle_when_not_done() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let movement = _Movement(false.into());

		let agent = app
			.world
			.spawn((config, GlobalTransform::from(transform), movement))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<_Idle>());
	}

	#[test]
	fn no_movement_when_immobilized() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				ConfigFast,
				GlobalTransform::from_xyz(1., 2., 3.),
				_Movement(false.into()),
				Immobilized,
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<_MoveParams>());
	}
}
