use crate::{
	components::{interacting_entities::InteractingEntities, interactions::Interactions},
	traits::act_on::ActOn,
};
use bevy::{ecs::component::Mutable, prelude::*};
use std::time::Duration;

type Components<'a, TActor, TTarget> = (
	Entity,
	&'a mut TActor,
	&'a mut Interactions<TActor, TTarget>,
	&'a InteractingEntities,
);

impl<T> ActOnSystem for T where T: Component<Mutability = Mutable> + Sized {}

pub(crate) trait ActOnSystem: Component<Mutability = Mutable> + Sized {
	fn act_on<TTarget>(
		In(delta): In<Duration>,
		mut actors: Query<Components<Self, TTarget>>,
		mut targets: Query<(Entity, &mut TTarget)>,
	) where
		Self: ActOn<TTarget>,
		TTarget: Component<Mutability = Mutable>,
	{
		for (entity, mut actor, mut interactions, interacting_entities) in &mut actors {
			for target in interacting_entities.iter() {
				let Ok((target_entity, mut target)) = targets.get_mut(*target) else {
					continue;
				};

				match interactions.insert(target_entity) {
					true => actor.on_begin_interaction(entity, &mut target),
					false => actor.on_repeated_interaction(entity, &mut target, delta),
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
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, NestedMocks)]
	pub struct _Actor {
		mock: Mock_Actor,
	}

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	pub struct _Target;

	impl UpdateBlockers for _Actor {}
	impl UpdateBlockers for Mock_Actor {}

	#[automock]
	impl ActOn<_Target> for _Actor {
		fn on_begin_interaction(&mut self, self_entity: Entity, target: &mut _Target) {
			self.mock.on_begin_interaction(self_entity, target);
		}

		fn on_repeated_interaction(
			&mut self,
			self_entity: Entity,
			target: &mut _Target,
			delta: Duration,
		) {
			self.mock
				.on_repeated_interaction(self_entity, target, delta);
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn begin_interaction() -> Result<(), RunSystemError> {
		let mut app = setup();
		let target = app.world_mut().spawn(_Target).id();
		let entity = app
			.world_mut()
			.spawn((
				Interactions::<_Actor, _Target>::default(),
				InteractingEntities::new([target]),
			))
			.id();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Actor::new().with_mock(|mock| {
				mock.expect_on_begin_interaction()
					.times(1)
					.with(eq(entity), eq(_Target))
					.return_const(());
				mock.expect_on_repeated_interaction().never();
			}));

		app.world_mut()
			.run_system_once_with(_Actor::act_on::<_Target>, Duration::from_millis(42))
	}

	#[test]
	fn track_interacting_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let target = app.world_mut().spawn(_Target).id();
		let entity = app
			.world_mut()
			.spawn((
				Interactions::<_Actor, _Target>::default(),
				InteractingEntities::new([target]),
			))
			.id();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Actor::new().with_mock(|mock| {
				mock.expect_on_begin_interaction().return_const(());
				mock.expect_on_repeated_interaction().return_const(());
			}));

		app.world_mut()
			.run_system_once_with(_Actor::act_on::<_Target>, Duration::from_millis(42))?;

		assert_eq!(
			Some(&Interactions::<_Actor, _Target>::from([target])),
			app.world()
				.entity(entity)
				.get::<Interactions::<_Actor, _Target>>(),
		);
		Ok(())
	}

	#[test]
	fn repeat_interaction() -> Result<(), RunSystemError> {
		let mut app = setup();
		let target = app.world_mut().spawn(_Target).id();
		let entity = app
			.world_mut()
			.spawn((
				Interactions::<_Actor, _Target>::from([target]),
				InteractingEntities::new([target]),
			))
			.id();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Actor::new().with_mock(|mock| {
				mock.expect_on_begin_interaction().never();
				mock.expect_on_repeated_interaction()
					.times(1)
					.with(eq(entity), eq(_Target), eq(Duration::from_millis(42)))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once_with(_Actor::act_on::<_Target>, Duration::from_millis(42))
	}
}
