mod bevy_impls;

use crate::{
	attributes::{effect_target::EffectTarget, health::Health},
	components::is_blocker::Blocker,
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	tools::{Done, Units, speed::Speed},
	traits::{
		accessors::get::{GetProperty, Property},
		cast_ray::TimeOfImpact,
	},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub trait HandlesRaycast {
	type TRaycast<'world, 'state>: SystemParam
		+ for<'w, 's> SystemParam<Item<'w, 's>: Raycast<SolidObjects>>
		+ for<'w, 's> SystemParam<Item<'w, 's>: Raycast<Ground>>;
}

/// Helper type to designate [`HandlesRaycast::TRaycast`] as a [`SystemParam`] implementation for a
/// given generic system constraint
pub type RaycastSystemParam<'w, 's, T> = <T as HandlesRaycast>::TRaycast<'w, 's>;

pub trait Raycast<TExtra>
where
	TExtra: RaycastExtra,
{
	fn raycast(&self, ray: Ray3d, constraints: TExtra) -> TExtra::TResult;
}

pub trait RaycastExtra {
	type TResult;
}

pub trait HandlesPhysicalAttributes {
	type TDefaultAttributes: Component + From<PhysicalDefaultAttributes>;
}

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
	type TMotion: Component + From<LinearMotion> + GetProperty<Done> + GetProperty<LinearMotion>;
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum LinearMotion {
	Direction { speed: Speed, direction: Dir3 },
	ToTarget { speed: Speed, target: Vec3 },
	Stop,
}

impl Property for LinearMotion {
	type TValue<'a> = Self;
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
	HandlesPhysicalEffect<HealthDamage, TAffectedComponent: GetProperty<Health>>
{
}

impl<T> HandlesLife for T where
	T: HandlesPhysicalEffect<HealthDamage, TAffectedComponent: GetProperty<Health>>
{
}

pub trait Effect {
	type TTarget;
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicalDefaultAttributes {
	pub health: Health,
	pub force_interaction: EffectTarget<Force>,
	pub gravity_interaction: EffectTarget<Gravity>,
}

impl Property for PhysicalDefaultAttributes {
	type TValue<'a> = Self;
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

#[derive(Debug, PartialEq)]
pub struct Ground;

impl RaycastExtra for Ground {
	type TResult = Option<TimeOfImpact>;
}

#[derive(Debug, PartialEq, Default)]
pub struct SolidObjects {
	pub exclude: Vec<Entity>,
}

impl RaycastExtra for SolidObjects {
	type TResult = Option<RaycastHit>;
}

#[derive(Debug, PartialEq)]
pub struct RaycastHit {
	pub entity: Entity,
	pub time_of_impact: f32,
}
