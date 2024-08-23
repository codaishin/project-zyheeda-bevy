use crate::{components::interacting_entities::InteractingEntities, traits::ActOn};
use bevy::ecs::{
	component::Component,
	entity::Entity,
	system::{Commands, Query},
};
use common::traits::try_remove_from::TryRemoveFrom;

pub(crate) fn act_on_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
	mut commands: Commands,
	mut actors: Query<(Entity, &mut TActor, &InteractingEntities)>,
	mut targets: Query<&mut TTarget>,
) {
	for (entity, mut actor, interactions) in &mut actors {
		for target in interactions.iter() {
			let Ok(mut target) = targets.get_mut(*target) else {
				continue;
			};

			actor.act_on(&mut target);
			commands.try_remove_from::<TActor>(entity);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use common::{components::ColliderRoot, traits::nested_mock::NestedMock};
	use macros::NestedMock;
	use mockall::{automock, predicate::eq};

	#[derive(Component, NestedMock)]
	pub struct _Actor {
		mock: Mock_Actor,
	}

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	pub struct _Target;

	#[automock]
	impl ActOn<_Target> for _Actor {
		fn act_on(&mut self, target: &mut _Target) {
			self.mock.act_on(target)
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, act_on_interaction::<_Actor, _Target>);

		app
	}

	#[test]
	fn act_on_target() {
		let mut app = setup();
		let target = app.world_mut().spawn(_Target).id();
		app.world_mut().spawn((
			InteractingEntities::new([ColliderRoot(target)]),
			_Actor::new_mock(|mock| {
				mock.expect_act_on()
					.times(1)
					.with(eq(_Target))
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn remove_actor() {
		let mut app = setup();
		let target = app.world_mut().spawn(_Target).id();
		let actor = app
			.world_mut()
			.spawn((
				InteractingEntities::new([ColliderRoot(target)]),
				_Actor::new_mock(|mock| {
					mock.expect_act_on().return_const(());
				}),
			))
			.id();

		app.update();

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
				InteractingEntities::new([ColliderRoot(target)]),
				_Actor::new_mock(|mock| {
					mock.expect_act_on().return_const(());
				}),
			))
			.id();

		app.update();

		let actor = app.world().entity(actor);

		assert!(actor.contains::<_Actor>());
	}
}
