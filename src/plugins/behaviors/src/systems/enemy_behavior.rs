use crate::components::{Attack, Chase};
use bevy::prelude::*;
use common::traits::{
	accessors::get::GetterRef,
	handles_enemies::{AttackConfig, AttackTarget},
};

impl<T> EnemyBehaviorSystem for T {}

pub(crate) trait EnemyBehaviorSystem {
	fn select_behavior<TPlayer>(
		mut commands: Commands,
		agents: Query<(Entity, &GlobalTransform, &Self)>,
		players: Query<(Entity, &GlobalTransform), With<TPlayer>>,
		all: Query<(Entity, &GlobalTransform)>,
	) where
		TPlayer: Component,
		Self: GetterRef<AttackConfig> + Component + Sized,
	{
		let player = players.get_single().ok();

		for (id, transform, attack) in agents.iter() {
			let attack = attack.get();
			let foe = match attack.target {
				AttackTarget::Player => player,
				AttackTarget::Entity(entity) => all.get(entity).ok(),
			};
			let Some((foe, foe_transform)) = foe else {
				continue;
			};
			let distance = (transform.translation() - foe_transform.translation()).length();
			let Some(mut entity) = commands.get_entity(id) else {
				continue;
			};

			match strategy(attack, distance) {
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
}

enum Strategy {
	Attack,
	Chase,
	Idle,
}

fn strategy(attack: &AttackConfig, distance: f32) -> Strategy {
	if distance > *attack.aggro_range {
		return Strategy::Idle;
	}
	if distance > *attack.range {
		return Strategy::Chase;
	}

	Strategy::Attack
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::Units,
		traits::{clamp_zero_positive::ClampZeroPositive, handles_enemies::AttackConfig},
	};

	#[derive(Component)]
	struct _Player;

	#[derive(Component, Default)]
	struct _Enemy(AttackConfig);

	impl GetterRef<AttackConfig> for _Enemy {
		fn get(&self) -> &AttackConfig {
			let _Enemy(attack) = self;
			attack
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, _Enemy::select_behavior::<_Player>);

		app
	}

	#[test]
	fn chase_player() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(2., 0., 1.),
				Attack(player),
				_Enemy(AttackConfig {
					aggro_range: Units::new(2.),
					range: Units::new(1.),
					target: AttackTarget::Player,
					..default()
				}),
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
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.5),
				Chase(player),
				_Enemy(AttackConfig {
					aggro_range: Units::new(2.),
					range: Units::new(1.),
					target: AttackTarget::Player,
					..default()
				}),
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
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(3., 0., 3.),
				Chase(player),
				Attack(player),
				_Enemy(AttackConfig {
					aggro_range: Units::new(2.),
					range: Units::new(1.),
					target: AttackTarget::Player,
					..default()
				}),
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
				_Enemy(AttackConfig {
					aggro_range: Units::new(2.),
					range: Units::new(1.),
					target: AttackTarget::Entity(foe),
					..default()
				}),
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
				_Enemy(AttackConfig {
					aggro_range: Units::new(2.),
					range: Units::new(1.),
					target: AttackTarget::Entity(foe),
					..default()
				}),
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
				_Enemy(AttackConfig {
					aggro_range: Units::new(2.),
					range: Units::new(1.),
					target: AttackTarget::Entity(foe),
					..default()
				}),
			))
			.id();

		app.update();

		let enemy = app.world().entity(enemy);

		assert_eq!((None, None), (enemy.get::<Chase>(), enemy.get::<Attack>()));
	}
}
