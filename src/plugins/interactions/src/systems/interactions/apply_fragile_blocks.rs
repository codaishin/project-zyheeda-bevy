use crate::{
	components::blockable::Blockable,
	events::{Collision, InteractionEvent},
};
use bevy::prelude::*;
use common::{
	components::is_blocker::IsBlocker,
	traits::{handles_interactions::InteractAble::Fragile, try_despawn::TryDespawn},
};

pub(crate) fn apply_fragile_blocks(
	mut commands: Commands,
	mut interaction_event: EventReader<InteractionEvent>,
	fragiles: Query<(Entity, &Blockable)>,
	blockers: Query<&IsBlocker>,
) {
	for (a, b) in interaction_event.read().filter_map(collision_started) {
		if let Some(fragile) = fragile_blocked_entity(a, b, &fragiles, &blockers) {
			commands.try_despawn(fragile);
		}
		if let Some(fragile) = fragile_blocked_entity(b, a, &fragiles, &blockers) {
			commands.try_despawn(fragile);
		}
	}
}

fn collision_started(
	InteractionEvent(a, collision): &InteractionEvent,
) -> Option<(&Entity, &Entity)> {
	match collision {
		Collision::Started(b) => Some((a, b)),
		Collision::Ended(_) => None,
	}
}

fn fragile_blocked_entity(
	fragile: &Entity,
	blocker: &Entity,
	fragiles: &Query<(Entity, &Blockable)>,
	blockers: &Query<&IsBlocker>,
) -> Option<Entity> {
	let IsBlocker(blocker) = blockers.get(*blocker).ok()?;
	let Ok((entity, Blockable(Fragile { destroyed_by }))) = fragiles.get(*fragile) else {
		return None;
	};

	blocker.intersection(destroyed_by).next().map(|_| entity)
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		components::is_blocker::Blocker,
		traits::handles_interactions::{BeamEmitter, InteractAble::Beam},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, apply_fragile_blocks);
		app.add_event::<InteractionEvent>();

		app
	}

	#[test]
	fn destroy_on_collision() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Blockable(Fragile {
				destroyed_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(IsBlocker::from([Blocker::Physical]))
			.id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent::of(fragile).collision(Collision::Started(blocker)));

		app.update();

		assert!(app.world().get_entity(fragile).is_err());
	}

	#[test]
	fn do_not_destroy_on_collision_if_not_fragile() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Blockable(Beam {
				emitter: BeamEmitter {
					mounted_on: default(),
					range: default(),
					insert_beam_model: |_| {},
				},
				blocked_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(IsBlocker::from([Blocker::Physical]))
			.id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent::of(fragile).collision(Collision::Started(blocker)));

		app.update();

		assert!(app.world().get_entity(fragile).is_ok());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_non_matching_blocker() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Blockable(Fragile {
				destroyed_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(IsBlocker::from([Blocker::Force]))
			.id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent::of(fragile).collision(Collision::Started(blocker)));

		app.update();

		assert!(app.world().get_entity(fragile).is_ok());
	}

	#[test]
	fn destroy_on_collision_reversed() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Blockable(Fragile {
				destroyed_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(IsBlocker::from([Blocker::Physical]))
			.id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent::of(blocker).collision(Collision::Started(fragile)));

		app.update();

		assert!(app.world().get_entity(fragile).is_err());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_non_matching_blocker_reversed() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Blockable(Fragile {
				destroyed_by: [Blocker::Physical].into(),
			}))
			.id();
		let blocker = app
			.world_mut()
			.spawn(IsBlocker::from([Blocker::Force]))
			.id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent::of(fragile).collision(Collision::Started(blocker)));

		app.update();

		assert!(app.world().get_entity(fragile).is_ok());
	}
}
