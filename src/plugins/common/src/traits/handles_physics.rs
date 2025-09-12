use crate::{
	attributes::health::Health,
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
	HandlesLife + HandlesPhysicalEffect<Gravity> + HandlesPhysicalEffect<Force>
{
}

impl<T> HandlesAllPhysicalEffects for T where
	T: HandlesLife + HandlesPhysicalEffect<Gravity> + HandlesPhysicalEffect<Force>
{
}

pub trait HandlesPhysicalEffect<TEffect>
where
	TEffect: Effect,
{
	type TEffectComponent: Component;
	type TAffectedComponent: Component;

	fn into_effect_component(effect: TEffect) -> Self::TEffectComponent;
}

pub trait HandlesLife:
	HandlesPhysicalEffect<HealthDamage, TAffectedComponent: for<'a> RefInto<'a, Health>>
{
}

impl<T> HandlesLife for T where
	T: HandlesPhysicalEffect<HealthDamage, TAffectedComponent: for<'a> RefInto<'a, Health>>
{
}

pub trait Effect {
	type TTarget;
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
