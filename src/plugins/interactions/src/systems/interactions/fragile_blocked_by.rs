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
	roots: Query<&ColliderRoot>,
) {
	for InteractionEvent(a, b) in collision_events.read() {
		if let Some(entity) = fragile_blocked_entity(a, b, &roots, &fragiles, &blockers) {
			commands.try_insert_on(entity, Destroy::Immediately);
		}
		if let Some(entity) = fragile_blocked_entity(b, a, &roots, &fragiles, &blockers) {
			commands.try_insert_on(entity, Destroy::Immediately);
		}
	}
}

fn fragile_blocked_entity<TBlocker: Component>(
	fragile: &Entity,
	blocker: &Entity,
	roots: &Query<&ColliderRoot>,
	fragiles: &Query<Entity, With<Is<Fragile>>>,
	blockers: &Query<(), With<TBlocker>>,
) -> Option<Entity> {
	let fragile = roots
		.get(*fragile)
		.map(|ColliderRoot(r)| *r)
		.unwrap_or(*fragile);
	let blocker = roots
		.get(*blocker)
		.map(|ColliderRoot(r)| *r)
		.unwrap_or(*blocker);

	if blockers.get(blocker).is_err() {
		return None;
	}
	fragiles.get(fragile).ok()
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
		let fragile_collider = app.world_mut().spawn(ColliderRoot(fragile)).id();
		let blocker = app.world_mut().spawn(_Blocker).id();
		let blocker_collider = app.world_mut().spawn(ColliderRoot(blocker)).id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent(fragile_collider, blocker_collider));

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
		let fragile_collider = app.world_mut().spawn(ColliderRoot(fragile)).id();
		let blocker = app.world_mut().spawn_empty().id();
		let blocker_collider = app.world_mut().spawn(ColliderRoot(blocker)).id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent(fragile_collider, blocker_collider));

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
	}

	#[test]
	fn do_not_destroy_on_collision_when_not_blocked_by_blocker() {
		let mut app = setup();

		let fragile = app.world_mut().spawn(Is::fragile()).id();
		let fragile_collider = app.world_mut().spawn(ColliderRoot(fragile)).id();
		let blocker = app.world_mut().spawn_empty().id();
		let blocker_collider = app.world_mut().spawn(ColliderRoot(blocker)).id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent(fragile_collider, blocker_collider));

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
		let fragile_collider = app.world_mut().spawn(ColliderRoot(fragile)).id();
		let blocker = app.world_mut().spawn(_Blocker).id();
		let blocker_collider = app.world_mut().spawn(ColliderRoot(blocker)).id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent(blocker_collider, fragile_collider));

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
		let fragile_collider = app.world_mut().spawn(ColliderRoot(fragile)).id();
		let blocker = app.world_mut().spawn_empty().id();
		let blocker_collider = app.world_mut().spawn(ColliderRoot(blocker)).id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent(blocker_collider, fragile_collider));

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
	}

	#[test]
	fn do_not_destroy_on_collision_when_not_blocked_by_blocker_reversed() {
		let mut app = setup();

		let fragile = app.world_mut().spawn(Is::fragile()).id();
		let fragile_collider = app.world_mut().spawn(ColliderRoot(fragile)).id();
		let blocker = app.world_mut().spawn_empty().id();
		let blocker_collider = app.world_mut().spawn(ColliderRoot(blocker)).id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent(blocker_collider, fragile_collider));

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
	}

	#[test]
	fn destroy_on_collision_without_collider_root() {
		let mut app = setup();

		let fragile = app
			.world_mut()
			.spawn(Is::fragile().blocked_by::<_Blocker>())
			.id();
		let blocker = app.world_mut().spawn(_Blocker).id();

		app.update();

		app.world_mut()
			.send_event(InteractionEvent(fragile, blocker));

		app.update();

		let fragile = app.world().entity(fragile);

		assert_eq!(Some(&Destroy::Immediately), fragile.get::<Destroy>());
	}
}
