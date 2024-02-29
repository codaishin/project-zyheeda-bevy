use super::{IsDone, Movement, Units};
use crate::components::SimpleMovement;
use bevy::{ecs::system::EntityCommands, prelude::*};

impl Movement for SimpleMovement {
	fn update(&mut self, agent: &mut Transform, distance: Units) -> IsDone {
		let target = self.target;
		let direction = target - agent.translation;

		if distance > direction.length() {
			agent.translation = target;
			return true;
		}

		agent.translation += direction.normalize() * distance;
		false
	}

	fn cleanup(&self, agent: &mut EntityCommands) {
		let Some(cleanup) = self.cleanup else {
			return;
		};
		(cleanup)(agent);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::{Transform, Vec3};
	use common::test_tools::utils::{assert_eq_approx, SingleThreadedApp};

	#[test]
	fn move_to_target() {
		let mut movement = SimpleMovement {
			target: Vec3::X,
			..default()
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 1.);

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn do_not_move_fully_if_distance_too_small() {
		let mut movement = SimpleMovement {
			target: Vec3::new(2., 0., 0.),
			..default()
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 0.5);

		assert_eq!(Vec3::X * 0.5, agent.translation);
	}

	#[test]
	fn do_not_overshoot() {
		let mut movement = SimpleMovement {
			target: Vec3::X,
			..default()
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 100.);

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn done_when_target_reached() {
		let mut movement = SimpleMovement {
			target: Vec3::ONE,
			..default()
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		let is_done = movement.update(&mut agent, 100.);

		assert!(is_done);
	}

	#[test]
	fn not_done_when_target_reached() {
		let mut movement = SimpleMovement {
			target: Vec3::ONE,
			..default()
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		let is_done = movement.update(&mut agent, 0.1);

		assert!(!is_done);
	}

	#[test]
	fn no_rotation_change_when_on_target() {
		let mut movement = SimpleMovement {
			target: Vec3::ONE,
			..default()
		};
		let mut agent = Transform::from_translation(Vec3::ONE);

		agent.look_at(Vec3::new(2., 1., 2.), Vec3::Y);

		movement.update(&mut agent, 0.1);

		assert_eq_approx!(Vec3::new(2., 0., 2.).normalize(), agent.forward(), 0.00001);
	}

	#[test]
	fn use_cleanup() {
		#[derive(Component)]
		struct _Cleanup;

		fn call_cleanup(mut commands: Commands, query: Query<(Entity, &SimpleMovement)>) {
			for (id, movement) in &query {
				movement.cleanup(&mut commands.entity(id));
			}
		}

		let mut app = App::new_single_threaded([Update]);
		let movement = SimpleMovement {
			cleanup: Some(|agent| {
				agent.insert(_Cleanup);
			}),
			..default()
		};
		let movement = app.world.spawn(movement).id();

		app.add_systems(Update, call_cleanup);
		app.update();

		let movement = app.world.entity(movement);

		assert!(movement.contains::<_Cleanup>());
	}
}
