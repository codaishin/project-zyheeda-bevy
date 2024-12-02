use crate::{
	components::{acted_on_targets::ActedOnTargets, interacting_entities::InteractingEntities},
	traits::act_on::ActOn,
};
use bevy::prelude::*;
use common::{effects::EffectApplies, traits::try_remove_from::TryRemoveFrom};
use std::time::Duration;

type Components<'a, TActor> = (
	Entity,
	&'a mut TActor,
	&'a mut ActedOnTargets<TActor>,
	&'a InteractingEntities,
);

pub(crate) fn act_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
	In(delta): In<Duration>,
	mut commands: Commands,
	mut actors: Query<Components<TActor>>,
	mut targets: Query<(Entity, &mut TTarget)>,
) {
	for (entity, mut actor, mut acted_on, interactions) in &mut actors {
		for target in interactions.iter() {
			let Ok((target_entity, mut target)) = targets.get_mut(*target) else {
				continue;
			};

			match actor.act(entity, &mut target, delta) {
				EffectApplies::Once => {
					commands.try_remove_from::<TActor>(entity);
				}
				EffectApplies::OncePerTarget => {
					acted_on.entities.insert(target_entity);
				}
				EffectApplies::Always => {}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;
	use common::{components::ColliderRoot, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, NestedMocks)]
	pub struct _Actor {
		mock: Mock_Actor,
	}

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	pub struct _Target;

	#[automock]
	impl ActOn<_Target> for _Actor {
		fn act(
			&mut self,
			self_entity: Entity,
			target: &mut _Target,
			delta: Duration,
		) -> EffectApplies {
			self.mock.act(self_entity, target, delta)
		}
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn act_on_target() {
		let mut app = setup();
		let target = app.world_mut().spawn(_Target).id();
		let entity = app
			.world_mut()
			.spawn((
				ActedOnTargets::<_Actor>::default(),
				InteractingEntities::new([ColliderRoot(target)]),
			))
			.id();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Actor::new().with_mock(|mock| {
				mock.expect_act()
					.times(1)
					.with(eq(entity), eq(_Target), eq(Duration::from_millis(42)))
					.return_const(EffectApplies::Once);
			}));

		app.world_mut().run_system_once_with(
			Duration::from_millis(42),
			act_interaction::<_Actor, _Target>,
		);
	}

	#[test]
	fn remove_actor() {
		let mut app = setup();
		let target = app.world_mut().spawn(_Target).id();
		let actor = app
			.world_mut()
			.spawn((
				ActedOnTargets::<_Actor>::default(),
				InteractingEntities::new([ColliderRoot(target)]),
				_Actor::new().with_mock(|mock| {
					mock.expect_act().return_const(EffectApplies::Once);
				}),
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::ZERO, act_interaction::<_Actor, _Target>);

		let actor = app.world().entity(actor);

		assert!(!actor.contains::<_Actor>());
	}

	#[test]
	fn do_not_remove_actor_when_not_acted() {
		let mut app = setup();
		let target = app.world_mut().spawn_empty().id();
		let actor = app
			.world_mut()
			.spawn((
				ActedOnTargets::<_Actor>::default(),
				InteractingEntities::new([ColliderRoot(target)]),
				_Actor::new().with_mock(|mock| {
					mock.expect_act().return_const(EffectApplies::Once);
				}),
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::ZERO, act_interaction::<_Actor, _Target>);

		let actor = app.world().entity(actor);

		assert!(actor.contains::<_Actor>());
	}

	#[test]
	fn do_not_remove_actor_when_action_type_not_once() {
		let mut app = setup();
		let target = app.world_mut().spawn(_Target).id();
		let actor_always = app
			.world_mut()
			.spawn((
				ActedOnTargets::<_Actor>::default(),
				InteractingEntities::new([ColliderRoot(target)]),
				_Actor::new().with_mock(|mock| {
					mock.expect_act().return_const(EffectApplies::Always);
				}),
			))
			.id();
		let actor_once_per_target = app
			.world_mut()
			.spawn((
				ActedOnTargets::<_Actor>::default(),
				InteractingEntities::new([ColliderRoot(target)]),
				_Actor::new().with_mock(|mock| {
					mock.expect_act().return_const(EffectApplies::OncePerTarget);
				}),
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::ZERO, act_interaction::<_Actor, _Target>);

		let actor_always = app.world().entity(actor_always);
		let actor_once_per_target = app.world().entity(actor_once_per_target);

		assert_eq!(
			(true, true),
			(
				actor_always.contains::<_Actor>(),
				actor_once_per_target.contains::<_Actor>()
			)
		);
	}

	#[test]
	fn add_to_acted_on_when_action_type_is_once_per_target() {
		let mut app = setup();
		let target = app.world_mut().spawn(_Target).id();
		let actor = app
			.world_mut()
			.spawn((
				ActedOnTargets::<_Actor>::default(),
				InteractingEntities::new([ColliderRoot(target)]),
				_Actor::new().with_mock(|mock| {
					mock.expect_act().return_const(EffectApplies::OncePerTarget);
				}),
			))
			.id();

		app.world_mut()
			.run_system_once_with(Duration::ZERO, act_interaction::<_Actor, _Target>);

		let actor = app.world().entity(actor);

		assert_eq!(
			Some(&ActedOnTargets::new([target])),
			actor.get::<ActedOnTargets<_Actor>>()
		);
	}
}
