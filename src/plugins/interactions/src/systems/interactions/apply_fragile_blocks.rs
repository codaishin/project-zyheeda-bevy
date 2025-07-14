use crate::{
	components::is::{Fragile, Is},
	events::{Collision, InteractionEvent},
};
use bevy::prelude::*;
use common::{blocker::Blockers, traits::try_despawn::TryDespawn};

pub(crate) fn apply_fragile_blocks(
	mut commands: Commands,
	mut interaction_event: EventReader<InteractionEvent>,
	fragiles: Query<(Entity, &Is<Fragile>)>,
	blockers: Query<&Blockers>,
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
	fragiles: &Query<(Entity, &Is<Fragile>)>,
	blockers: &Query<&Blockers>,
) -> Option<Entity> {
	let blocker = blockers.get(*blocker).ok()?;
	let (entity, Is(fragile)) = fragiles.get(*fragile).ok()?;

	if blocker.0.intersection(&fragile.0).count() == 0 {
		return None;
	}

	Some(entity)
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::blocker::Blocker;
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
			.spawn(Is::<Fragile>::interacting_with([Blocker::Physical]))
			.id();
		let blocker = app
			.world_mut()
			.spawn(Blockers::from([Blocker::Physical]))
			.id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent::of(fragile).collision(Collision::Started(blocker)));

		app.update();

		assert!(app.world().get_entity(fragile).is_err());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_non_matching_blocker() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Is::<Fragile>::interacting_with([Blocker::Physical]))
			.id();
		let blocker = app.world_mut().spawn(Blockers::from([Blocker::Force])).id();

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
			.spawn(Is::<Fragile>::interacting_with([Blocker::Physical]))
			.id();
		let blocker = app
			.world_mut()
			.spawn(Blockers::from([Blocker::Physical]))
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
			.spawn(Is::<Fragile>::interacting_with([Blocker::Physical]))
			.id();
		let blocker = app.world_mut().spawn(Blockers::from([Blocker::Force])).id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent::of(fragile).collision(Collision::Started(blocker)));

		app.update();

		assert!(app.world().get_entity(fragile).is_ok());
	}
}
