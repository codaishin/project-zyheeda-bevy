use crate::{
	components::collider::ChildCollider,
	resources::ongoing_interactions::OngoingInteractions,
	traits::send_collision_interaction::PushOngoingInteraction,
};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub(crate) struct UpdateOngoingInteractions<'w, 's, T>
where
	T: Component,
{
	interactions: ResMut<'w, OngoingInteractions<T>>,
	child_colliders: Query<'w, 's, &'static ChildCollider<T>>,
}

impl<T> UpdateOngoingInteractions<'_, '_, T>
where
	T: Component,
{
	fn get_root(&self, entity: Entity) -> Entity {
		match self.child_colliders.get(entity) {
			Ok(ChildCollider { root, .. }) => *root,
			Err(_) => entity,
		}
	}
}

impl<T> PushOngoingInteraction for UpdateOngoingInteractions<'_, '_, T>
where
	T: Component,
{
	fn push_ongoing_interaction(&mut self, actor: Entity, target: Entity) {
		let actor = self.get_root(actor);
		let target = self.get_root(target);
		let targets = self.interactions.targets.entry(actor).or_default();

		targets.insert(target);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::collections::HashSet;
	use testing::{SingleThreadedApp, fake_entity};

	#[derive(Component, Debug, PartialEq)]
	struct _C;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<OngoingInteractions<_C>>();

		app
	}

	#[test]
	fn add_event_pair() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut sender: UpdateOngoingInteractions<_C>| {
				sender.push_ongoing_interaction(fake_entity!(1), fake_entity!(2));
			})?;

		assert_eq!(
			&OngoingInteractions::from([(fake_entity!(1), HashSet::from([fake_entity!(2)]))]),
			app.world().resource::<OngoingInteractions<_C>>()
		);
		Ok(())
	}

	#[test]
	fn add_entity_roots() -> Result<(), RunSystemError> {
		let mut app = setup();
		let roots = [
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
		];
		let colliders = [
			app.world_mut()
				.spawn(ChildCollider::<_C>::of(roots[0]))
				.id(),
			app.world_mut()
				.spawn(ChildCollider::<_C>::of(roots[1]))
				.id(),
		];

		app.world_mut()
			.run_system_once(move |mut sender: UpdateOngoingInteractions<_C>| {
				sender.push_ongoing_interaction(colliders[0], colliders[1]);
			})?;

		assert_eq!(
			&OngoingInteractions::from([(roots[0], HashSet::from([roots[1]]))]),
			app.world().resource::<OngoingInteractions<_C>>()
		);
		Ok(())
	}

	#[test]
	fn do_not_override_existing_entries() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.insert_resource(OngoingInteractions::<_C>::from([(
				fake_entity!(1),
				HashSet::from([fake_entity!(11)]),
			)]));
		app.world_mut()
			.run_system_once(move |mut sender: UpdateOngoingInteractions<_C>| {
				sender.push_ongoing_interaction(fake_entity!(1), fake_entity!(2));
			})?;

		assert_eq!(
			&OngoingInteractions::from([(
				fake_entity!(1),
				HashSet::from([fake_entity!(11), fake_entity!(2)])
			)]),
			app.world().resource::<OngoingInteractions<_C>>()
		);
		Ok(())
	}
}
