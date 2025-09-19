use bevy::prelude::*;
use common::{
	attributes::effect_target::EffectTarget,
	effects::force::Force,
	tools::attribute::AttributeOnSpawn,
	traits::{
		accessors::get::GetProperty,
		register_derived_component::{DerivableFrom, InsertDerivedComponent},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct ForceAffected(pub(crate) EffectTarget<Force>);

impl<T> DerivableFrom<'_, '_, T> for ForceAffected
where
	T: GetProperty<AttributeOnSpawn<EffectTarget<Force>>>,
{
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;

	type TParam = ();

	fn derive_from(_: Entity, component: &T, _: &()) -> Self {
		Self(component.get_property())
	}
}
