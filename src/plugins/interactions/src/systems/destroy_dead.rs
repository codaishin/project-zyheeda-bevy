use crate::components::Destroy;
use bevy::ecs::{
	entity::Entity,
	system::{Commands, Query},
};
use common::components::Health;

pub(crate) fn set_dead_to_be_destroyed(mut commands: Commands, agents: Query<(Entity, &Health)>) {
	for id in agents.iter().filter_map(dead) {
		commands.entity(id).insert(Destroy);
	}
}

fn dead((id, health): (Entity, &Health)) -> Option<Entity> {
	if health.current <= 0 {
		Some(id)
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Destroy;
	use bevy::app::{App, Update};
	use common::components::Health;

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, set_dead_to_be_destroyed);

		app
	}

	#[test]
	fn add_destroy_when_health_zero() {
		let mut app = setup();
		let health = app
			.world
			.spawn(Health {
				current: 0,
				max: 100,
			})
			.id();

		app.update();

		let health = app.world.entity(health);

		assert!(health.contains::<Destroy>());
	}

	#[test]
	fn don_not_add_destroy_when_health_above_zero() {
		let mut app = setup();
		let health = app
			.world
			.spawn(Health {
				current: 1,
				max: 100,
			})
			.id();

		app.update();

		let health = app.world.entity(health);

		assert!(!health.contains::<Destroy>());
	}

	#[test]
	fn add_destroy_when_health_zero_below_zero() {
		let mut app = setup();
		let health = app
			.world
			.spawn(Health {
				current: -1,
				max: 100,
			})
			.id();

		app.update();

		let health = app.world.entity(health);

		assert!(health.contains::<Destroy>());
	}
}
