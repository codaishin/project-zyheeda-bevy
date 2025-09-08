use crate::{
	components::is_blocker::Blocker,
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	tools::Units,
};
use bevy::prelude::*;
use std::collections::HashSet;

pub trait HandlesPhysics {
	type TSystems: SystemSet;
	type TPhysicalObjectComponent: Component + From<PhysicalObject>;

	const SYSTEMS: Self::TSystems;
}

pub trait HandlesAllPhysicalEffects:
	HandlesPhysicalEffect<HealthDamage> + HandlesPhysicalEffect<Gravity> + HandlesPhysicalEffect<Force>
{
}

impl<T> HandlesAllPhysicalEffects for T where
	T: HandlesPhysicalEffect<HealthDamage>
		+ HandlesPhysicalEffect<Gravity>
		+ HandlesPhysicalEffect<Force>
{
}

pub trait HandlesPhysicalEffect<TEffect>
where
	TEffect: Effect,
{
	type TEffectComponent: Component;
	type TAffectedComponent: Component;

	fn into_effect_component(effect: TEffect) -> Self::TEffectComponent;
	fn into_affected_component(affected: TEffect::TAffected) -> Self::TAffectedComponent;
}

pub trait Effect {
	type TAffected;
}

#[derive(Debug, PartialEq, Clone)]
pub enum PhysicalObject {
	Beam {
		range: Units,
		blocked_by: HashSet<Blocker>,
	},
	Fragile {
		destroyed_by: HashSet<Blocker>,
	},
}
