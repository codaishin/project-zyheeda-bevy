use crate::systems::apply_pull::PullAbleByGravity;
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
use std::vec::Drain;

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

	pub(crate) fn push(&mut self, pull: GravityPull) {
		match self {
			GravityAffected::AffectedBy { pulls } => pulls.push(pull),
			GravityAffected::Immune => {}
		}
	}
}

impl PullAbleByGravity for GravityAffected {
	type TDrain<'a> = DrainPulls<'a>;

	fn is_pulled(&self) -> bool {
		match self {
			GravityAffected::AffectedBy { pulls } => !pulls.is_empty(),
			GravityAffected::Immune => false,
		}
	}

	fn drain_pulls(&mut self) -> Self::TDrain<'_> {
		match self {
			GravityAffected::AffectedBy { pulls } => DrainPulls::Iterator(pulls.drain(..)),
			GravityAffected::Immune => DrainPulls::None,
		}
	}
}

pub(crate) enum DrainPulls<'a> {
	Iterator(Drain<'a, GravityPull>),
	None,
}

impl<'a> Iterator for DrainPulls<'a> {
	type Item = GravityPull;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			DrainPulls::Iterator(it) => it.next(),
			DrainPulls::None => None,
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
			AttributeOnSpawn(EffectTarget::Immune) => Self::Immune,
		}
	}
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct GravityPull {
	pub(crate) strength: UnitsPerSecond,
	pub(crate) towards: PersistentEntity,
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test_case(GravityAffected::affected([GravityPull::default()]), true; "true when affected")]
	#[test_case(GravityAffected::affected([]), false; "false when affected and empty")]
	#[test_case(GravityAffected::Immune, false; "false when immune")]
	fn is_pulled(affected: GravityAffected, expected: bool) {
		assert_eq!(expected, affected.is_pulled());
	}

	#[test]
	fn drain_affected() {
		let towards = PersistentEntity::default();
		let mut affected = GravityAffected::affected([
			GravityPull {
				strength: UnitsPerSecond::from(42.),
				towards,
			},
			GravityPull {
				strength: UnitsPerSecond::from(11.),
				towards,
			},
		]);

		let drained = affected.drain_pulls().collect::<Vec<_>>();
		assert_eq!(
			(
				GravityAffected::affected([]),
				vec![
					GravityPull {
						strength: UnitsPerSecond::from(42.),
						towards,
					},
					GravityPull {
						strength: UnitsPerSecond::from(11.),
						towards,
					}
				]
			),
			(affected, drained)
		)
	}

	#[test]
	fn drain_immune() {
		let mut immune = GravityAffected::Immune;

		let drained = immune.drain_pulls().collect::<Vec<_>>();
		assert_eq!((GravityAffected::Immune, vec![]), (immune, drained))
	}
}
