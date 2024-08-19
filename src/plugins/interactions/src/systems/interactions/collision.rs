use crate::{events::InteractionEvent, traits::ActOn};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		event::EventReader,
		system::{Commands, Query},
	},
	prelude::Mut,
};
use common::{components::ColliderRoot, traits::try_remove_from::TryRemoveFrom};

pub(crate) fn collision_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
	mut commands: Commands,
	mut collisions: EventReader<InteractionEvent>,
	mut actors: Query<&mut TActor>,
	mut targets: Query<&mut TTarget>,
) {
	let root_or_entities =
		|InteractionEvent(ColliderRoot(a), ColliderRoot(b)): &InteractionEvent| (*a, *b);

	for (a, b) in collisions.read().map(root_or_entities) {
		handle_collision_interaction(a, b, &mut actors, &mut targets, &mut commands);
	}
}

fn handle_collision_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
	a: Entity,
	b: Entity,
	actors: &mut Query<&mut TActor>,
	targets: &mut Query<&mut TTarget>,
	commands: &mut Commands,
) {
	if let Some((mut actor, mut target)) = get_actor_and_target(a, b, actors, targets) {
		actor.act_on(&mut target);
		commands.try_remove_from::<TActor>(a);
	}
	if let Some((mut actor, mut target)) = get_actor_and_target(b, a, actors, targets) {
		actor.act_on(&mut target);
		commands.try_remove_from::<TActor>(b);
	}
}

fn get_actor_and_target<'a, TActor: Component, TTarget: Component>(
	actor: Entity,
	target: Entity,
	actors: &'a mut Query<&mut TActor>,
	targets: &'a mut Query<&mut TTarget>,
) -> Option<(Mut<'a, TActor>, Mut<'a, TTarget>)> {
	let actor = actors.get_mut(actor).ok()?;
	let target = targets.get_mut(target).ok()?;

	Some((actor, target))
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use common::traits::nested_mock::NestedMock;
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
		app.add_event::<InteractionEvent>();
		app.add_systems(Update, collision_interaction::<_Actor, _Target>);

		app
	}

	#[test]
	fn act_on_target() {
		let mut app = setup();
		let actor = app
			.world_mut()
			.spawn(_Actor::new_mock(|mock| {
				mock.expect_act_on()
					.times(1)
					.with(eq(_Target))
					.return_const(());
			}))
			.id();
		let target = app.world_mut().spawn(_Target).id();

		app.world_mut()
			.send_event(InteractionEvent::of(ColliderRoot(actor)).with(ColliderRoot(target)));
		app.update();
	}

	#[test]
	fn act_on_target_reversed() {
		let mut app = setup();
		let actor = app
			.world_mut()
			.spawn(_Actor::new_mock(|mock| {
				mock.expect_act_on()
					.times(1)
					.with(eq(_Target))
					.return_const(());
			}))
			.id();
		let target = app.world_mut().spawn(_Target).id();

		app.world_mut()
			.send_event(InteractionEvent::of(ColliderRoot(actor)).with(ColliderRoot(target)));
		app.update();
	}

	#[test]
	fn remove_actor() {
		let mut app = setup();
		let actor = app
			.world_mut()
			.spawn(_Actor::new_mock(|mock| {
				mock.expect_act_on().return_const(());
			}))
			.id();
		let target = app.world_mut().spawn(_Target).id();

		app.world_mut()
			.send_event(InteractionEvent::of(ColliderRoot(actor)).with(ColliderRoot(target)));
		app.update();

		let actor = app.world().entity(actor);

		assert!(!actor.contains::<_Actor>());
	}

	#[test]
	fn remove_actor_reversed() {
		let mut app = setup();
		let actor = app
			.world_mut()
			.spawn(_Actor::new_mock(|mock| {
				mock.expect_act_on().return_const(());
			}))
			.id();
		let target = app.world_mut().spawn(_Target).id();

		app.world_mut()
			.send_event(InteractionEvent::of(ColliderRoot(actor)).with(ColliderRoot(target)));
		app.update();

		let actor = app.world().entity(actor);

		assert!(!actor.contains::<_Actor>());
	}
}
