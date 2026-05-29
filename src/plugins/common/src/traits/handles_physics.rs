pub mod physical_bodies;

use crate::{
	attributes::{effect_target::EffectTarget, health::Health},
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	toi,
	tools::{Units, speed::Speed},
	traits::{
		accessors::get::{GetContext, TryGetContextMut, View, ViewField},
		handles_physics::physical_bodies::{Blocker, BodyConfig},
	},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use macros::EntityKey;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashSet,
	ops::{Deref, DerefMut},
};

pub trait HandlesRaycast {
	/// Marks the world camera used in [`MouseHover`] raycasting. Only one instance may exist in
	/// the world.
	type TWorldCamera: Component + Default;

	/// Raycast system parameter. [`MouseHover`] raycast requires that `Self::TWorldCamera` is being
	/// attached to the actual camera.
	type TRaycast: SystemParam
		+ for<'w, 's> SystemParam<Item<'w, 's>: Raycast<SolidObjects>>
		+ for<'w, 's> SystemParam<Item<'w, 's>: Raycast<Terrain>>
		+ for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseTerrainHover>>
		+ for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseHover>>;
}

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

pub trait HandlesPhysicsConfig {
	type TConfigMut: SystemParam
		+ for<'c> TryGetContextMut<NoDefaultAttributes, TContext<'c>: ConfigureDefaultAttributes>
		+ for<'c> TryGetContextMut<NoBodyConfigured, TContext<'c>: ConfigureBody>;
}

#[derive(EntityKey)]
pub struct NoDefaultAttributes {
	pub entity: Entity,
}

pub trait ConfigureDefaultAttributes {
	fn configure_default_attributes(&mut self, default: PhysicalDefaultAttributes);
}

impl<T> ConfigureDefaultAttributes for T
where
	T: DerefMut<Target: ConfigureDefaultAttributes>,
{
	fn configure_default_attributes(&mut self, default: PhysicalDefaultAttributes) {
		self.deref_mut().configure_default_attributes(default);
	}
}

#[derive(EntityKey)]
pub struct NoBodyConfigured {
	pub entity: Entity,
}

pub trait ConfigureBody {
	fn configure_body(&mut self, body: BodyConfig, offsets: TranslationOffsets);
}

impl<T> ConfigureBody for T
where
	T: DerefMut<Target: ConfigureBody>,
{
	fn configure_body(&mut self, body: BodyConfig, offsets: TranslationOffsets) {
		self.deref_mut().configure_body(body, offsets);
	}
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct TranslationOffsets {
	pub aim: f32,
	pub center: f32,
}

impl TranslationOffsets {
	pub const ZERO: Self = Self {
		aim: 0.,
		center: 0.,
	};
}

pub trait HandlesInteractiveDetection {
	type TInteractions: SystemParam
		+ for<'c> GetContext<Interactions, TContext<'c>: IterInteractions>;
}

pub trait IterInteractions {
	type TIter<'a>: ExactSizeIterator<Item = Entity>
	where
		Self: 'a;

	fn iter_interactions(&self) -> Self::TIter<'_>;
}

impl<T> IterInteractions for T
where
	T: Deref<Target: IterInteractions>,
{
	type TIter<'a>
		= <T::Target as IterInteractions>::TIter<'a>
	where
		Self: 'a;

	fn iter_interactions(&self) -> Self::TIter<'_> {
		self.deref().iter_interactions()
	}
}

#[derive(EntityKey)]
pub struct Interactions {
	pub entity: Entity,
}

pub trait HandlesMotion {
	/// The component controlling physical motion of characters and related physical and collider
	/// computations.
	///
	/// Implementors must make sure this works on top level entities. No guarantees are made for
	/// entities that are a child of other entities.
	type TCharacterMotion: Component + From<CharacterMotion> + View<CharacterMotion>;
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum CharacterMotion {
	Direction { speed: Speed, direction: Dir3 },
	ToTarget { speed: Speed, target: Vec3 },
	Done,
}

impl ViewField for CharacterMotion {
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
	HandlesPhysicalEffect<HealthDamage, TAffectedComponent: View<Health>>
{
}

impl<T> HandlesLife for T where
	T: HandlesPhysicalEffect<HealthDamage, TAffectedComponent: View<Health>>
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

impl Default for PhysicalDefaultAttributes {
	fn default() -> Self {
		Self {
			health: Health::new(10.),
			force_interaction: EffectTarget::Affected,
			gravity_interaction: EffectTarget::Affected,
		}
	}
}

impl ViewField for PhysicalDefaultAttributes {
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
pub struct Terrain {
	pub ray: Ray3d,
}

impl RaycastResult for Terrain {
	type TResult = Option<TimeOfImpact>;
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct TimeOfImpact(f32);

impl TimeOfImpact {
	pub const ZERO: Self = toi!(0.);

	pub const fn try_from_f32(toi: f32) -> Result<Self, IsNaN> {
		if toi.is_nan() {
			return Err(IsNaN);
		}

		Ok(Self(toi))
	}
}

impl From<Units> for TimeOfImpact {
	fn from(toi: Units) -> Self {
		Self(*toi)
	}
}

impl TryFrom<f32> for TimeOfImpact {
	type Error = IsNaN;

	fn try_from(toi: f32) -> Result<Self, Self::Error> {
		Self::try_from_f32(toi)
	}
}

impl Deref for TimeOfImpact {
	type Target = f32;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Create a [`TimeOfImpact`] value at compile time.
#[macro_export]
macro_rules! toi {
	($toi:expr) => {{
		const TOI: $crate::traits::handles_physics::TimeOfImpact =
			match $crate::traits::handles_physics::TimeOfImpact::try_from_f32($toi) {
				Ok(toi) => toi,
				Err(IsNaN) => panic!("invalid time of impact"),
			};
		TOI
	}};
}

impl Eq for TimeOfImpact {}

impl PartialOrd for TimeOfImpact {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for TimeOfImpact {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.0.partial_cmp(&other.0).unwrap_or_else(|| {
			unreachable!("Should not have happened, `NaN`s are not allowed in `TimeOfImpact`")
		})
	}
}

pub struct IsNaN;

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

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct MouseHover {
	pub mode: HoverMode,
	pub exclude: Vec<Entity>,
}

impl MouseHover {
	pub const TERRAIN_WITHOUT_EXCLUDES: Self = Self {
		exclude: vec![],
		mode: HoverMode::ColliderOrTerrain,
	};

	pub fn excluding(exclude: impl IntoIterator<Item = Entity>) -> Self {
		Self {
			exclude: Vec::from_iter(exclude),
			..default()
		}
	}

	pub fn with_mode(mut self, mode: HoverMode) -> Self {
		self.mode = mode;
		self
	}
}

impl RaycastResult for MouseHover {
	type TResult = Option<MouseHoversOver>;
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub enum HoverMode {
	#[default]
	ColliderOrTerrain,
	ColliderOrDirectionFrom(Entity),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MouseHoversOver {
	/// Hovering over a point
	Point(Vec3),
	/// Hovering an entity on the point (which may not be the entities translation)
	Object { entity: Entity, point: Vec3 },
}

#[derive(Debug, PartialEq)]
pub struct MouseTerrainHover;

impl RaycastResult for MouseTerrainHover {
	type TResult = Option<MouseTerrainPoint>;
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct MouseTerrainPoint(pub Vec3);
