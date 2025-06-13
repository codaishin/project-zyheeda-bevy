use crate::traits::handles_saving::SavableComponent;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Used as an [`Entity`] reference through different game sessions
///
/// Works in tandem with [`PersistentEntities`](crate::resources::persistent_entities::PersistentEntities).
/// Requires:
/// - either [`CommonPlugin`](crate::CommonPlugin)
/// - or [`RegisterPersistentEntities`](crate::traits::register_persistent_entities::RegisterPersistentEntities).
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use common::{
///   components::persistent_entity::PersistentEntity,
///   resources::persistent_entities::PersistentEntities,
///   traits::register_persistent_entities::RegisterPersistentEntities,
/// };
///
/// #[derive(Component)]
/// #[require(PersistentEntity)]
/// struct MyComponent(&'static str);
///
/// impl MyComponent {
///   fn demonstrate_persistence_entity_usage(
///     // needs to be mut, because it buffers failed attempts
///     mut persistent_entities: ResMut<PersistentEntities>,
///     entities: Query<&PersistentEntity>,
///     components: Query<&MyComponent>
///   ) {
///     let Ok(entity) = entities.single() else {
///       return;
///     };
///     let Some(entity) = persistent_entities.get_entity(entity) else {
///       return;
///     };
///     let Ok(&MyComponent(name)) = components.get(entity) else {
///       return;
///     };
///     assert_eq!("name", name);
///   }
/// }
///
/// let mut app = App::new();
/// app.register_persistent_entities(); // use `app.add_plugins(CommonPlugin);` for production code
/// app.add_systems(Update, MyComponent::demonstrate_persistence_entity_usage);
/// app.world_mut().spawn(MyComponent("name"));
/// app.update();
/// ```
#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
#[component(immutable)]
pub struct PersistentEntity(Uuid);

impl Default for PersistentEntity {
	fn default() -> Self {
		Self(Uuid::new_v4())
	}
}

impl SavableComponent for PersistentEntity {
	type TDto = Self;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn two_ids_are_different() {
		let a = PersistentEntity::default();
		let b = PersistentEntity::default();

		assert!(a != b);
	}
}
