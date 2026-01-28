use crate::{
	assets::agent_config::{Bones, dto::BonesConfig},
	components::{
		enemy::{Enemy, attack_config::EnemyAttackConfig},
		movement_config::MovementConfig,
	},
};
use bevy::{
	color::{Color, LinearRgba},
	math::{Dir3, Vec3, primitives::Torus},
	pbr::{NotShadowCaster, StandardMaterial},
	prelude::*,
	render::mesh::Mesh,
	transform::components::Transform,
	utils::default,
};
use common::{
	components::{ground_offset::GroundOffset, insert_asset::InsertAsset},
	errors::Unreachable,
	tools::{Units, UnitsPerSecond, action_key::slot::SlotKey, bone_name::BoneName},
	traits::{
		handles_enemies::EnemyType,
		handles_map_generation::AgentType,
		handles_physics::physical_bodies::{
			Blocker,
			Body,
			HandlesPhysicalBodies,
			PhysicsType,
			Shape,
		},
		handles_skill_physics::SkillSpawner,
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, f32::consts::PI, sync::LazyLock, time::Duration};

#[derive(Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[require(
	Enemy = Self::enemy(),
	EnemyAttackConfig = Self::attack_config(),
	MovementConfig = Self::movement_config(),
	GroundOffset = GroundOffset(Self::GROUND_OFFSET),
)]
pub struct VoidSphere;

type LazyBoneName = LazyLock<BoneName>;

/// We use the same name for hand/forearm/essence slots.
static ALL_PURPOSE_SLOT_BONE: LazyBoneName = LazyLock::new(|| BoneName::from("slot"));
static SKILL_SPAWN: LazyBoneName = LazyLock::new(|| BoneName::from("skill_spawn"));
static SKILL_SPAWN_NEUTRAL: LazyBoneName = LazyLock::new(|| BoneName::from("skill_spawn_neutral"));

impl VoidSphere {
	const SLOT_KEY: SlotKey = SlotKey(0);

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

	fn attack_config() -> EnemyAttackConfig {
		EnemyAttackConfig {
			key: Self::SLOT_KEY,
			hold: Duration::from_secs(1),
			cooldown: Duration::from_secs(2),
		}
	}

	fn enemy() -> Enemy {
		Enemy {
			aggro_range: Units::from(8.),
			attack_range: Units::from(6.),
			min_target_distance: Some(Units::from(3.)),
		}
	}

	fn movement_config() -> MovementConfig {
		MovementConfig {
			collider_radius: Units::from(Self::OUTER_RADIUS),
			speed: UnitsPerSecond::from(1.),
		}
	}
}

impl From<VoidSphere> for AgentType {
	fn from(_: VoidSphere) -> Self {
		Self::Enemy(EnemyType::VoidSphere)
	}
}

impl<TPhysics> Prefab<TPhysics> for VoidSphere
where
	TPhysics: HandlesPhysicalBodies,
{
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Unreachable> {
		let child_transform = Transform::from_translation(Self::GROUND_OFFSET);
		let shape = Shape::Sphere {
			radius: Units::from(Self::OUTER_RADIUS),
		};
		let body = Body::from_shape(shape)
			.with_center_offset(Self::GROUND_OFFSET)
			.with_physics_type(PhysicsType::Agent)
			.with_blocker_types([Blocker::Character]);
		let mut transform_2nd_ring = child_transform;
		transform_2nd_ring.rotate_axis(Dir3::Z, PI / 2.);

		entity
			.try_insert_if_new(TPhysics::TBody::from(body))
			.with_child((VoidSpherePart::Core, VoidSphereCore, child_transform))
			.with_child((
				VoidSpherePart::RingA(UnitsPerSecond::from(PI / 50.)),
				VoidSphereRing,
				child_transform,
			))
			.with_child((
				VoidSpherePart::RingB(UnitsPerSecond::from(PI / 75.)),
				VoidSphereRing,
				transform_2nd_ring,
			))
			// One unified slot bone
			.with_child((
				Transform::from_translation(Self::SLOT_OFFSET),
				Name::from(ALL_PURPOSE_SLOT_BONE.clone()),
			))
			// Skill spawn directly on slot offset
			.with_child((
				Transform::from_translation(Self::SLOT_OFFSET),
				Name::from(SKILL_SPAWN.clone()),
			))
			// Neutral skill spawn directly on slot offset
			.with_child((
				Transform::from_translation(Self::SLOT_OFFSET),
				Name::from(SKILL_SPAWN_NEUTRAL.clone()),
			));

		Ok(())
	}
}

impl BonesConfig for VoidSphere {
	fn bones() -> Bones {
		Bones {
			spawners: HashMap::from([
				(SKILL_SPAWN_NEUTRAL.clone(), SkillSpawner::Neutral),
				(
					SKILL_SPAWN.clone(),
					SkillSpawner::Slot(VoidSphere::SLOT_KEY),
				),
			]),
			hand_slots: HashMap::from([(ALL_PURPOSE_SLOT_BONE.clone(), VoidSphere::SLOT_KEY)]),
			forearm_slots: HashMap::from([(ALL_PURPOSE_SLOT_BONE.clone(), VoidSphere::SLOT_KEY)]),
			essence_slots: HashMap::from([(ALL_PURPOSE_SLOT_BONE.clone(), VoidSphere::SLOT_KEY)]),
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
