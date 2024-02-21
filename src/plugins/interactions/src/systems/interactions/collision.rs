use crate::traits::ActOn;
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		event::EventReader,
		system::{Commands, Query},
	},
	prelude::Mut,
};
use bevy_rapier3d::pipeline::CollisionEvent;
use common::components::ColliderRoot;

pub(crate) fn collision_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
	mut commands: Commands,
	mut collisions: EventReader<CollisionEvent>,
	mut actors: Query<&mut TActor>,
	mut targets: Query<&mut TTarget>,
	roots: Query<&ColliderRoot>,
) {
	let root_entities = |event: &CollisionEvent| match event {
		CollisionEvent::Started(a, b, _) => get_roots(*a, *b, &roots),
		_ => None,
	};

	for (a, b) in collisions.read().filter_map(root_entities) {
		handle_collision_interaction(a, b, &mut actors, &mut targets, &mut commands);
	}
}

fn get_roots(a: Entity, b: Entity, roots: &Query<&ColliderRoot>) -> Option<(Entity, Entity)> {
	let root_a = roots.get(a).ok()?;
	let root_b = roots.get(b).ok()?;

	Some((root_a.0, root_b.0))
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
		commands.entity(a).remove::<TActor>();
	}
	if let Some((mut actor, mut target)) = get_actor_and_target(b, a, actors, targets) {
		actor.act_on(&mut target);
		commands.entity(b).remove::<TActor>();
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
	use bevy_rapier3d::{pipeline::CollisionEvent, rapier::geometry::CollisionEventFlags};
	use mockall::{automock, predicate::eq};

	#[derive(Component, Default)]
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
		app.add_event::<CollisionEvent>();
		app.add_systems(Update, collision_interaction::<_Actor, _Target>);

		app
	}

	#[test]
	fn act_on_target() {
		let mut app = setup();
		let mut actor = _Actor::default();
		let target = _Target;
		actor
			.mock
			.expect_act_on()
			.times(1)
			.with(eq(target))
			.return_const(());

		let actor = app.world.spawn(actor).id();
		let target = app.world.spawn(target).id();
		let coll_actor = app.world.spawn(ColliderRoot(actor)).id();
		let coll_target = app.world.spawn(ColliderRoot(target)).id();

		app.world.send_event(CollisionEvent::Started(
			coll_actor,
			coll_target,
			CollisionEventFlags::empty(),
		));

		app.update();
	}

	#[test]
	fn act_on_target_reversed() {
		let mut app = setup();
		let mut actor = _Actor::default();
		let target = _Target;
		actor
			.mock
			.expect_act_on()
			.times(1)
			.with(eq(target))
			.return_const(());

		let actor = app.world.spawn(actor).id();
		let target = app.world.spawn(target).id();
		let coll_actor = app.world.spawn(ColliderRoot(actor)).id();
		let coll_target = app.world.spawn(ColliderRoot(target)).id();

		app.world.send_event(CollisionEvent::Started(
			coll_target,
			coll_actor,
			CollisionEventFlags::empty(),
		));

		app.update();
	}

	#[test]
	fn remove_actor() {
		let mut app = setup();
		let mut actor = _Actor::default();
		let target = _Target;
		actor.mock.expect_act_on().return_const(());

		let actor = app.world.spawn(actor).id();
		let target = app.world.spawn(target).id();
		let coll_actor = app.world.spawn(ColliderRoot(actor)).id();
		let coll_target = app.world.spawn(ColliderRoot(target)).id();

		app.world.send_event(CollisionEvent::Started(
			coll_actor,
			coll_target,
			CollisionEventFlags::empty(),
		));

		app.update();

		let actor = app.world.entity(actor);

		assert!(!actor.contains::<_Actor>());
	}

	#[test]
	fn remove_actor_reversed() {
		let mut app = setup();
		let mut actor = _Actor::default();
		let target = _Target;
		actor.mock.expect_act_on().return_const(());

		let actor = app.world.spawn(actor).id();
		let target = app.world.spawn(target).id();
		let coll_actor = app.world.spawn(ColliderRoot(actor)).id();
		let coll_target = app.world.spawn(ColliderRoot(target)).id();

		app.world.send_event(CollisionEvent::Started(
			coll_target,
			coll_actor,
			CollisionEventFlags::empty(),
		));

		app.update();

		let actor = app.world.entity(actor);

		assert!(!actor.contains::<_Actor>());
	}
}
