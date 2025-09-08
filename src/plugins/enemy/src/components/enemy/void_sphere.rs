use crate::components::enemy::{Enemy, enemy_type::EnemyTypeInternal};
use bevy::{
	asset::AssetPath,
	color::{Color, LinearRgba},
	math::{Dir3, Vec3, primitives::Torus},
	pbr::{NotShadowCaster, StandardMaterial},
	prelude::*,
	render::mesh::Mesh,
	transform::components::Transform,
	utils::default,
};
use bevy_rapier3d::geometry::Collider;
use common::{
	self,
	attributes::{affected_by::Affected, health::Health},
	components::{ground_offset::GroundOffset, insert_asset::InsertAsset},
	effects::gravity::Gravity,
	errors::Error,
	tools::{
		Units,
		UnitsPerSecond,
		action_key::slot::{NoValidSlotKey, SlotKey},
		aggro_range::AggroRange,
		attack_range::AttackRange,
		bone::Bone,
		collider_radius::ColliderRadius,
		speed::Speed,
	},
	traits::{
		handles_enemies::{EnemySkillUsage, EnemyTarget},
		handles_physics::HandlesAllPhysicalEffects,
		handles_skill_behaviors::SkillSpawner,
		load_asset::LoadAsset,
		loadout::LoadoutConfig,
		mapper::Mapper,
		prefab::{Prefab, PrefabEntityCommands},
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use macros::item_asset;
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, time::Duration};

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct VoidSphere;

impl VoidSphere {
	const GROUND_OFFSET: Vec3 = Vec3::new(0., 1.2, 0.);
	const INNER_RADIUS: f32 = 0.3;
	const OUTER_RADIUS: f32 = 0.4;
	const TORUS_RADIUS: f32 = 0.35;
	const TORUS_RING_RADIUS: f32 = Self::OUTER_RADIUS - Self::TORUS_RADIUS;

	const SLOT_OFFSET: Vec3 = Vec3::new(
		Self::GROUND_OFFSET.x,
		Self::GROUND_OFFSET.y,
		Self::GROUND_OFFSET.z - (Self::OUTER_RADIUS + Self::TORUS_RING_RADIUS),
	);

	// We use the same name for hand/forearm/essence slots.
	pub(crate) const UNIFIED_SLOT_KEY: &str = "slot";

	pub(crate) const SKILL_SPAWN: &str = "skill_spawn";
	pub(crate) const SKILL_SPAWN_NEUTRAL: &str = "skill_spawn_neutral";

	pub(crate) fn new_enemy() -> Enemy {
		Enemy {
			speed: Speed(UnitsPerSecond::from(1.)),
			movement_animation: None,
			aggro_range: AggroRange(Units::from(10.)),
			attack_range: AttackRange(Units::from(5.)),
			target: EnemyTarget::Player,
			collider_radius: Self::collider_radius(),
			enemy_type: EnemyTypeInternal::VoidSphere(Self),
		}
	}

	fn collider_radius() -> ColliderRadius {
		ColliderRadius(Units::from(Self::OUTER_RADIUS))
	}

	fn unified_slot(bone: &str) -> Option<SlotKey> {
		if bone != Self::UNIFIED_SLOT_KEY {
			return None;
		}

		Some(SlotKey::from(VoidSphereSlot))
	}
}

impl<TPhysics> Prefab<TPhysics> for VoidSphere
where
	TPhysics: HandlesAllPhysicalEffects,
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
				Health::new(5.).component::<TPhysics>(),
				Affected::by::<Gravity>().component::<TPhysics>(),
			))
			.with_child((VoidSpherePart::Core, VoidSphereCore, transform))
			.with_child((
				VoidSpherePart::RingA(UnitsPerSecond::from(PI / 50.)),
				VoidSphereRing,
				transform,
			))
			.with_child((
				VoidSpherePart::RingB(UnitsPerSecond::from(PI / 75.)),
				VoidSphereRing,
				transform_2nd_ring,
			))
			.with_child((Collider::ball(Self::OUTER_RADIUS), transform))
			// One unified slot
			.with_child((
				Transform::from_translation(Self::SLOT_OFFSET),
				Name::from(Self::UNIFIED_SLOT_KEY),
			))
			// Skill spawn directly on slot offset
			.with_child((
				Transform::from_translation(Self::SLOT_OFFSET),
				Name::from(Self::SKILL_SPAWN),
			))
			// Neutral skill spawn directly on slot offset
			.with_child((
				Transform::from_translation(Self::SLOT_OFFSET),
				Name::from(Self::SKILL_SPAWN_NEUTRAL),
			));

		Ok(())
	}
}

impl LoadoutConfig for VoidSphere {
	fn inventory(&self) -> impl Iterator<Item = Option<AssetPath<'static>>> {
		std::iter::empty()
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<AssetPath<'static>>)> {
		std::iter::once((
			SlotKey::from(VoidSphereSlot),
			Some(AssetPath::from(item_asset!("void_beam"))),
		))
	}
}

impl VisibleSlots for VoidSphere {
	fn visible_slots(&self) -> impl Iterator<Item = SlotKey> {
		[SlotKey::from(VoidSphereSlot)].into_iter()
	}
}

impl Mapper<Bone<'_>, Option<SkillSpawner>> for VoidSphere {
	fn map(&self, Bone(name): Bone) -> Option<SkillSpawner> {
		match name {
			Self::SKILL_SPAWN => Some(SkillSpawner::Slot(SlotKey::from(VoidSphereSlot))),
			Self::SKILL_SPAWN_NEUTRAL => Some(SkillSpawner::Neutral),
			_ => None,
		}
	}
}

impl Mapper<Bone<'_>, Option<EssenceSlot>> for VoidSphere {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<EssenceSlot> {
		Self::unified_slot(bone).map(EssenceSlot)
	}
}

impl Mapper<Bone<'_>, Option<HandSlot>> for VoidSphere {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<HandSlot> {
		Self::unified_slot(bone).map(HandSlot)
	}
}

impl Mapper<Bone<'_>, Option<ForearmSlot>> for VoidSphere {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<ForearmSlot> {
		Self::unified_slot(bone).map(ForearmSlot)
	}
}

impl EnemySkillUsage for VoidSphere {
	fn hold_skill(&self) -> Duration {
		Duration::from_secs(2)
	}

	fn cool_down(&self) -> Duration {
		Duration::from_secs(5)
	}

	fn skill_key(&self) -> SlotKey {
		SlotKey::from(VoidSphereSlot)
	}
}

impl From<&VoidSphere> for GroundOffset {
	fn from(_: &VoidSphere) -> Self {
		Self::from(VoidSphere::GROUND_OFFSET)
	}
}

struct VoidSphereSlot;

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
