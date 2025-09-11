use crate::{
	components::is_blocker::Blocker,
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	tools::{Done, Units, speed::Speed},
	traits::accessors::get::RefInto,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub trait HandlesPhysicalObjects {
	type TSystems: SystemSet;
	type TPhysicalObjectComponent: Component + From<PhysicalObject>;

	const SYSTEMS: Self::TSystems;
}

pub trait HandlesMotion {
	/// The component controlling physical motion and related physical and collider computations.
	///
	/// Implementors must make sure this works on top level entities. No guarantees are made for
	/// entities that are a child of other entities.
	type TMotion: Component
		+ From<LinearMotion>
		+ for<'a> RefInto<'a, Done>
		+ for<'a> RefInto<'a, LinearMotion>;
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum LinearMotion {
	Direction { speed: Speed, direction: Dir3 },
	ToTarget { speed: Speed, target: Vec3 },
	Stop,
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
