use bevy::{ecs::entity::EntityHashSet, prelude::*};
use bevy_rapier3d::prelude::*;

/// Links a [`Collider`] entity to the corresponding [`InteractionTarget`] entity.
///
/// It is inserted automatically by an observer in the [`crate::CommonPlugin`].
#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, Copy, PartialOrd, Ord)]
#[relationship(relationship_target = InteractionColliders)]
#[require(Collider, Transform, ActiveEvents, ActiveCollisionTypes)]
pub struct ColliderOfInteractionTarget(pub(crate) Entity);

impl ColliderOfInteractionTarget {
	/// Creates a new [`ColliderOfInteractionTarget`] directly from an [`Entity`].
	///
	/// This bypasses the automatic insertion done by [`crate::CommonPlugin`]'s observer.
	/// Typically used in tests.
	pub fn from_raw(entity: Entity) -> Self {
		Self(entity)
	}

	pub fn target(&self) -> Entity {
		self.0
	}
}

/// Marks an entity as the target for interactions like damaging effects, healing, etc.
#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, Copy, PartialOrd, Ord, Default)]
#[component(immutable)]
pub struct InteractionTarget;

#[derive(Component, PartialEq, Debug, Clone)]
#[relationship_target(relationship = ColliderOfInteractionTarget)]
pub struct InteractionColliders(EntityHashSet);
