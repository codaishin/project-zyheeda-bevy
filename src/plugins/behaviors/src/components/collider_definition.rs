use bevy::prelude::*;
use common::{
	tools::{Units, collider_radius::ColliderRadius},
	traits::{
		accessors::get::GetProperty,
		register_derived_component::{DerivableFrom, InsertDerivedComponent},
	},
};

#[derive(Component, Debug, PartialEq)]
pub struct ColliderDefinition {
	pub(crate) radius: Units,
}

impl From<ColliderRadius> for ColliderDefinition {
	fn from(ColliderRadius(radius): ColliderRadius) -> Self {
		Self { radius }
	}
}

impl<T> DerivableFrom<'_, '_, T> for ColliderDefinition
where
	T: GetProperty<ColliderRadius>,
{
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::Always;

	type TParam = ();

	fn derive_from(_: Entity, component: &T, _: &()) -> Self {
		Self {
			radius: component.get_property(),
		}
	}
}
