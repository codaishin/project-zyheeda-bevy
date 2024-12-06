use crate::{
	components::{Attack, Attacker, OnCoolDown, Target},
	traits::GetAttackSpawner,
};
use bevy::prelude::*;
use common::traits::{
	accessors::get::GetterRef,
	handles_behaviors::AttackCoolDown,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};
use std::sync::Arc;

impl<T> AttackSystem for T {}

pub(crate) trait AttackSystem {
	fn attack<TGetAttackSpawner>(
		mut commands: Commands,
		mut removed_attacks: RemovedComponents<Attack>,
		enemies: Query<(Entity, &Attack, &Self), Without<OnCoolDown>>,
		despawners: Query<&Despawn>,
	) where
		TGetAttackSpawner: GetAttackSpawner<Self>,
		Self: GetterRef<AttackCoolDown> + Component + Sized,
	{
		for (entity, Attack(target), enemy) in &enemies {
			let AttackCoolDown(cold_down) = enemy.get();
			let spawner = TGetAttackSpawner::attack_spawner(enemy);
			let despawn_fn = spawner.spawn(&mut commands, Attacker(entity), Target(*target));
			commands.try_insert_on(entity, (OnCoolDown(*cold_down), Despawn(despawn_fn)));
		}

		for entity in removed_attacks.read() {
			let Ok(Despawn(despawn_fn)) = despawners.get(entity) else {
				continue;
			};
			despawn_fn(&mut commands);
			commands.try_remove_from::<Despawn>(entity);
		}
	}
}

#[derive(Component)]
pub(crate) struct Despawn(Arc<dyn Fn(&mut Commands) + Sync + Send>);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Attacker, Target},
		traits::SpawnAttack,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use std::{sync::Arc, time::Duration};

	#[derive(Component)]
	struct _Enemy(AttackCoolDown);

	impl GetterRef<AttackCoolDown> for _Enemy {
		fn get(&self) -> &AttackCoolDown {
			let _Enemy(cool_down) = self;
			cool_down
		}
	}

	struct _FakeAttackSpawnerFactory;

	impl GetAttackSpawner<_Enemy> for _FakeAttackSpawnerFactory {
		fn attack_spawner(_: &_Enemy) -> Arc<dyn SpawnAttack> {
			Arc::new(_FakeAttackSpawner)
		}
	}

	struct _FakeAttackSpawner;

	impl SpawnAttack for _FakeAttackSpawner {
		fn spawn(
			&self,
			commands: &mut Commands,
			attacker: Attacker,
			target: Target,
		) -> Arc<dyn Fn(&mut Commands) + Sync + Send> {
			let attack = commands.spawn(_FakeAttack { attacker, target }).id();
			Arc::new(move |commands| {
				commands.entity(attack).despawn();
			})
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _FakeAttack {
		attacker: Attacker,
		target: Target,
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Enemy::attack::<_FakeAttackSpawnerFactory>);

		app
	}

	#[test]
	fn use_spawn_function() {
		let mut app = setup();
		let attacker = app
			.world_mut()
			.spawn((
				Attack(Entity::from_raw(11)),
				_Enemy(AttackCoolDown::default()),
			))
			.id();

		app.update();

		let fake_attack = app
			.world()
			.iter_entities()
			.find_map(|e| e.get::<_FakeAttack>());

		assert_eq!(
			Some(&_FakeAttack {
				attacker: Attacker(attacker),
				target: Target(Entity::from_raw(11))
			}),
			fake_attack
		);
	}

	#[test]
	fn insert_cool_down() {
		let mut app = setup();
		let attacker = app
			.world_mut()
			.spawn((
				Attack(Entity::from_raw(11)),
				_Enemy(AttackCoolDown(Duration::from_secs(42))),
			))
			.id();

		app.update();

		let attacker = app.world().entity(attacker);

		assert_eq!(
			Some(&OnCoolDown(Duration::from_secs(42))),
			attacker.get::<OnCoolDown>()
		);
	}

	#[test]
	fn do_nothing_when_cool_down_present() {
		let mut app = setup();
		let attacker = app
			.world_mut()
			.spawn((
				Attack(Entity::from_raw(11)),
				_Enemy(AttackCoolDown::default()),
				OnCoolDown(Duration::from_millis(100)),
			))
			.id();

		app.update();

		let fake_attack = app
			.world()
			.iter_entities()
			.find_map(|e| e.get::<_FakeAttack>());
		let attacker = app.world().entity(attacker);

		assert_eq!(
			(None, Some(&OnCoolDown(Duration::from_millis(100)))),
			(fake_attack, attacker.get::<OnCoolDown>())
		);
	}

	#[test]
	fn use_despawn_function_when_attack_removed() {
		let mut app = setup();
		let attacker = app
			.world_mut()
			.spawn((
				Attack(Entity::from_raw(11)),
				_Enemy(AttackCoolDown::default()),
			))
			.id();

		app.update();

		app.world_mut().entity_mut(attacker).remove::<Attack>();

		app.update();

		let fake_attack = app
			.world()
			.iter_entities()
			.find_map(|e| e.get::<_FakeAttack>());

		assert_eq!(None, fake_attack);
	}

	#[test]
	fn remove_despawn_component_when_attack_removed() {
		let mut app = setup();
		let attacker = app
			.world_mut()
			.spawn((
				Attack(Entity::from_raw(11)),
				_Enemy(AttackCoolDown::default()),
			))
			.id();

		app.update();

		app.world_mut().entity_mut(attacker).remove::<Attack>();

		app.update();

		let despawners = app.world().iter_entities().find_map(|e| e.get::<Despawn>());

		assert!(despawners.is_none());
	}
}
