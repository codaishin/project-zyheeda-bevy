use crate::traits::effect_material::EffectMaterial;
use bevy::{pbr::NotShadowCaster, prelude::*};
use common::traits::{
	track::{IsTracking, Track, Untrack},
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};
use std::{
	any::TypeId,
	collections::{hash_map::Entry, HashMap, HashSet},
};

#[derive(Component, Default, Debug, PartialEq)]
pub struct ShadowsManager {
	has_shadow: HashMap<Entity, HashSet<TypeId>>,
}

impl ShadowsManager {
	pub(crate) fn system(
		mut commands: Commands,
		managers: Query<&ShadowsManager, Changed<ShadowsManager>>,
	) {
		for manager in &managers {
			for (entity, has_shadow) in &manager.has_shadow {
				if has_shadow.is_empty() {
					commands.try_insert_on(*entity, NotShadowCaster);
				} else {
					commands.try_remove_from::<NotShadowCaster>(*entity);
				}
			}
		}
	}

	#[cfg(test)]
	fn with_shadow_caster<T>(mut self, entity: Entity) -> Self
	where
		T: 'static,
	{
		match self.has_shadow.entry(entity) {
			Entry::Occupied(mut entry) => {
				entry.get_mut().insert(TypeId::of::<T>());
			}
			Entry::Vacant(entry) => {
				entry.insert(HashSet::from([TypeId::of::<T>()]));
			}
		};

		self
	}

	#[cfg(test)]
	fn with_no_shadow_casters(mut self, entity: Entity) -> Self {
		match self.has_shadow.entry(entity) {
			Entry::Occupied(mut entry) => {
				entry.insert(HashSet::new());
			}
			Entry::Vacant(entry) => {
				entry.insert(HashSet::new());
			}
		};

		self
	}
}

impl<TShader> Track<Handle<TShader>> for ShadowsManager
where
	TShader: Asset + EffectMaterial,
{
	fn track(&mut self, entity: Entity, _: &Handle<TShader>) {
		match (self.has_shadow.entry(entity), TShader::casts_shadows()) {
			(Entry::Vacant(entry), true) => {
				entry.insert(HashSet::from([TypeId::of::<TShader>()]));
			}
			(Entry::Vacant(entry), false) => {
				entry.insert(HashSet::from([]));
			}
			(Entry::Occupied(mut entry), true) => {
				entry.get_mut().insert(TypeId::of::<TShader>());
			}
			_ => {}
		}
	}
}

impl<TShader> IsTracking<Handle<TShader>> for ShadowsManager
where
	TShader: Asset,
{
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.has_shadow.contains_key(entity)
	}
}

impl<TShader> Untrack<Handle<TShader>> for ShadowsManager
where
	TShader: Asset,
{
	fn untrack(&mut self, entity: &Entity) {
		let Entry::Occupied(mut entry) = self.has_shadow.entry(*entity) else {
			return;
		};

		entry.get_mut().remove(&TypeId::of::<TShader>());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{pbr::NotShadowCaster, render::render_resource::AsBindGroup};
	use common::test_tools::utils::new_handle;
	use std::ops::DerefMut;

	#[derive(Asset, TypePath, Clone, AsBindGroup)]
	struct _CastsShadowsA {}

	impl Material for _CastsShadowsA {}

	impl EffectMaterial for _CastsShadowsA {
		fn casts_shadows() -> bool {
			true
		}
	}

	#[derive(Asset, TypePath, Clone, AsBindGroup)]
	struct _CastsShadowsB {}

	impl Material for _CastsShadowsB {}

	impl EffectMaterial for _CastsShadowsB {
		fn casts_shadows() -> bool {
			true
		}
	}

	#[derive(Asset, TypePath, Clone, AsBindGroup)]
	struct _CastsNoShadows {}

	impl Material for _CastsNoShadows {}

	impl EffectMaterial for _CastsNoShadows {
		fn casts_shadows() -> bool {
			false
		}
	}

	fn as_tracker_for<T>(
		shadow_manager: &mut ShadowsManager,
	) -> &mut (impl IsTracking<Handle<T>> + Track<Handle<T>> + Untrack<Handle<T>>)
	where
		T: EffectMaterial + Asset,
	{
		shadow_manager
	}

	#[test]
	fn track_when_cast_shadow_is_true() {
		let mut shadow_manager = ShadowsManager::default();

		shadow_manager.track(Entity::from_raw(42), &new_handle::<_CastsShadowsA>());

		assert_eq!(
			ShadowsManager::default().with_shadow_caster::<_CastsShadowsA>(Entity::from_raw(42)),
			shadow_manager
		);
	}

	#[test]
	fn track_multiple_when_cast_shadow_is_true() {
		let mut shadow_manager = ShadowsManager::default();

		shadow_manager.track(Entity::from_raw(42), &new_handle::<_CastsShadowsA>());
		shadow_manager.track(Entity::from_raw(42), &new_handle::<_CastsShadowsB>());

		assert_eq!(
			ShadowsManager::default()
				.with_shadow_caster::<_CastsShadowsA>(Entity::from_raw(42))
				.with_shadow_caster::<_CastsShadowsB>(Entity::from_raw(42)),
			shadow_manager
		);
	}

	#[test]
	fn do_not_track_when_cast_shadow_is_false() {
		let mut shadow_manager = ShadowsManager::default();

		shadow_manager.track(Entity::from_raw(42), &new_handle::<_CastsNoShadows>());

		assert_eq!(
			ShadowsManager::default().with_no_shadow_casters(Entity::from_raw(42)),
			shadow_manager
		);
	}

	#[test]
	fn do_not_track_when_cast_shadow_is_false_and_already_tracking_something_else() {
		let mut shadow_manager = ShadowsManager::default();

		shadow_manager.track(Entity::from_raw(42), &new_handle::<_CastsShadowsA>());
		shadow_manager.track(Entity::from_raw(42), &new_handle::<_CastsNoShadows>());

		assert_eq!(
			ShadowsManager::default().with_shadow_caster::<_CastsShadowsA>(Entity::from_raw(42)),
			shadow_manager
		);
	}

	#[test]
	fn is_tracking_true() {
		let mut shadow_manager = ShadowsManager::default();

		let tracker = as_tracker_for::<_CastsShadowsA>(&mut shadow_manager);
		tracker.track(Entity::from_raw(42), &new_handle());

		assert!(tracker.is_tracking(&Entity::from_raw(42)));
	}

	#[test]
	fn is_tracking_false() {
		let mut shadow_manager = ShadowsManager::default();

		let tracker = as_tracker_for::<_CastsShadowsA>(&mut shadow_manager);
		tracker.track(Entity::from_raw(43), &new_handle());

		assert!(!tracker.is_tracking(&Entity::from_raw(42)));
	}

	#[test]
	fn untrack_entity() {
		let mut shadow_manager = ShadowsManager::default();

		let tracker = as_tracker_for::<_CastsShadowsA>(&mut shadow_manager);
		tracker.track(Entity::from_raw(42), &new_handle());
		tracker.untrack(&Entity::from_raw(42));

		assert_eq!(
			ShadowsManager::default().with_no_shadow_casters(Entity::from_raw(42)),
			shadow_manager
		);
	}

	#[test]
	fn untrack_entity_when_multiple_elements_stored_for_entity() {
		let mut shadow_manager = ShadowsManager::default();

		{
			let tracker = as_tracker_for::<_CastsShadowsA>(&mut shadow_manager);
			tracker.track(Entity::from_raw(42), &new_handle());
		}
		{
			let tracker = as_tracker_for::<_CastsShadowsB>(&mut shadow_manager);
			tracker.track(Entity::from_raw(42), &new_handle());
			tracker.untrack(&Entity::from_raw(42));
		}

		assert_eq!(
			ShadowsManager::default().with_shadow_caster::<_CastsShadowsA>(Entity::from_raw(42)),
			shadow_manager
		);
	}

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, ShadowsManager::system);

		app
	}

	#[test]
	fn remove_not_shadow_casting_when_casting_shadow_handle_tracked() {
		let mut app = setup();
		let entity = app.world_mut().spawn(NotShadowCaster).id();
		app.world_mut()
			.spawn(ShadowsManager::default().with_shadow_caster::<_CastsShadowsA>(entity));

		app.update();

		assert!(!app.world().entity(entity).contains::<NotShadowCaster>());
	}

	#[test]
	fn insert_not_shadow_casting_when_casting_no_shadow_handle_tracked() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut()
			.spawn(ShadowsManager::default().with_no_shadow_casters(entity));

		app.update();

		assert!(app.world().entity(entity).contains::<NotShadowCaster>());
	}

	#[test]
	fn do_work_only_once() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		app.world_mut()
			.spawn(ShadowsManager::default().with_no_shadow_casters(entity));

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<NotShadowCaster>();
		app.update();

		assert!(!app.world().entity(entity).contains::<NotShadowCaster>());
	}

	#[test]
	fn do_work_again_when_manager_mutably_dereferenced() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let manager = app
			.world_mut()
			.spawn(ShadowsManager::default().with_no_shadow_casters(entity))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<NotShadowCaster>();
		app.world_mut()
			.entity_mut(manager)
			.get_mut::<ShadowsManager>()
			.unwrap()
			.deref_mut();
		app.update();

		assert!(app.world().entity(entity).contains::<NotShadowCaster>());
	}
}
