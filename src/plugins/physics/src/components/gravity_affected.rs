use bevy::prelude::*;
use common::{
	attributes::effect_target::EffectTarget,
	components::persistent_entity::PersistentEntity,
	effects::gravity::Gravity,
	tools::{UnitsPerSecond, attribute::AttributeOnSpawn},
	traits::{
		accessors::get::RefInto,
		register_derived_component::{DerivableFrom, InsertDerivedComponent},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::{ops::RangeBounds, vec::Drain};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum GravityAffected {
	AffectedBy {
		#[serde(skip_serializing_if = "Vec::is_empty")]
		pulls: Vec<GravityPull>,
	},
	Immune,
}

impl GravityAffected {
	pub(crate) fn affected<T>(pulls: T) -> Self
	where
		T: IntoIterator<Item = GravityPull>,
	{
		Self::AffectedBy {
			pulls: pulls.into_iter().collect(),
		}
	}

	pub(crate) fn immune() -> Self {
		Self::Immune
	}

	pub(crate) fn is_not_pulled(&self) -> bool {
		match self {
			GravityAffected::AffectedBy { pulls } => pulls.is_empty(),
			GravityAffected::Immune => true,
		}
	}

	pub(crate) fn push(&mut self, pull: GravityPull) {
		match self {
			GravityAffected::AffectedBy { pulls } => pulls.push(pull),
			GravityAffected::Immune => {}
		}
	}

	pub(crate) fn drain_pulls<TRange>(&mut self, range: TRange) -> GravityDrain<'_>
	where
		TRange: RangeBounds<usize>,
	{
		match self {
			GravityAffected::AffectedBy { pulls } => GravityDrain::Pulls(pulls.drain(range)),
			GravityAffected::Immune => GravityDrain::None,
		}
	}
}

pub(crate) enum GravityDrain<'a> {
	Pulls(Drain<'a, GravityPull>),
	None,
}

impl<'a> Iterator for GravityDrain<'a> {
	type Item = GravityPull;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			GravityDrain::Pulls(drain) => drain.next(),
			GravityDrain::None => None,
		}
	}
}

impl<T> DerivableFrom<'_, '_, T> for GravityAffected
where
	T: for<'a> RefInto<'a, AttributeOnSpawn<EffectTarget<Gravity>>>,
{
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;

	type TParam = ();

	fn derive_from(_: Entity, component: &T, _: &()) -> Self {
		match component.ref_into() {
			AttributeOnSpawn(EffectTarget::Affected) => Self::affected([]),
			AttributeOnSpawn(EffectTarget::Immune) => Self::immune(),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct GravityPull {
	pub(crate) strength: UnitsPerSecond,
	pub(crate) towards: PersistentEntity,
}
