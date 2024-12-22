use crate::components::{Attack, Chase};
use bevy::prelude::*;
use common::{
	tools::{aggro_range::AggroRange, attack_range::AttackRange},
	traits::{accessors::get::Getter, handles_enemies::EnemyTarget},
};

impl<T> SelectBehavior for T {}

pub(crate) trait SelectBehavior {
	fn select_behavior<TPlayer>(
		mut commands: Commands,
		agents: Query<(Entity, &GlobalTransform, &Self)>,
		players: Query<(Entity, &GlobalTransform), With<TPlayer>>,
		all: Query<(Entity, &GlobalTransform)>,
	) where
		Self: Component + Sized + Getter<AggroRange> + Getter<AttackRange> + Getter<EnemyTarget>,
		TPlayer: Component,
	{
		let player = players.get_single().ok();

		for (entity, transform, agent) in &agents {
			let target = match agent.get() {
				EnemyTarget::Player => player,
				EnemyTarget::Entity(entity) => all.get(entity).ok(),
			};
			let Some((target, target_transform)) = target else {
				continue;
			};
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			let distance = (transform.translation() - target_transform.translation()).length();

			match strategy(agent, distance) {
				Behavior::Attack => {
					entity.try_insert(Attack(target));
					entity.remove::<Chase>();
				}
				Behavior::Chase => {
					entity.try_insert(Chase(target));
					entity.remove::<Attack>();
				}
				Behavior::Idle => {
					entity.remove::<Chase>();
					entity.remove::<Attack>();
				}
			}
		}
	}
}

enum Behavior {
	Attack,
	Chase,
	Idle,
}

fn strategy<TAgent>(enemy: &TAgent, distance: f32) -> Behavior
where
	TAgent: Getter<AggroRange> + Getter<AttackRange>,
{
	if distance > aggro_range(enemy) {
		return Behavior::Idle;
	}
	if distance > attack_range(enemy) {
		return Behavior::Chase;
	}
	Behavior::Attack
}

fn aggro_range<TAgent>(agent: &TAgent) -> f32
where
	TAgent: Getter<AggroRange>,
{
	**agent.get()
}

fn attack_range<TAgent>(agent: &TAgent) -> f32
where
	TAgent: Getter<AttackRange>,
{
	**agent.get()
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{tools::Units, traits::clamp_zero_positive::ClampZeroPositive};

	#[derive(Component)]
	struct _Enemy {
		aggro_range: AggroRange,
		attack_range: AttackRange,
		target: EnemyTarget,
	}

	impl Getter<AggroRange> for _Enemy {
		fn get(&self) -> AggroRange {
			self.aggro_range
		}
	}

	impl Getter<AttackRange> for _Enemy {
		fn get(&self) -> AttackRange {
			self.attack_range
		}
	}

	impl Getter<EnemyTarget> for _Enemy {
		fn get(&self) -> EnemyTarget {
			self.target
		}
	}

	#[derive(Component)]
	struct _Player;

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
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Player,
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
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.5),
				Chase(player),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Player,
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
			.spawn((GlobalTransform::from_xyz(1., 0., 0.), _Player))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(3., 0., 3.),
				Chase(player),
				Attack(player),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Player,
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
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 0.))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(2., 0., 1.),
				Attack(target),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Entity(target),
				},
			))
			.id();

		app.update();

		let enemy = app.world().entity(enemy);

		assert_eq!(
			(Some(&Chase(target)), None),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
	}

	#[test]
	fn attack_entity() {
		let mut app = setup();
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 0.))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 0., 0.5),
				Chase(target),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Entity(target),
				},
			))
			.id();

		app.update();

		let enemy = app.world().entity(enemy);

		assert_eq!(
			(None, Some(&Attack(target))),
			(enemy.get::<Chase>(), enemy.get::<Attack>())
		);
	}

	#[test]
	fn do_nothing_when_out_of_aggro_range_of_entity() {
		let mut app = setup();
		let target = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 0., 0.))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(3., 0., 3.),
				Chase(target),
				Attack(target),
				_Enemy {
					attack_range: Units::new(1.).into(),
					aggro_range: Units::new(2.).into(),
					target: EnemyTarget::Entity(target),
				},
			))
			.id();

		app.update();

		let enemy = app.world().entity(enemy);

		assert_eq!((None, None), (enemy.get::<Chase>(), enemy.get::<Attack>()));
	}
}
