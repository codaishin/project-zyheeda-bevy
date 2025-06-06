use crate::components::{interacting_entities::InteractingEntities, interactions::Interactions};
use bevy::prelude::*;

impl<TActor, TTarget> Interactions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	pub(crate) fn untrack_non_interacting_targets(
		mut agents: Query<(&mut Self, &InteractingEntities), Changed<InteractingEntities>>,
	) {
		for (mut interactions, interacting_entities) in &mut agents {
			interactions.retain(|e| interacting_entities.contains(e));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;
	use std::ops::DerefMut;

	#[derive(Component)]
	struct _Actor;

	#[derive(Component)]
	struct _Target;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			Interactions::<_Actor, _Target>::untrack_non_interacting_targets,
		);

		app
	}

	#[test]
	fn remove_entities_not_contained_in_interacting_entities() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				InteractingEntities::new([Entity::from_raw(100), Entity::from_raw(300)]),
				Interactions::<_Actor, _Target>::from([
					Entity::from_raw(100),
					Entity::from_raw(200),
					Entity::from_raw(300),
				]),
			))
			.id();

		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&Interactions::<_Actor, _Target>::from([
				Entity::from_raw(100),
				Entity::from_raw(300),
			])),
			entity.get::<Interactions<_Actor, _Target>>(),
		)
	}

	#[test]
	fn do_remove_entities_not_contained_in_not_added_interacting_entities() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				InteractingEntities::new([Entity::from_raw(100), Entity::from_raw(300)]),
				Interactions::<_Actor, _Target>::from([
					Entity::from_raw(100),
					Entity::from_raw(300),
				]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Interactions<_Actor, _Target>>()
			.unwrap()
			.insert(Entity::from_raw(123));
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&Interactions::<_Actor, _Target>::from([
				Entity::from_raw(100),
				Entity::from_raw(300),
				Entity::from_raw(123),
			])),
			entity.get::<Interactions<_Actor, _Target>>(),
		)
	}

	#[test]
	fn remove_entities_not_contained_in_mutable_dereferenced_interacting_entities() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				InteractingEntities::new([Entity::from_raw(100), Entity::from_raw(300)]),
				Interactions::<_Actor, _Target>::from([
					Entity::from_raw(100),
					Entity::from_raw(300),
				]),
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
			.insert(Interactions::<_Actor, _Target>::from([
				Entity::from_raw(100),
				Entity::from_raw(200),
				Entity::from_raw(300),
			]));
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&Interactions::<_Actor, _Target>::from([
				Entity::from_raw(100),
				Entity::from_raw(300)
			])),
			entity.get::<Interactions<_Actor, _Target>>(),
		)
	}
}
