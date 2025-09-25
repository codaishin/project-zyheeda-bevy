use crate::{
	assets::agent_config::{Bones, dto::BonesConfig},
	components::enemy::{Enemy, enemy_type::EnemyTypeInternal},
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
use bevy_rapier3d::geometry::Collider;
use common::{
	components::{ground_offset::GroundOffset, insert_asset::InsertAsset},
	errors::Error,
	tools::{
		Units,
		UnitsPerSecond,
		action_key::slot::SlotKey,
		aggro_range::AggroRange,
		attack_range::AttackRange,
		collider_radius::ColliderRadius,
		speed::Speed,
	},
	traits::{
		handles_agents::AgentType,
		handles_enemies::{EnemySkillUsage, EnemyTarget, EnemyType},
		handles_skill_behaviors::SkillSpawner,
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, f32::consts::PI, time::Duration};

#[derive(Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[require(Enemy = VoidSphere::enemy())]
pub struct VoidSphere;

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

	/// We use the same name for hand/forearm/essence slots.
	pub(crate) const ALL_PURPOSE_SLOT_BONE: &str = "slot";

	pub(crate) const SKILL_SPAWN: &str = "skill_spawn";
	pub(crate) const SKILL_SPAWN_NEUTRAL: &str = "skill_spawn_neutral";

	pub(crate) fn enemy() -> Enemy {
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
}

impl From<VoidSphere> for AgentType {
	fn from(_: VoidSphere) -> Self {
		Self::Enemy(EnemyType::VoidSphere)
	}
}

impl Prefab<()> for VoidSphere {
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Error> {
		let transform = Transform::from_translation(Self::GROUND_OFFSET);
		let mut transform_2nd_ring = transform;
		transform_2nd_ring.rotate_axis(Dir3::Z, PI / 2.);

		entity
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
			// One unified slot bone
			.with_child((
				Transform::from_translation(Self::SLOT_OFFSET),
				Name::from(Self::ALL_PURPOSE_SLOT_BONE),
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

impl BonesConfig for VoidSphere {
	fn bones() -> Bones {
		Bones {
			spawners: HashMap::from([
				(
					VoidSphere::SKILL_SPAWN_NEUTRAL.to_owned(),
					SkillSpawner::Neutral,
				),
				(
					VoidSphere::SKILL_SPAWN.to_owned(),
					SkillSpawner::Slot(VoidSphere::SLOT_KEY),
				),
			]),
			hand_slots: HashMap::from([(
				VoidSphere::ALL_PURPOSE_SLOT_BONE.to_owned(),
				VoidSphere::SLOT_KEY,
			)]),
			forearm_slots: HashMap::from([(
				VoidSphere::ALL_PURPOSE_SLOT_BONE.to_owned(),
				VoidSphere::SLOT_KEY,
			)]),
			essence_slots: HashMap::from([(
				VoidSphere::ALL_PURPOSE_SLOT_BONE.to_owned(),
				VoidSphere::SLOT_KEY,
			)]),
		}
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
		VoidSphere::SLOT_KEY
	}
}

impl From<&VoidSphere> for GroundOffset {
	fn from(_: &VoidSphere) -> Self {
		Self::from(VoidSphere::GROUND_OFFSET)
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
