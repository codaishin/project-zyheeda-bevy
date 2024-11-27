use crate::{
	components::{
		blocker::Blockers,
		is::{Fragile, Is},
	},
	events::{Collision, InteractionEvent},
};
use bevy::prelude::*;
use common::{
	components::{destroy::Destroy, ColliderRoot},
	traits::try_insert_on::TryInsertOn,
};

pub(crate) fn apply_fragile_blocks(
	mut commands: Commands,
	mut interaction_event: EventReader<InteractionEvent>,
	fragiles: Query<(Entity, &Is<Fragile>)>,
	blockers: Query<&Blockers>,
) {
	for (a, b) in interaction_event.read().filter_map(collision_started) {
		if let Some(fragile) = fragile_blocked_entity(a, b, &fragiles, &blockers) {
			commands.try_insert_on(fragile, Destroy::Immediately);
		}
		if let Some(fragile) = fragile_blocked_entity(b, a, &fragiles, &blockers) {
			commands.try_insert_on(fragile, Destroy::Immediately);
		}
	}
}

fn collision_started(
	InteractionEvent(ColliderRoot(a), collision): &InteractionEvent,
) -> Option<(&Entity, &Entity)> {
	match collision {
		Collision::Started(ColliderRoot(b)) => Some((a, b)),
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
	use crate::components::blocker::Blocker;
	use common::{components::ColliderRoot, test_tools::utils::SingleThreadedApp};

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
			.spawn(Blockers::new([Blocker::Physical]))
			.id();

		app.update();

		app.world_mut().send_event(
			InteractionEvent::of(ColliderRoot(fragile))
				.collision(Collision::Started(ColliderRoot(blocker))),
		);

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(Some(&Destroy::Immediately), fragile.get::<Destroy>());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_non_matching_blocker() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Is::<Fragile>::interacting_with([Blocker::Physical]))
			.id();
		let blocker = app.world_mut().spawn(Blockers::new([Blocker::Force])).id();

		app.update();

		app.world_mut().send_event(
			InteractionEvent::of(ColliderRoot(fragile))
				.collision(Collision::Started(ColliderRoot(blocker))),
		);

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
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
			.spawn(Blockers::new([Blocker::Physical]))
			.id();

		app.update();

		app.world_mut().send_event(
			InteractionEvent::of(ColliderRoot(blocker))
				.collision(Collision::Started(ColliderRoot(fragile))),
		);

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(Some(&Destroy::Immediately), fragile.get::<Destroy>());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_non_matching_blocker_reversed() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Is::<Fragile>::interacting_with([Blocker::Physical]))
			.id();
		let blocker = app.world_mut().spawn(Blockers::new([Blocker::Force])).id();

		app.update();

		app.world_mut().send_event(
			InteractionEvent::of(ColliderRoot(fragile))
				.collision(Collision::Started(ColliderRoot(blocker))),
		);

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
	}
}
