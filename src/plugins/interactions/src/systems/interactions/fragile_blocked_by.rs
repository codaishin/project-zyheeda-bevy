use crate::{
	components::{
		is::{Fragile, Is},
		Destroy,
	},
	events::InteractionEvent,
};
use bevy::{
	ecs::{
		entity::Entity,
		event::EventReader,
		query::With,
		system::{Commands, Query},
	},
	prelude::Component,
};
use common::{components::ColliderRoot, traits::try_insert_on::TryInsertOn};

pub(crate) fn fragile_blocked_by<TBlocker: Component>(
	mut commands: Commands,
	mut collision_events: EventReader<InteractionEvent>,
	fragiles: Query<Entity, With<Is<Fragile>>>,
	blockers: Query<(), With<TBlocker>>,
) {
	for InteractionEvent(ColliderRoot(a), ColliderRoot(b)) in collision_events.read() {
		if let Some(entity) = fragile_blocked_entity(a, b, &fragiles, &blockers) {
			commands.try_insert_on(entity, Destroy::Immediately);
		}
		if let Some(entity) = fragile_blocked_entity(b, a, &fragiles, &blockers) {
			commands.try_insert_on(entity, Destroy::Immediately);
		}
	}
}

fn fragile_blocked_entity<TBlocker: Component>(
	fragile: &Entity,
	blocker: &Entity,
	fragiles: &Query<Entity, With<Is<Fragile>>>,
	blockers: &Query<(), With<TBlocker>>,
) -> Option<Entity> {
	if blockers.get(*blocker).is_err() {
		return None;
	}
	fragiles.get(*fragile).ok()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Destroy;
	use bevy::{
		app::{App, Update},
		prelude::Component,
	};
	use common::{components::ColliderRoot, test_tools::utils::SingleThreadedApp};

	#[derive(Component)]
	struct _Blocker;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, fragile_blocked_by::<_Blocker>);
		app.add_event::<InteractionEvent>();

		app
	}

	#[test]
	fn destroy_on_collision() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Is::fragile().blocked_by::<_Blocker>())
			.id();
		let blocker = app.world_mut().spawn(_Blocker).id();

		app.update();

		app.world_mut().send_event(InteractionEvent(
			ColliderRoot(fragile),
			ColliderRoot(blocker),
		));

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(Some(&Destroy::Immediately), fragile.get::<Destroy>());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_not_blocker() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Is::fragile().blocked_by::<_Blocker>())
			.id();
		let blocker = app.world_mut().spawn_empty().id();

		app.update();

		app.world_mut().send_event(InteractionEvent(
			ColliderRoot(fragile),
			ColliderRoot(blocker),
		));

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
	}

	#[test]
	fn do_not_destroy_on_collision_when_not_blocked_by_blocker() {
		let mut app = setup();

		let fragile = app.world_mut().spawn(Is::fragile()).id();
		let blocker = app.world_mut().spawn_empty().id();

		app.update();

		app.world_mut().send_event(InteractionEvent(
			ColliderRoot(fragile),
			ColliderRoot(blocker),
		));

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
	}

	#[test]
	fn destroy_on_collision_reversed() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Is::fragile().blocked_by::<_Blocker>())
			.id();
		let blocker = app.world_mut().spawn(_Blocker).id();

		app.update();

		app.world_mut().send_event(InteractionEvent(
			ColliderRoot(blocker),
			ColliderRoot(fragile),
		));

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(Some(&Destroy::Immediately), fragile.get::<Destroy>());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_no_blocker_reversed() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Is::fragile().blocked_by::<_Blocker>())
			.id();
		let blocker = app.world_mut().spawn_empty().id();

		app.update();

		app.world_mut().send_event(InteractionEvent(
			ColliderRoot(blocker),
			ColliderRoot(fragile),
		));

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
	}

	#[test]
	fn do_not_destroy_on_collision_when_not_blocked_by_blocker_reversed() {
		let mut app = setup();

		let fragile = app.world_mut().spawn(Is::fragile()).id();
		let blocker = app.world_mut().spawn_empty().id();

		app.update();

		app.world_mut().send_event(InteractionEvent(
			ColliderRoot(blocker),
			ColliderRoot(fragile),
		));

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
	}
}
