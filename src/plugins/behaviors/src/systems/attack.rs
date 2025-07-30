use crate::components::{Attack, on_cool_down::OnCoolDown};
use bevy::prelude::*;
use common::{
	components::{ground_offset::GroundOffset, persistent_entity::PersistentEntity},
	traits::{
		handles_enemies::{Attacker, EnemyAttack, Target},
		try_despawn::TryDespawn,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

impl<T> AttackSystem for T {}

type AttackerComponents<'a, TAttacker> = (
	Entity,
	&'a PersistentEntity,
	&'a TAttacker,
	&'a Attack,
	Option<&'a GroundOffset>,
);

pub trait AttackSystem {
	fn attack(
		mut commands: Commands,
		mut stop_attacks: RemovedComponents<Attack>,
		attackers: Query<AttackerComponents<Self>, Without<OnCoolDown>>,
		ongoing: Query<&Ongoing>,
	) where
		Self: Component + EnemyAttack + Sized,
	{
		for (entity, persistent_entity, enemy, Attack(target), offset) in &attackers {
			let offset = match offset {
				Some(GroundOffset(offset)) => *offset,
				None => Vec3::ZERO,
			};
			let mut attack_entity =
				commands.spawn((ChildOf(entity), Transform::from_translation(offset)));
			let attack = attack_entity.id();

			enemy.insert_attack(
				&mut attack_entity,
				Attacker(*persistent_entity),
				Target(*target),
			);
			commands.try_insert_on(entity, (OnCoolDown(enemy.cool_down()), Ongoing(attack)));
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
	use common::traits::handles_enemies::EnemyAttack;
	use macros::NestedMocks;
	use mockall::automock;
	use std::time::Duration;
	use testing::{NestedMocks, SingleThreadedApp, assert_count, get_children};

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
		let entity = app
			.world_mut()
			.spawn((
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
			))
			.id();

		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			(
				Some(&_FakeAttack {
					attacker: Attacker(attacker),
					target: Target(target)
				}),
				Some(&Transform::default())
			),
			(child.get::<_FakeAttack>(), child.get::<Transform>(),)
		);
	}

	#[test]
	fn spawn_with_offset() {
		let mut app = setup();
		let target = PersistentEntity::default();
		let attacker = PersistentEntity::default();
		let entity = app
			.world_mut()
			.spawn((
				attacker,
				Attack(target),
				GroundOffset(Vec3::new(1., 2., 3.)),
				_Enemy::new().with_mock(|mock| {
					mock.expect_insert_attack()
						.times(1)
						.returning(|entity, attacker, target| {
							entity.insert(_FakeAttack { attacker, target });
						});
					mock.expect_cool_down().return_const(Duration::ZERO);
				}),
			))
			.id();

		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			(
				Some(&_FakeAttack {
					attacker: Attacker(attacker),
					target: Target(target)
				}),
				Some(&Transform::from_xyz(1., 2., 3.))
			),
			(child.get::<_FakeAttack>(), child.get::<Transform>(),)
		);
	}

	#[test]
	fn insert_cool_down() {
		let mut app = setup();
		let entity = app
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
			app.world().entity(entity).get::<OnCoolDown>()
		);
	}

	#[test]
	fn do_nothing_when_cool_down_present() {
		let mut app = setup();
		let entity = app
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
				app.world().entity(entity).get::<OnCoolDown>()
			)
		);
	}

	#[test]
	fn despawn_attack_entity_when_attack_removed() {
		let mut app = setup();
		let entity = app
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
		app.world_mut().entity_mut(entity).remove::<Attack>();
		app.update();

		assert_count!(0, get_children!(app, entity));
	}

	#[test]
	fn remove_ongoing_component_when_attack_removed() {
		let mut app = setup();
		let entity = app
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
		app.world_mut().entity_mut(entity).remove::<Attack>();
		app.update();

		assert_count!(0, get_children!(app, entity));
	}
}
