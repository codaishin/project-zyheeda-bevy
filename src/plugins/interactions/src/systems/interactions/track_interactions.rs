use crate::{
	components::{InteractingEntities, RemainingSubCollisions, SubCollisions},
	events::{Collision, InteractionEvent},
};
use bevy::prelude::{Entity, EventReader, Query};
use common::components::ColliderRoot;
use std::collections::hash_map::Entry;

pub(crate) fn track_interactions(
	mut interaction_events: EventReader<InteractionEvent>,
	mut agents: Query<&mut InteractingEntities>,
) {
	for InteractionEvent(ColliderRoot(a), collision) in interaction_events.read() {
		match collision {
			Collision::Started(ColliderRoot(b)) => {
				track_started(&mut agents, *a, *b);
				track_started(&mut agents, *b, *a);
			}
			Collision::Ended(ColliderRoot(b)) => {
				track_ended(&mut agents, *a, *b);
				track_ended(&mut agents, *b, *a);
			}
		}
	}
}

fn track_started(agents: &mut Query<&mut InteractingEntities>, agent: Entity, other: Entity) {
	let Ok(mut agent) = agents.get_mut(agent) else {
		return;
	};

	match agent.entry(ColliderRoot(other)) {
		Entry::Occupied(mut entry) => entry.get_mut().increment(),
		Entry::Vacant(entry) => entry.insert(SubCollisions::one()),
	};
}

fn track_ended(agents: &mut Query<&mut InteractingEntities>, agent: Entity, other: Entity) {
	let Ok(mut agent) = agents.get_mut(agent) else {
		return;
	};
	let Entry::Occupied(mut entry) = agent.entry(ColliderRoot(other)) else {
		return;
	};
	let sub_collisions = entry.get_mut();

	match sub_collisions.try_decrement() {
		RemainingSubCollisions::Some(remaining) => {
			*sub_collisions = remaining;
		}
		RemainingSubCollisions::None => {
			entry.remove();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::Collision;
	use bevy::app::{App, Update};
	use common::{components::ColliderRoot, test_tools::utils::SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_event::<InteractionEvent>();
		app.add_systems(Update, track_interactions);

		app
	}

	#[test]
	fn track_started_interactions() {
		let mut app = setup();
		let a = app.world_mut().spawn(InteractingEntities::default()).id();
		let b = app.world_mut().spawn(InteractingEntities::default()).id();

		app.world_mut().send_event(
			InteractionEvent::of(ColliderRoot(a)).collision(Collision::Started(ColliderRoot(b))),
		);
		app.update();

		assert_eq!(
			[
				Some(&InteractingEntities::new([(
					ColliderRoot(b),
					SubCollisions::one()
				)])),
				Some(&InteractingEntities::new([(
					ColliderRoot(a),
					SubCollisions::one()
				)]))
			],
			[
				app.world().entity(a).get::<InteractingEntities>(),
				app.world().entity(b).get::<InteractingEntities>()
			]
		)
	}

	#[test]
	fn increase_sub_collision_count_on_started_interactions() {
		let mut app = setup();
		let a = app.world_mut().spawn_empty().id();
		let b = app.world_mut().spawn_empty().id();
		app.world_mut()
			.entity_mut(a)
			.insert(InteractingEntities::new([(
				ColliderRoot(b),
				SubCollisions::one(),
			)]));
		app.world_mut()
			.entity_mut(b)
			.insert(InteractingEntities::new([(
				ColliderRoot(a),
				SubCollisions::one(),
			)]));

		app.world_mut().send_event(
			InteractionEvent::of(ColliderRoot(a)).collision(Collision::Started(ColliderRoot(b))),
		);
		app.update();

		assert_eq!(
			[
				Some(&InteractingEntities::new([(
					ColliderRoot(b),
					*SubCollisions::one().increment(),
				)])),
				Some(&InteractingEntities::new([(
					ColliderRoot(a),
					*SubCollisions::one().increment(),
				)]))
			],
			[
				app.world().entity(a).get::<InteractingEntities>(),
				app.world().entity(b).get::<InteractingEntities>()
			]
		)
	}

	#[test]
	fn track_ended_interactions() {
		let mut app = setup();
		let a = app.world_mut().spawn_empty().id();
		let b = app.world_mut().spawn_empty().id();
		app.world_mut()
			.entity_mut(a)
			.insert(InteractingEntities::new([(
				ColliderRoot(b),
				SubCollisions::one(),
			)]));
		app.world_mut()
			.entity_mut(b)
			.insert(InteractingEntities::new([(
				ColliderRoot(a),
				SubCollisions::one(),
			)]));

		app.world_mut().send_event(
			InteractionEvent::of(ColliderRoot(a)).collision(Collision::Ended(ColliderRoot(b))),
		);
		app.update();

		assert_eq!(
			[
				Some(&InteractingEntities::default()),
				Some(&InteractingEntities::default())
			],
			[
				app.world().entity(a).get::<InteractingEntities>(),
				app.world().entity(b).get::<InteractingEntities>()
			]
		)
	}

	#[test]
	fn decrease_sub_collision_count_on_ended_interactions() {
		let mut app = setup();
		let a = app.world_mut().spawn_empty().id();
		let b = app.world_mut().spawn_empty().id();
		app.world_mut()
			.entity_mut(a)
			.insert(InteractingEntities::new([(
				ColliderRoot(b),
				*SubCollisions::one().increment(),
			)]));
		app.world_mut()
			.entity_mut(b)
			.insert(InteractingEntities::new([(
				ColliderRoot(a),
				*SubCollisions::one().increment(),
			)]));

		app.world_mut().send_event(
			InteractionEvent::of(ColliderRoot(a)).collision(Collision::Ended(ColliderRoot(b))),
		);
		app.update();

		assert_eq!(
			[
				Some(&InteractingEntities::new([(
					ColliderRoot(b),
					SubCollisions::one()
				)])),
				Some(&InteractingEntities::new([(
					ColliderRoot(a),
					SubCollisions::one()
				)]))
			],
			[
				app.world().entity(a).get::<InteractingEntities>(),
				app.world().entity(b).get::<InteractingEntities>()
			]
		)
	}
}
