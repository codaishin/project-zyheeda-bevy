use bevy::prelude::*;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::traits::accessors::get::Property;

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
///   traits::register_persistent_entities::RegisterPersistentEntities,
///   traits::accessors::get::GetMut,
///   zyheeda_commands::ZyheedaCommands,
/// };
///
/// #[derive(Component)]
/// struct Seeker {
///   target: PersistentEntity,
/// }
///
/// #[derive(Component)]
/// struct Target;
///
/// impl Seeker {
///   fn set_target_name(
///     mut commands: ZyheedaCommands,
///     seekers: Query<&Seeker>
///   ) {
///     let Ok(Seeker { target }) = seekers.single() else {
///       return;
///     };
///     let Some(mut entity) = commands.get_mut(target) else {
///       return;
///     };
///     entity.try_insert(Name::from("target"));
///   }
/// }
///
/// let mut app = App::new();
/// app.register_persistent_entities(); // use `app.add_plugins(CommonPlugin);` for production code
/// app.add_systems(Update, Seeker::set_target_name);
///
/// let target_persistent_entity = PersistentEntity::default();
/// let target_entity = app.world_mut().spawn((Target, target_persistent_entity)).id();
/// app.world_mut().spawn(Seeker { target: target_persistent_entity });
/// app.update();
///
/// assert_eq!(
///   Some(&Name::from("target")),
///   app.world().entity(target_entity).get::<Name>(),
/// );
/// ```
#[derive(
	Component, SavableComponent, Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize,
)]
#[component(immutable)]
#[savable_component(has_priority)]
pub struct PersistentEntity(Uuid);

impl PersistentEntity {
	pub const fn from_uuid(uuid: Uuid) -> Self {
		Self(uuid)
	}
}

impl Default for PersistentEntity {
	fn default() -> Self {
		Self(Uuid::new_v4())
	}
}

impl Property for PersistentEntity {
	type TValue<'a> = Self;
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
