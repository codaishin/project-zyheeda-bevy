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
use std::{collections::HashSet, ops::DerefMut};

pub trait HandlesRaycast {
	/// Marks the world camera used in [`MouseHover`] raycasting. Only one instance may exist in
	/// the world.
	type TWorldCamera: Component + Default;

	/// Colliders with this component are ignored when determining mouse hover targets
	type TNoMouseHover: Component + Default;

	/// Raycast system parameter. [`MouseHover`] raycast requires that `Self::TWorldCamera` is being
	/// attached to the actual camera.
	type TRaycast<'world, 'state>: SystemParam
		+ for<'w, 's> SystemParam<Item<'w, 's>: Raycast<SolidObjects>>
		+ for<'w, 's> SystemParam<Item<'w, 's>: Raycast<Ground>>
		+ for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseGroundHover>>
		+ for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseHover>>;
}

/// Helper type to designate [`HandlesRaycast::TRaycast`] as a [`SystemParam`] implementation for a
/// given generic system constraint
pub type RaycastSystemParam<'w, 's, T> = <T as HandlesRaycast>::TRaycast<'w, 's>;

pub trait Raycast<TArgs>
where
	TArgs: RaycastResult,
{
	fn raycast(&mut self, args: TArgs) -> TArgs::TResult;
}

impl<T, TArgs> Raycast<TArgs> for T
where
	T: DerefMut<Target: Raycast<TArgs>>,
	TArgs: RaycastResult,
{
	fn raycast(&mut self, args: TArgs) -> <TArgs as RaycastResult>::TResult {
		self.deref_mut().raycast(args)
	}
}

pub trait RaycastResult {
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
pub struct Ground {
	pub ray: Ray3d,
}

impl RaycastResult for Ground {
	type TResult = Option<TimeOfImpact>;
}

#[derive(Debug, PartialEq)]
pub struct SolidObjects {
	pub ray: Ray3d,
	pub exclude: Vec<Entity>,
	pub only_hoverable: bool,
}

impl RaycastResult for SolidObjects {
	type TResult = Option<RaycastHit>;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RaycastHit {
	pub entity: Entity,
	pub time_of_impact: f32,
}

#[derive(Debug, PartialEq)]
pub struct MouseHover {
	pub exclude: Vec<Entity>,
}

impl MouseHover {
	pub const NO_EXCLUDES: Self = Self { exclude: vec![] };
}

impl RaycastResult for MouseHover {
	type TResult = Option<MouseHoversOver>;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MouseHoversOver {
	/// Hovering over ground on the given point
	Ground { point: Vec3 },
	/// Hovering an entity on the given point (which may not be the entities translation)
	Object { entity: Entity, point: Vec3 },
}

#[derive(Debug, PartialEq)]
pub struct MouseGroundHover;

impl RaycastResult for MouseGroundHover {
	type TResult = Option<MouseGroundPoint>;
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct MouseGroundPoint(pub Vec3);
