use crate::components::{
	affected::{force_affected::ForceAffected, gravity_affected::GravityAffected, life::Life},
	effects::{force::ForceEffect, gravity::GravityEffect, health_damage::HealthDamageEffect},
};
use bevy::prelude::{Component, default};
use common::prelude::*;
use macros::serde_model;
use std::{collections::HashSet, fmt::Debug, marker::PhantomData};

#[serde_model]
#[derive(Component)]
pub(crate) struct OngoingEffects<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	pub(crate) entities: HashSet<PersistentEntity>,
	#[serde(skip)]
	_p: PhantomData<(TActor, TTarget)>,
}

impl SavableComponent for OngoingEffects<HealthDamageEffect, Life> {
	type TDto = Self;

	const ID: UniqueComponentId = UniqueComponentId::from_str("ongoing_health_damage_effects");
}

impl SavableComponent for OngoingEffects<GravityEffect, GravityAffected> {
	type TDto = Self;

	const ID: UniqueComponentId = UniqueComponentId::from_str("ongoing_gravity_effects");
}

impl SavableComponent for OngoingEffects<ForceEffect, ForceAffected> {
	type TDto = Self;

	const ID: UniqueComponentId = UniqueComponentId::from_str("ongoing_force_effects");
}

impl<TActor, TTarget> Clone for OngoingEffects<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn clone(&self) -> Self {
		Self {
			entities: self.entities.clone(),
			_p: PhantomData,
		}
	}
}

impl<TActor, TTarget> Default for OngoingEffects<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn default() -> Self {
		Self {
			_p: PhantomData,
			entities: default(),
		}
	}
}

impl<TActor, TTarget> Debug for OngoingEffects<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Interactions")
			.field("entities", &self.entities)
			.field("_p", &self._p)
			.finish()
	}
}

impl<TActor, TTarget> PartialEq for OngoingEffects<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn eq(&self, other: &Self) -> bool {
		self._p == other._p && self.entities == other.entities
	}
}

#[cfg(test)]
impl<TActor, TTarget, const N: usize> From<[PersistentEntity; N]>
	for OngoingEffects<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn from(entities: [PersistentEntity; N]) -> Self {
		Self {
			entities: HashSet::from(entities),
			_p: PhantomData,
		}
	}
}
