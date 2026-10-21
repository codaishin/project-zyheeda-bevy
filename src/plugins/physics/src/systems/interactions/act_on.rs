use crate::{
	components::{
		collision_domains::Physical,
		ongoing_effects::OngoingEffects,
		persistent_root::PersistentRoot,
	},
	observers::skill_prefab::GetCaster,
	resources::root_collisions::RootCollisions,
	traits::act_on::ActOn,
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::prelude::*;
use std::time::Duration;

type Components<'a, TActor, TTarget> = (
	Entity,
	&'a PersistentRoot,
	&'a mut TActor,
	&'a mut OngoingEffects<TActor, TTarget>,
);

impl<T> ActOnSystem for T where T: Component<Mutability = Mutable> + Sized {}

pub(crate) trait ActOnSystem: Component<Mutability = Mutable> + Sized {
	fn act_on<TGetCaster, TTarget>(
		In(delta): In<Duration>,
		commands: ZyheedaCommands,
		ongoing_interactions: Res<RootCollisions<Physical>>,
		caster_lookups: Query<&TGetCaster>,
		mut actors: Query<Components<Self, TTarget>>,
		mut targets: Query<(&PersistentEntity, &mut TTarget)>,
	) where
		Self: ActOn<TTarget>,
		TGetCaster: Component + GetCaster,
		TTarget: Component<Mutability = Mutable>,
	{
		for (entity, persistent_root, mut actor, mut ongoing_effects) in &mut actors {
			let interaction_targets = ongoing_interactions.ongoing(&entity);
			let Some(root) = commands.get(persistent_root) else {
				continue;
			};
			let Ok(caster_lookup) = caster_lookups.get(root) else {
				continue;
			};
			let persistent_caster = &caster_lookup.get_caster();

			ongoing_effects.entities.retain(|persistent_target| {
				match commands.get(persistent_target) {
					Some(target) => interaction_targets.contains(&target),
					None => false,
				}
			});

			for target in interaction_targets {
				let Ok((persistent_target_entity, mut target)) = targets.get_mut(*target) else {
					continue;
				};

				if persistent_target_entity == persistent_caster {
					continue;
				}

				match ongoing_effects.entities.insert(*persistent_target_entity) {
					true => actor.on_begin_interaction(**persistent_root, &mut target),
					false => actor.on_repeated_interaction(**persistent_root, &mut target, delta),
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::update_blockers::UpdateBlockers;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::HashSet, sync::LazyLock};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
	pub struct _Actor {
		mock: Mock_Actor,
	}

	static TARGET: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	#[require(PersistentEntity = *TARGET)]
	pub struct _Target;

	impl UpdateBlockers for _Actor {}
	impl UpdateBlockers for Mock_Actor {}

	#[automock]
	impl ActOn<_Target> for _Actor {
		fn on_begin_interaction(&mut self, self_entity: PersistentEntity, target: &mut _Target) {
			self.mock.on_begin_interaction(self_entity, target);
		}

		fn on_repeated_interaction(
			&mut self,
			self_entity: PersistentEntity,
			target: &mut _Target,
			delta: Duration,
		) {
			self.mock
				.on_repeated_interaction(self_entity, target, delta);
		}
	}

	static CASTER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[derive(Component)]
	struct _GetCaster;

	impl GetCaster for _GetCaster {
		fn get_caster(&self) -> PersistentEntity {
			*CASTER
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin::with_asset_loading(false));
		app.init_resource::<RootCollisions<Physical>>();
		app.register_persistent_entities();

		app
	}

	#[test]
	fn begin_interaction() -> Result<(), RunSystemError> {
		let mut app = setup();
		let actor = PersistentEntity::default();
		app.world_mut().spawn((actor, _GetCaster));
		let target = app.world_mut().spawn(_Target).id();
		let entity = app
			.world_mut()
			.spawn(OngoingEffects::<_Actor, _Target>::default())
			.id();
		app.insert_resource(RootCollisions::<Physical>::from_ongoing([(
			entity,
			HashSet::from([target]),
		)]));

		app.world_mut().entity_mut(entity).insert((
			PersistentRoot(actor),
			_Actor::new().with_mock(|mock| {
				mock.expect_on_begin_interaction()
					.times(1)
					.with(eq(actor), eq(_Target))
					.return_const(());
				mock.expect_on_repeated_interaction().never();
			}),
		));

		app.world_mut().run_system_once_with(
			_Actor::act_on::<_GetCaster, _Target>,
			Duration::from_millis(42),
		)
	}

	#[test]
	fn ignore_interaction_with_caster() -> Result<(), RunSystemError> {
		let mut app = setup();
		let actor = PersistentEntity::default();
		app.world_mut().spawn((actor, _GetCaster));
		let target = app.world_mut().spawn((*CASTER, _Target)).id();
		let entity = app
			.world_mut()
			.spawn(OngoingEffects::<_Actor, _Target>::default())
			.id();
		app.insert_resource(RootCollisions::<Physical>::from_ongoing([(
			entity,
			HashSet::from([target]),
		)]));

		app.world_mut().entity_mut(entity).insert((
			PersistentRoot(actor),
			_Actor::new().with_mock(|mock| {
				mock.expect_on_begin_interaction().never().return_const(());
				mock.expect_on_repeated_interaction().never();
			}),
		));

		app.world_mut().run_system_once_with(
			_Actor::act_on::<_GetCaster, _Target>,
			Duration::from_millis(42),
		)
	}

	#[test]
	fn track_interacting_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let actor = PersistentEntity::default();
		app.world_mut().spawn((actor, _GetCaster));
		let target = app.world_mut().spawn(_Target).id();
		let entity = app
			.world_mut()
			.spawn(OngoingEffects::<_Actor, _Target>::default())
			.id();
		app.insert_resource(RootCollisions::<Physical>::from_ongoing([(
			entity,
			HashSet::from([target]),
		)]));
		app.world_mut().entity_mut(entity).insert((
			PersistentRoot(actor),
			_Actor::new().with_mock(|mock| {
				mock.expect_on_begin_interaction().return_const(());
				mock.expect_on_repeated_interaction().return_const(());
			}),
		));

		app.world_mut().run_system_once_with(
			_Actor::act_on::<_GetCaster, _Target>,
			Duration::from_millis(42),
		)?;

		assert_eq!(
			Some(&OngoingEffects::<_Actor, _Target>::from([*TARGET])),
			app.world()
				.entity(entity)
				.get::<OngoingEffects::<_Actor, _Target>>(),
		);
		Ok(())
	}

	#[test]
	fn repeat_interaction() -> Result<(), RunSystemError> {
		let mut app = setup();
		let actor = PersistentEntity::default();
		app.world_mut().spawn((actor, _GetCaster));
		let target = app.world_mut().spawn(_Target).id();
		let entity = app
			.world_mut()
			.spawn(OngoingEffects::<_Actor, _Target>::from([*TARGET]))
			.id();
		app.insert_resource(RootCollisions::<Physical>::from_ongoing([(
			entity,
			HashSet::from([target]),
		)]));
		app.world_mut().entity_mut(entity).insert((
			PersistentRoot(actor),
			_Actor::new().with_mock(|mock| {
				mock.expect_on_begin_interaction().never();
				mock.expect_on_repeated_interaction()
					.times(1)
					.with(eq(actor), eq(_Target), eq(Duration::from_millis(42)))
					.return_const(());
			}),
		));

		app.world_mut().run_system_once_with(
			_Actor::act_on::<_GetCaster, _Target>,
			Duration::from_millis(42),
		)
	}

	mod ongoing_effects {
		use super::*;

		#[test]
		fn remove_entities_not_contained_in_ongoing_interactions() -> Result<(), RunSystemError> {
			let mut app = setup();
			let actor = PersistentEntity::default();
			app.world_mut().spawn((actor, _GetCaster));
			let not_interacting = PersistentEntity::default();
			let interacting = [PersistentEntity::default(), PersistentEntity::default()];
			let interacting_entities = interacting.map(|e| app.world_mut().spawn(e).id());
			let entity = app
				.world_mut()
				.spawn((
					OngoingEffects::<_Actor, _Target>::from([
						interacting[0],
						interacting[1],
						not_interacting,
					]),
					PersistentRoot(actor),
					_Actor::new().with_mock(|mock| {
						mock.expect_on_begin_interaction().return_const(());
						mock.expect_on_repeated_interaction().return_const(());
					}),
				))
				.id();
			app.world_mut().spawn(not_interacting);
			app.insert_resource(RootCollisions::<Physical>::from_ongoing([(
				entity,
				HashSet::from(interacting_entities),
			)]));

			app.world_mut().run_system_once_with(
				_Actor::act_on::<_GetCaster, _Target>,
				Duration::from_millis(42),
			)?;

			assert_eq!(
				Some(&OngoingEffects::<_Actor, _Target>::from(interacting)),
				app.world()
					.entity(entity)
					.get::<OngoingEffects<_Actor, _Target>>(),
			);
			Ok(())
		}

		#[test]
		fn remove_entities_not_contained_when_ongoing_interactions_has_actor_entry()
		-> Result<(), RunSystemError> {
			let mut app = setup();
			let actor = PersistentEntity::default();
			app.world_mut().spawn((actor, _GetCaster));
			let not_interacting = PersistentEntity::default();
			let entity = app
				.world_mut()
				.spawn((
					OngoingEffects::<_Actor, _Target>::from([not_interacting]),
					PersistentRoot(actor),
					_Actor::new().with_mock(|mock| {
						mock.expect_on_begin_interaction().return_const(());
						mock.expect_on_repeated_interaction().return_const(());
					}),
				))
				.id();
			app.world_mut().spawn(not_interacting);

			app.world_mut().run_system_once_with(
				_Actor::act_on::<_GetCaster, _Target>,
				Duration::from_millis(42),
			)?;

			assert_eq!(
				Some(&OngoingEffects::<_Actor, _Target>::from([])),
				app.world()
					.entity(entity)
					.get::<OngoingEffects<_Actor, _Target>>(),
			);
			Ok(())
		}

		#[test]
		fn remove_entities_that_does_not_exist() -> Result<(), RunSystemError> {
			let mut app = setup();
			let actor = PersistentEntity::default();
			app.world_mut().spawn((actor, _GetCaster));
			let not_interacting = PersistentEntity::default();
			let interacting = [PersistentEntity::default(), PersistentEntity::default()];
			let interacting_entities = interacting.map(|e| app.world_mut().spawn(e).id());
			let entity = app
				.world_mut()
				.spawn((
					OngoingEffects::<_Actor, _Target>::from([
						interacting[0],
						interacting[1],
						not_interacting,
					]),
					PersistentRoot(actor),
					_Actor::new().with_mock(|mock| {
						mock.expect_on_begin_interaction().return_const(());
						mock.expect_on_repeated_interaction().return_const(());
					}),
				))
				.id();
			app.insert_resource(RootCollisions::<Physical>::from_ongoing([(
				entity,
				HashSet::from(interacting_entities),
			)]));

			app.world_mut().run_system_once_with(
				_Actor::act_on::<_GetCaster, _Target>,
				Duration::from_millis(42),
			)?;

			assert_eq!(
				Some(&OngoingEffects::<_Actor, _Target>::from(interacting)),
				app.world()
					.entity(entity)
					.get::<OngoingEffects<_Actor, _Target>>(),
			);
			Ok(())
		}
	}
}
