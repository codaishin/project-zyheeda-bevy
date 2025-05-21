use crate::components::{
	acted_on_targets::ActedOnTargets,
	interacting_entities::InteractingEntities,
};
use bevy::prelude::{Changed, Component, Query};

pub(crate) fn untrack_non_interacting_targets<TActor: Component>(
	mut agents: Query<
		(&InteractingEntities, &mut ActedOnTargets<TActor>),
		Changed<InteractingEntities>,
	>,
) {
	for (interacting, mut acted_on) in &mut agents {
		let ActedOnTargets { entities, .. } = acted_on.as_mut();
		entities.retain(|e| interacting.contains(e));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		acted_on_targets::ActedOnTargets,
		interacting_entities::InteractingEntities,
	};
	use bevy::{
		app::{App, Update},
		prelude::{Component, Entity},
	};
	use common::{
		components::collider_relations::ChildColliderOf,
		test_tools::utils::SingleThreadedApp,
	};
	use std::ops::DerefMut;

	#[derive(Component)]
	struct _Actor;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, untrack_non_interacting_targets::<_Actor>);

		app
	}

	#[test]
	fn remove_entities_not_contained_in_interacting_entities() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				InteractingEntities::new([
					ChildColliderOf(Entity::from_raw(100)),
					ChildColliderOf(Entity::from_raw(300)),
				]),
				ActedOnTargets::<_Actor>::new([
					Entity::from_raw(100),
					Entity::from_raw(200),
					Entity::from_raw(300),
				]),
			))
			.id();

		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&ActedOnTargets::<_Actor>::new([
				Entity::from_raw(100),
				Entity::from_raw(300),
			])),
			entity.get::<ActedOnTargets<_Actor>>(),
		)
	}

	#[test]
	fn do_remove_entities_not_contained_in_not_added_interacting_entities() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				InteractingEntities::new([
					ChildColliderOf(Entity::from_raw(100)),
					ChildColliderOf(Entity::from_raw(300)),
				]),
				ActedOnTargets::<_Actor>::new([Entity::from_raw(100), Entity::from_raw(300)]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<ActedOnTargets<_Actor>>()
			.unwrap()
			.entities
			.insert(Entity::from_raw(123));
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&ActedOnTargets::<_Actor>::new([
				Entity::from_raw(100),
				Entity::from_raw(300),
				Entity::from_raw(123),
			])),
			entity.get::<ActedOnTargets<_Actor>>(),
		)
	}

	#[test]
	fn remove_entities_not_contained_in_mutable_dereferenced_interacting_entities() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				InteractingEntities::new([
					ChildColliderOf(Entity::from_raw(100)),
					ChildColliderOf(Entity::from_raw(300)),
				]),
				ActedOnTargets::<_Actor>::new([Entity::from_raw(100), Entity::from_raw(300)]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<InteractingEntities>()
			.unwrap()
			.deref_mut();
		app.world_mut()
			.entity_mut(entity)
			.insert(ActedOnTargets::<_Actor>::new([
				Entity::from_raw(100),
				Entity::from_raw(200),
				Entity::from_raw(300),
			]));
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&ActedOnTargets::<_Actor>::new([
				Entity::from_raw(100),
				Entity::from_raw(300)
			])),
			entity.get::<ActedOnTargets<_Actor>>(),
		)
	}
}
