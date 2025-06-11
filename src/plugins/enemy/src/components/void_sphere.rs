use super::{enemy::Enemy, void_beam::VoidBeamAttack};
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
	components::{GroundOffset, insert_asset::InsertAsset},
	effects::{deal_damage::DealDamage, gravity::Gravity},
	errors::Error,
	tools::{Units, UnitsPerSecond, collider_radius::ColliderRadius},
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_effect::HandlesEffect,
		handles_enemies::EnemyTarget,
		prefab::Prefab,
	},
};
use std::{f32::consts::PI, sync::Arc, time::Duration};

#[derive(Component)]
#[require(
	Enemy = VoidSphere::with_attack_range(Units::new(5.)),
	GroundOffset = Self::GROUND_OFFSET,
	RigidBody = RigidBody::Dynamic,
	GravityScale = GravityScale(0.),
)]
pub struct VoidSphere;

impl VoidSphere {
	const GROUND_OFFSET: Vec3 = Vec3::new(0., 1.2, 0.);

	fn collider_radius() -> ColliderRadius {
		ColliderRadius(Units::new(VOID_SPHERE_OUTER_RADIUS))
	}

	fn with_attack_range(attack_range: Units) -> Enemy {
		Enemy {
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

	pub(crate) fn spawn(mut commands: Commands) {
		let directions = [
			("Sphere A", Vec3::new(1., 0., 1.)),
			("Sphere B", Vec3::new(-1., 0., 1.)),
			("Sphere C", Vec3::new(1., 0., -1.)),
			("Sphere D", Vec3::new(-1., 0., -1.)),
		];
		let distance = 10.;

		for (name, direction) in directions {
			commands.spawn((
				Name::new(name),
				VoidSphere,
				Transform::from_translation(direction * distance),
				Visibility::default(),
			));
		}
	}
}

impl<TInteractions> Prefab<TInteractions> for VoidSphere
where
	TInteractions: HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
{
	fn insert_prefab_components(&self, entity: &mut EntityCommands) -> Result<(), Error> {
		let transform = Transform::from_translation(Self::GROUND_OFFSET);
		let mut transform_2nd_ring = transform;
		transform_2nd_ring.rotate_axis(Dir3::Z, PI / 2.);

		entity
			.try_insert((
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
			.with_child((Collider::ball(VOID_SPHERE_OUTER_RADIUS), transform));

		Ok(())
	}
}

const VOID_SPHERE_INNER_RADIUS: f32 = 0.3;
const VOID_SPHERE_OUTER_RADIUS: f32 = 0.4;
const VOID_SPHERE_TORUS_RADIUS: f32 = 0.35;
const VOID_SPHERE_TORUS_RING_RADIUS: f32 = VOID_SPHERE_OUTER_RADIUS - VOID_SPHERE_TORUS_RADIUS;

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
				radius: VOID_SPHERE_INNER_RADIUS,
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
				major_radius: VOID_SPHERE_TORUS_RADIUS,
				minor_radius: VOID_SPHERE_TORUS_RING_RADIUS,
			})
		})
	}
}
