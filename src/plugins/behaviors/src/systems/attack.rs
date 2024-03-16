pub(crate) mod execute_beam;

use crate::components::{Attack, AttackConfig, Attacker, OnCoolDown, Target};
use bevy::ecs::{
	component::Component,
	entity::Entity,
	query::Without,
	removal_detection::RemovedComponents,
	system::{Commands, Query},
};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};
use std::sync::Arc;

#[derive(Component)]
pub(crate) struct Despawn(Arc<dyn Fn(&mut Commands) + Sync + Send>);

pub(crate) fn attack(
	mut commands: Commands,
	mut removed_attacks: RemovedComponents<Attack>,
	attackers: Query<(Entity, &Attack, &AttackConfig), Without<OnCoolDown>>,
	despawns: Query<&Despawn>,
) {
	for (id, attack, conf, ..) in &attackers {
		let spawn = &conf.spawn;
		let despawn = spawn.spawn(&mut commands, Attacker(id), Target(attack.0));
		commands.try_insert_on(id, (OnCoolDown(conf.cool_down), Despawn(despawn)));
	}

	for id in removed_attacks.read() {
		let Ok(despawn) = despawns.get(id) else {
			continue;
		};
		(despawn.0)(&mut commands);
		commands.try_remove_from::<Despawn>(id);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Attacker, Target},
		traits::{SpawnAttack, ToArc},
	};
	use bevy::{
		app::{App, Update},
		ecs::component::Component,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use std::{sync::Arc, time::Duration};

	#[derive(Component, Debug, PartialEq)]
	struct _FakeAttack {
		attacker: Attacker,
		target: Target,
	}

	struct _FakeAttackConfig;

	impl SpawnAttack for _FakeAttackConfig {
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

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, attack);

		app
	}

	#[test]
	fn use_spawn_function() {
		let mut app = setup();
		let attacker = app
			.world
			.spawn((
				Attack(Entity::from_raw(11)),
				AttackConfig {
					spawn: _FakeAttackConfig.to_arc(),
					cool_down: Duration::ZERO,
				},
			))
			.id();

		app.update();

		let fake_attack = app
			.world
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
			.world
			.spawn((
				Attack(Entity::from_raw(11)),
				AttackConfig {
					spawn: _FakeAttackConfig.to_arc(),
					cool_down: Duration::from_secs(42),
				},
			))
			.id();

		app.update();

		let attacker = app.world.entity(attacker);

		assert_eq!(
			Some(&OnCoolDown(Duration::from_secs(42))),
			attacker.get::<OnCoolDown>()
		);
	}

	#[test]
	fn do_nothing_when_cool_down_present() {
		let mut app = setup();
		let attacker = app
			.world
			.spawn((
				Attack(Entity::from_raw(11)),
				OnCoolDown(Duration::from_millis(100)),
				AttackConfig {
					spawn: _FakeAttackConfig.to_arc(),
					cool_down: Duration::ZERO,
				},
			))
			.id();

		app.update();

		let fake_attack = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<_FakeAttack>());
		let attacker = app.world.entity(attacker);

		assert_eq!(
			(None, Some(&OnCoolDown(Duration::from_millis(100)))),
			(fake_attack, attacker.get::<OnCoolDown>())
		);
	}

	#[test]
	fn use_despawn_function_when_attack_removed() {
		let mut app = setup();
		let attacker = app
			.world
			.spawn((
				Attack(Entity::from_raw(11)),
				AttackConfig {
					spawn: _FakeAttackConfig.to_arc(),
					cool_down: Duration::ZERO,
				},
			))
			.id();

		app.update();

		app.world.entity_mut(attacker).remove::<Attack>();

		app.update();

		let fake_attack = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<_FakeAttack>());

		assert_eq!(None, fake_attack);
	}

	#[test]
	fn remove_despawn_component_when_attack_removed() {
		let mut app = setup();
		let attacker = app
			.world
			.spawn((
				Attack(Entity::from_raw(11)),
				AttackConfig {
					spawn: _FakeAttackConfig.to_arc(),
					cool_down: Duration::ZERO,
				},
			))
			.id();

		app.update();

		app.world.entity_mut(attacker).remove::<Attack>();

		app.update();

		let despawners = app.world.iter_entities().find_map(|e| e.get::<Despawn>());

		assert!(despawners.is_none());
	}
}
