use crate::components::{Attack, AttackConfig, Attacker, OnCoolDown, Target};
use bevy::ecs::{
	entity::Entity,
	query::Without,
	system::{Commands, Query},
};

pub(crate) fn attack(
	mut commands: Commands,
	attackers: Query<(Entity, &Attack, &AttackConfig), Without<OnCoolDown>>,
) {
	for (id, attack, conf, ..) in &attackers {
		(conf.attack)(&mut commands, Attacker(id), Target(attack.0));
		commands.entity(id).insert(OnCoolDown(conf.cool_down));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Attacker, Target};
	use bevy::{
		app::{App, Update},
		ecs::component::Component,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use std::time::Duration;

	#[derive(Component, Debug, PartialEq)]
	struct _FakeAttack {
		attacker: Attacker,
		target: Target,
	}

	fn fake_attack(commands: &mut Commands, attacker: Attacker, target: Target) {
		commands.spawn(_FakeAttack { attacker, target });
	}

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, attack);

		app
	}

	#[test]
	fn use_attack_function() {
		let mut app = setup();
		let attacker = app
			.world
			.spawn((
				Attack(Entity::from_raw(11)),
				AttackConfig {
					attack: fake_attack,
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
					attack: fake_attack,
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
					attack: fake_attack,
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
}
