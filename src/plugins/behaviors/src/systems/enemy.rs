use crate::components::{Attack, Chase, Enemy, Foe};
use bevy::{
	ecs::{
		entity::Entity,
		query::With,
		system::{Commands, Query},
	},
	transform::components::GlobalTransform,
};
use common::components::Player;

pub(crate) fn enemy(
	mut commands: Commands,
	agents: Query<(Entity, &GlobalTransform, &Enemy)>,
	players: Query<(Entity, &GlobalTransform), With<Player>>,
	all: Query<(Entity, &GlobalTransform)>,
) {
	let player = players.get_single().ok();
	let valid_foe = |(id, transform, agent): (Entity, &GlobalTransform, &Enemy)| {
		let (foe_id, foe_transform) = match agent.foe {
			Foe::Player => player?,
			Foe::Entity(entity) => all.get(entity).ok()?,
		};
		let distance = (transform.translation() - foe_transform.translation()).length();
		Some((id, *agent, foe_id, distance))
	};

	for (id, agent, foe, distance) in agents.iter().filter_map(valid_foe) {
		let Some(mut entity) = commands.get_entity(id) else {
			continue;
		};

		match strategy(agent, distance) {
			Strategy::Attack => {
				entity.try_insert(Attack(foe));
				entity.remove::<Chase>();
			}
			Strategy::Chase => {
				entity.try_insert(Chase(foe));
				entity.remove::<Attack>();
			}
			Strategy::Idle => {
				entity.remove::<Chase>();
				entity.remove::<Attack>();
			}
		}
	}
}

enum Strategy {
	Attack,
	Chase,
	Idle,
}

fn strategy(enemy: Enemy, distance: f32) -> Strategy {
	if distance > enemy.aggro_range {
		return Strategy::Idle;
	}
	if distance > enemy.attack_range {
		return Strategy::Chase;
	}
	Strategy::Attack
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Attack, Chase, Enemy, Foe};
	use bevy::{
		app::{App, Update},
		transform::components::GlobalTransform,
	};
	use common::components::Player;

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, enemy);

		app
	}

	#[test]
	fn chase_player() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), Player))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(2., 0., 1.),
				Attack(player),
				Enemy {
					attack_range: 1.,
					aggro_range: 2.,
					foe: Foe::Player,
				},
			))
			.id();

		app.update();

		let enemy = app.world().entity(enemy);

		assert_eq!(
			(Some(&Chase(player)), None),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
	}

	#[test]
	fn attack_player() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), Player))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.5),
				Chase(player),
				Enemy {
					attack_range: 1.,
					aggro_range: 2.,
					foe: Foe::Player,
				},
			))
			.id();

		app.update();

		let enemy = app.world().entity(enemy);

		assert_eq!(
			(None, Some(&Attack(player))),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
	}

	#[test]
	fn do_nothing_when_out_of_aggro_range_of_player() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), Player))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(3., 0., 3.),
				Chase(player),
				Attack(player),
				Enemy {
					attack_range: 1.,
					aggro_range: 2.,
					foe: Foe::Player,
				},
			))
			.id();

		app.update();

		let enemy = app.world().entity(enemy);

		assert_eq!((None, None), (enemy.get::<Chase>(), enemy.get::<Attack>()));
	}

	#[test]
	fn chase_entity() {
		let mut app = setup();
		let foe = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 0.))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(2., 0., 1.),
				Attack(foe),
				Enemy {
					attack_range: 1.,
					aggro_range: 2.,
					foe: Foe::Entity(foe),
				},
			))
			.id();

		app.update();

		let enemy = app.world().entity(enemy);

		assert_eq!(
			(Some(&Chase(foe)), None),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
	}

	#[test]
	fn attack_entity() {
		let mut app = setup();
		let foe = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 0.))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.5),
				Chase(foe),
				Enemy {
					attack_range: 1.,
					aggro_range: 2.,
					foe: Foe::Entity(foe),
				},
			))
			.id();

		app.update();

		let enemy = app.world().entity(enemy);

		assert_eq!(
			(None, Some(&Attack(foe))),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
	}

	#[test]
	fn do_nothing_when_out_of_aggro_range_of_entity() {
		let mut app = setup();
		let foe = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 0.))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(3., 0., 3.),
				Chase(foe),
				Attack(foe),
				Enemy {
					attack_range: 1.,
					aggro_range: 2.,
					foe: Foe::Entity(foe),
				},
			))
			.id();

		app.update();

		let enemy = app.world().entity(enemy);

		assert_eq!((None, None), (enemy.get::<Chase>(), enemy.get::<Attack>()));
	}
}
