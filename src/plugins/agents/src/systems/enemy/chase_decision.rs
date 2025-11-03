use crate::components::{
	enemy::{Enemy, attacking::Attacking, chasing::Chasing},
	player::Player,
};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl Enemy {
	pub(crate) fn chase_decision(
		mut commands: ZyheedaCommands,
		players: Query<(Entity, &Transform), With<Player>>,
		enemies: Query<(Entity, &Self, &Transform, Option<&Attacking>)>,
	) {
		let Ok((player, player_transform)) = players.single() else {
			return;
		};

		for (entity, enemy, transform, attacking) in &enemies {
			let should_chase = || {
				let distance = (player_transform.translation - transform.translation).length();

				if distance > *enemy.aggro_range {
					return false;
				}

				if distance > enemy.min_target_distance.map(|d| *d).unwrap_or_default() {
					return true;
				}

				attacking.map(|a| !a.has_los).unwrap_or(false)
			};

			commands.try_apply_on(&entity, |mut e| match should_chase() {
				true => {
					e.try_insert(Chasing { player });
				}
				false => {
					e.try_remove::<Chasing>();
				}
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		enemy::{attacking::Attacking, chasing::Chasing},
		player::Player,
	};
	use common::tools::Units;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Enemy::chase_decision);

		app
	}

	#[test]
	fn chase_when_inside_aggro_range() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					aggro_range: Units::from(5.),
					..default()
				},
				Transform::from_xyz(1., 2., 7.9),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Chasing { player }),
			app.world().entity(enemy).get::<Chasing>(),
		);
	}

	#[test]
	fn do_not_chase_when_outside_aggro_range() {
		let mut app = setup();
		app.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)));
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					aggro_range: Units::from(5.),
					..default()
				},
				Transform::from_xyz(1., 2., 8.1),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(enemy).get::<Chasing>());
	}

	#[test]
	fn do_not_chase_when_inside_min_distance() {
		let mut app = setup();
		app.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)));
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					aggro_range: Units::from(5.),
					min_target_distance: Some(Units::from(2.)),
					..default()
				},
				Transform::from_xyz(1., 2., 4.9),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(enemy).get::<Chasing>());
	}

	#[test]
	fn chase_when_inside_min_range_but_attacking_without_los() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					aggro_range: Units::from(5.),
					min_target_distance: Some(Units::from(2.)),
					..default()
				},
				Transform::from_xyz(1., 2., 4.9),
				Attacking {
					has_los: false,
					player,
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Chasing { player }),
			app.world().entity(enemy).get::<Chasing>(),
		);
	}

	#[test]
	fn do_not_chase_when_inside_min_range_but_attacking_with_los() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					aggro_range: Units::from(5.),
					min_target_distance: Some(Units::from(2.)),
					..default()
				},
				Transform::from_xyz(1., 2., 4.9),
				Attacking {
					has_los: true,
					player,
				},
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(enemy).get::<Chasing>());
	}

	#[test]
	fn remove_chase_when_out_of_range() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					aggro_range: Units::from(5.),
					..default()
				},
				Transform::from_xyz(1., 2., 8.1),
				Chasing { player },
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(enemy).get::<Chasing>());
	}
}
