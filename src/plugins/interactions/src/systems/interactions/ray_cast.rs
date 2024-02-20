use crate::{events::RayCastEvent, traits::ActOn};
use bevy::ecs::{component::Component, event::EventReader, system::Query, world::Mut};
use common::components::ColliderRoot;

pub(crate) fn ray_cast_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
	mut ray_casts: EventReader<RayCastEvent>,
	roots: Query<&ColliderRoot>,
	mut actors: Query<&mut TActor>,
	mut targets: Query<&mut TTarget>,
) {
	let target_root_entity = |event: &RayCastEvent| {
		let target_root = roots.get(event.target).ok()?;
		Some(RayCastEvent {
			source: event.source,
			target: target_root.0,
		})
	};

	for event in ray_casts.read().filter_map(target_root_entity) {
		handle_collision_interaction(event, &mut actors, &mut targets);
	}
}

fn handle_collision_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
	event: RayCastEvent,
	actors: &mut Query<&mut TActor>,
	targets: &mut Query<&mut TTarget>,
) {
	let Some((mut actor, mut target)) = get_actor_and_target(event, actors, targets) else {
		return;
	};
	actor.act_on(&mut target);
}

fn get_actor_and_target<'a, TActor: Component, TTarget: Component>(
	event: RayCastEvent,
	actors: &'a mut Query<&mut TActor>,
	targets: &'a mut Query<&mut TTarget>,
) -> Option<(Mut<'a, TActor>, Mut<'a, TTarget>)> {
	let actor = actors.get_mut(event.source).ok()?;
	let target = targets.get_mut(event.target).ok()?;

	Some((actor, target))
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
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
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, ray_cast_interaction::<_Actor, _Target>);

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
		let coll_target = app.world.spawn(ColliderRoot(target)).id();

		app.world.send_event(RayCastEvent {
			source: actor,
			target: coll_target,
		});

		app.update();
	}
}
