use crate::components::{Attack, OnCoolDown};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{
		handles_enemies::{Attacker, EnemyAttack, Target},
		try_despawn::TryDespawn,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

impl<T> AttackSystem for T {}

pub trait AttackSystem {
	fn attack(
		mut commands: Commands,
		mut stop_attacks: RemovedComponents<Attack>,
		attackers: Query<(Entity, &PersistentEntity, &Self, &Attack), Without<OnCoolDown>>,
		ongoing: Query<&Ongoing>,
	) where
		Self: Component + EnemyAttack + Sized,
	{
		for (entity, persistent_entity, enemy, Attack(target)) in &attackers {
			let mut attack_entity = commands.spawn_empty();
			let attack = attack_entity.id();

			enemy.insert_attack(
				&mut attack_entity,
				Attacker(*persistent_entity),
				Target(*target),
			);
			attack_entity
				.commands()
				.try_insert_on(entity, (OnCoolDown(enemy.cool_down()), Ongoing(attack)));
		}

		for attacker in stop_attacks.read() {
			let Ok(Ongoing(attack)) = ongoing.get(attacker).cloned() else {
				continue;
			};
			commands.try_despawn(attack);
			commands.try_remove_from::<Ongoing>(attacker);
		}
	}
}

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct Ongoing(Entity);

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::component::Component,
		prelude::EntityCommands,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{handles_enemies::EnemyAttack, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::automock;
	use std::time::Duration;

	#[derive(Component, Debug, PartialEq)]
	struct _FakeAttack {
		attacker: Attacker,
		target: Target,
	}

	#[derive(Component, Debug, PartialEq)]
	struct _FakeAttackEmpty;

	#[derive(Component, NestedMocks)]
	struct _Enemy {
		mock: Mock_Enemy,
	}

	#[automock]
	impl EnemyAttack for _Enemy {
		#[allow(clippy::needless_lifetimes)]
		fn insert_attack<'a>(
			&self,
			entity: &mut EntityCommands<'a>,
			attacker: Attacker,
			target: Target,
		) {
			self.mock.insert_attack(entity, attacker, target);
		}

		fn cool_down(&self) -> Duration {
			self.mock.cool_down()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Enemy::attack);

		app
	}

	#[test]
	fn use_spawn_function() {
		let mut app = setup();
		let target = PersistentEntity::default();
		let attacker = PersistentEntity::default();
		app.world_mut().spawn((
			attacker,
			Attack(target),
			_Enemy::new().with_mock(|mock| {
				mock.expect_insert_attack()
					.times(1)
					.returning(|entity, attacker, target| {
						entity.insert(_FakeAttack { attacker, target });
					});
				mock.expect_cool_down().return_const(Duration::ZERO);
			}),
		));

		app.update();

		assert_eq!(
			Some(&_FakeAttack {
				attacker: Attacker(attacker),
				target: Target(target)
			}),
			app.world()
				.iter_entities()
				.find_map(|e| e.get::<_FakeAttack>())
		);
	}

	#[test]
	fn insert_cool_down() {
		let mut app = setup();
		let attacker = app
			.world_mut()
			.spawn((
				PersistentEntity::default(),
				Attack(PersistentEntity::default()),
				_Enemy::new().with_mock(|mock| {
					mock.expect_insert_attack().return_const(());
					mock.expect_cool_down()
						.return_const(Duration::from_secs(42));
				}),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&OnCoolDown(Duration::from_secs(42))),
			app.world().entity(attacker).get::<OnCoolDown>()
		);
	}

	#[test]
	fn do_nothing_when_cool_down_present() {
		let mut app = setup();
		let attacker = app
			.world_mut()
			.spawn((
				PersistentEntity::default(),
				Attack(PersistentEntity::default()),
				OnCoolDown(Duration::from_millis(100)),
				_Enemy::new().with_mock(|mock| {
					mock.expect_insert_attack().returning(|entity, _, _| {
						entity.insert(_FakeAttackEmpty);
					});
					mock.expect_cool_down()
						.return_const(Duration::from_secs(42));
				}),
			))
			.id();

		app.update();

		assert_eq!(
			(None, Some(&OnCoolDown(Duration::from_millis(100)))),
			(
				app.world()
					.iter_entities()
					.find_map(|e| e.get::<_FakeAttackEmpty>()),
				app.world().entity(attacker).get::<OnCoolDown>()
			)
		);
	}

	#[test]
	fn despawn_attack_entity_when_attack_removed() {
		let mut app = setup();
		let attacker = app
			.world_mut()
			.spawn((
				PersistentEntity::default(),
				Attack(PersistentEntity::default()),
				_Enemy::new().with_mock(|mock| {
					mock.expect_insert_attack().returning(|entity, _, _| {
						entity.insert(_FakeAttackEmpty);
					});
					mock.expect_cool_down().return_const(Duration::ZERO);
				}),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(attacker).remove::<Attack>();
		app.update();

		assert_eq!(
			None,
			app.world()
				.iter_entities()
				.find_map(|e| e.get::<_FakeAttackEmpty>())
		);
	}

	#[test]
	fn remove_ongoing_component_when_attack_removed() {
		let mut app = setup();
		let attacker = app
			.world_mut()
			.spawn((
				PersistentEntity::default(),
				Attack(PersistentEntity::default()),
				_Enemy::new().with_mock(|mock| {
					mock.expect_insert_attack().return_const(());
					mock.expect_cool_down().return_const(Duration::ZERO);
				}),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(attacker).remove::<Attack>();
		app.update();

		assert_eq!(
			None,
			app.world().iter_entities().find_map(|e| e.get::<Ongoing>())
		);
	}
}
