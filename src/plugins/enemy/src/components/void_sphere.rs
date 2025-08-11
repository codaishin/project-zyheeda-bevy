use super::{enemy_behavior::EnemyBehavior, void_beam::VoidBeamAttack};
use bevy::{
	color::{Color, LinearRgba},
	math::{Dir3, Vec3, primitives::Torus},
	pbr::{NotShadowCaster, StandardMaterial},
	prelude::*,
	render::mesh::Mesh,
	transform::components::Transform,
	utils::default,
};
use bevy_rapier3d::{
	dynamics::{GravityScale, RigidBody},
	geometry::Collider,
};
use common::{
	self,
	attributes::{
		affected_by::{Affected, AffectedBy},
		health::Health,
	},
	components::{ground_offset::GroundOffset, insert_asset::InsertAsset},
	effects::{deal_damage::DealDamage, gravity::Gravity},
	errors::Error,
	tools::{
		Units,
		UnitsPerSecond,
		action_key::slot::{NoValidSlotKey, SlotKey},
		bone::Bone,
		collider_radius::ColliderRadius,
	},
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_effect::HandlesEffect,
		handles_enemies::EnemyTarget,
		handles_skill_behaviors::SkillSpawner,
		load_asset::LoadAsset,
		mapper::Mapper,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, sync::Arc, time::Duration};

#[derive(Component, SavableComponent, Default, Clone, Serialize, Deserialize)]
#[require(
	GroundOffset = Self::GROUND_OFFSET,
	RigidBody = RigidBody::Dynamic,
	GravityScale = GravityScale(0.),
	EnemyBehavior = VoidSphere::with_attack_range(Units::new(5.))
)]
pub struct VoidSphere;

impl VoidSphere {
	const GROUND_OFFSET: Vec3 = Vec3::new(0., 1.2, 0.);
	const INNER_RADIUS: f32 = 0.3;
	const OUTER_RADIUS: f32 = 0.4;
	const TORUS_RADIUS: f32 = 0.35;
	const TORUS_RING_RADIUS: f32 = Self::OUTER_RADIUS - Self::TORUS_RADIUS;

	const SLOT_OFFSET: Vec3 = Vec3::new(0., 0., -(Self::OUTER_RADIUS + Self::TORUS_RING_RADIUS));
	pub(crate) const SLOT_NAME: &str = "skill_slot";

	fn collider_radius() -> ColliderRadius {
		ColliderRadius(Units::new(Self::OUTER_RADIUS))
	}

	pub(crate) fn with_attack_range(attack_range: Units) -> EnemyBehavior {
		EnemyBehavior {
			speed: UnitsPerSecond::new(1.).into(),
			movement_animation: None,
			aggro_range: Units::new(10.).into(),
			attack_range: attack_range.into(),
			target: EnemyTarget::Player,
			attack: Arc::new(VoidBeamAttack {
				damage: 10.,
				lifetime: Duration::from_secs(1),
				range: attack_range,
			}),
			cool_down: Duration::from_secs(5),
			collider_radius: Self::collider_radius(),
		}
	}
}

impl<TInteractions> Prefab<TInteractions> for VoidSphere
where
	TInteractions: HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
{
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Error> {
		let transform = Transform::from_translation(Self::GROUND_OFFSET);
		let mut transform_2nd_ring = transform;
		transform_2nd_ring.rotate_axis(Dir3::Z, PI / 2.);

		entity
			.try_insert_if_new((
				Health::new(5.).bundle_via::<TInteractions>(),
				Affected::by::<Gravity>().bundle_via::<TInteractions>(),
			))
			.with_child((VoidSpherePart::Core, VoidSphereCore, transform))
			.with_child((
				VoidSpherePart::RingA(UnitsPerSecond::new(PI / 50.)),
				VoidSphereRing,
				transform,
			))
			.with_child((
				VoidSpherePart::RingB(UnitsPerSecond::new(PI / 75.)),
				VoidSphereRing,
				transform_2nd_ring,
			))
			.with_child((Collider::ball(Self::OUTER_RADIUS), transform))
			.with_child((
				Transform::from_translation(Self::SLOT_OFFSET),
				Name::from(Self::SLOT_NAME),
			));

		Ok(())
	}
}

impl Mapper<Bone<'_>, Option<SkillSpawner>> for VoidSphere {
	fn map(&self, Bone(name): Bone) -> Option<SkillSpawner> {
		if name != Self::SLOT_NAME {
			return None;
		}

		Some(SkillSpawner::Neutral)
	}
}

pub struct VoidSphereSlot;

impl From<VoidSphereSlot> for SlotKey {
	fn from(_: VoidSphereSlot) -> Self {
		Self(0)
	}
}

impl TryFrom<SlotKey> for VoidSphereSlot {
	type Error = NoValidSlotKey;

	fn try_from(SlotKey(key): SlotKey) -> Result<Self, Self::Error> {
		match key {
			0 => Ok(Self),
			_ => Err(NoValidSlotKey {
				slot_key: SlotKey(key),
			}),
		}
	}
}

#[derive(Component, Clone)]
#[require(Mesh3d, MeshMaterial3d<StandardMaterial>, NotShadowCaster)]
pub enum VoidSpherePart {
	Core,
	RingA(UnitsPerSecond),
	RingB(UnitsPerSecond),
}

#[derive(Component)]
#[require(
	InsertAsset::<StandardMaterial> = Self::material(),
	InsertAsset::<Mesh> =  Self::mesh(),
)]
struct VoidSphereCore;

impl VoidSphereCore {
	fn material() -> InsertAsset<StandardMaterial> {
		InsertAsset::shared::<Self>(|| StandardMaterial {
			base_color: Color::BLACK,
			metallic: 1.,
			..default()
		})
	}

	fn mesh() -> InsertAsset<Mesh> {
		InsertAsset::shared::<Self>(|| {
			Mesh::from(Sphere {
				radius: VoidSphere::INNER_RADIUS,
			})
		})
	}
}

#[derive(Component)]
#[require(
	InsertAsset::<StandardMaterial> = Self::material(),
	InsertAsset::<Mesh> = Self::mesh(),
)]
struct VoidSphereRing;

impl VoidSphereRing {
	fn material() -> InsertAsset<StandardMaterial> {
		InsertAsset::shared::<Self>(|| StandardMaterial {
			emissive: LinearRgba::new(23.0, 23.0, 23.0, 1.),
			..default()
		})
	}

	fn mesh() -> InsertAsset<Mesh> {
		InsertAsset::shared::<Self>(|| {
			Mesh::from(Torus {
				major_radius: VoidSphere::TORUS_RADIUS,
				minor_radius: VoidSphere::TORUS_RING_RADIUS,
			})
		})
	}
}
