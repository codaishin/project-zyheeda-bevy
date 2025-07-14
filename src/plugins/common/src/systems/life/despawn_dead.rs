use crate::{components::life::Life, traits::try_despawn::TryDespawn};
use bevy::prelude::*;

impl Life {
	pub(crate) fn despawn_dead(mut commands: Commands, agents: Query<(Entity, &Self)>) {
		for entity in agents.iter().filter_map(dead) {
			commands.try_despawn(entity);
		}
	}
}

fn dead((entity, Life(health)): (Entity, &Life)) -> Option<Entity> {
	if health.current <= 0. {
		Some(entity)
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::attributes::health::Health;

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, Life::despawn_dead);

		app
	}

	#[test]
	fn despawn_when_health_zero() {
		let mut app = setup();
		let health = app
			.world_mut()
			.spawn(Life::from(Health {
				current: 0.,
				max: 100.,
			}))
			.id();

		app.update();

		assert!(app.world().get_entity(health).is_err());
	}

	#[test]
	fn do_not_despawn_when_health_above_zero() {
		let mut app = setup();
		let health = app
			.world_mut()
			.spawn(Life::from(Health {
				current: 1.,
				max: 100.,
			}))
			.id();

		app.update();

		assert!(app.world().get_entity(health).is_ok());
	}

	#[test]
	fn despawn_when_health_zero_below_zero() {
		let mut app = setup();
		let health = app
			.world_mut()
			.spawn(Life::from(Health {
				current: -1.,
				max: 100.,
			}))
			.id();

		app.update();

		assert!(app.world().get_entity(health).is_err());
	}
}
