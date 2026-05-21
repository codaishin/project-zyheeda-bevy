use crate::{
	components::collider::ChildColliderOf,
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
	markers: Query<'w, 's, Option<&'static ChildColliderOf>, With<T>>,
}

impl<T> UpdateOngoingInteractions<'_, '_, T>
where
	T: Component,
{
	fn get_root(&self, entity: Entity) -> Option<Entity> {
		match self.markers.get(entity) {
			Ok(Some(ChildColliderOf(root))) => Some(*root),
			Ok(None) => Some(entity),
			Err(_) => None,
		}
	}
}

impl<T> PushOngoingInteraction for UpdateOngoingInteractions<'_, '_, T>
where
	T: Component,
{
	fn push_ongoing_interaction(&mut self, actor: Entity, target: Entity) {
		let Some(actor) = self.get_root(actor) else {
			return;
		};
		let Some(target) = self.get_root(target) else {
			return;
		};
		let targets = self.interactions.interactions.entry(actor).or_default();

		targets.insert(target);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::collections::HashSet;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Marker;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<OngoingInteractions<_Marker>>();

		app
	}

	#[test]
	fn add_event_pair() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app.world_mut().spawn(_Marker).id();
		let b = app.world_mut().spawn(_Marker).id();

		app.world_mut().run_system_once(
			move |mut sender: UpdateOngoingInteractions<_Marker>| {
				sender.push_ongoing_interaction(a, b);
			},
		)?;

		assert_eq!(
			&OngoingInteractions::from([(a, HashSet::from([b]))]),
			app.world().resource::<OngoingInteractions<_Marker>>()
		);
		Ok(())
	}

	#[test_case((), _Marker; "on first")]
	#[test_case(_Marker, (); "on second")]
	#[test_case((), (); "on both")]
	fn do_not_add_pair_if_marker_missing(
		bundle_a: impl Bundle,
		bundle_b: impl Bundle,
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app.world_mut().spawn(bundle_a).id();
		let b = app.world_mut().spawn(bundle_b).id();

		app.world_mut().run_system_once(
			move |mut sender: UpdateOngoingInteractions<_Marker>| {
				sender.push_ongoing_interaction(a, b);
			},
		)?;

		assert_eq!(
			&OngoingInteractions::from([]),
			app.world().resource::<OngoingInteractions<_Marker>>()
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
				.spawn((ChildColliderOf(roots[0]), _Marker))
				.id(),
			app.world_mut()
				.spawn((ChildColliderOf(roots[1]), _Marker))
				.id(),
		];

		app.world_mut().run_system_once(
			move |mut sender: UpdateOngoingInteractions<_Marker>| {
				sender.push_ongoing_interaction(colliders[0], colliders[1]);
			},
		)?;

		assert_eq!(
			&OngoingInteractions::from([(roots[0], HashSet::from([roots[1]]))]),
			app.world().resource::<OngoingInteractions<_Marker>>()
		);
		Ok(())
	}

	#[test]
	fn do_not_override_existing_entries() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app.world_mut().spawn(_Marker).id();
		let b = app.world_mut().spawn(_Marker).id();
		let c = app.world_mut().spawn(_Marker).id();

		app.world_mut()
			.insert_resource(OngoingInteractions::<_Marker>::from([(
				a,
				HashSet::from([b]),
			)]));
		app.world_mut().run_system_once(
			move |mut sender: UpdateOngoingInteractions<_Marker>| {
				sender.push_ongoing_interaction(a, c);
			},
		)?;

		assert_eq!(
			&OngoingInteractions::from([(a, HashSet::from([b, c]))]),
			app.world().resource::<OngoingInteractions<_Marker>>()
		);
		Ok(())
	}
}
