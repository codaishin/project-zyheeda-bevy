use super::ToArc;
use crate::components::{
	AttackConfig,
	BeamConfig,
	Enemy,
	Foe,
	MovementConfig,
	MovementMode,
	VoidSphere,
	VoidSpherePart,
};
use bars::components::Bar;
use bevy::{
	ecs::{bundle::Bundle, system::EntityCommands},
	hierarchy::BuildChildren,
	math::{primitives::Torus, Vec3},
	pbr::{NotShadowCaster, PbrBundle, StandardMaterial},
	render::{color::Color, mesh::Mesh},
	transform::{components::Transform, TransformBundle},
	utils::default,
};
use bevy_rapier3d::{
	dynamics::{GravityScale, RigidBody},
	geometry::Collider,
};
use common::{
	bundles::ColliderTransformBundle,
	components::{effected_by::EffectedBy, ColliderRoot, GroundOffset, Health},
	errors::Error,
	tools::UnitsPerSecond,
	traits::clamp_zero_positive::ClampZeroPositive,
};
use gravity::components::Gravity;
use prefabs::traits::{sphere, AssetHandles, Instantiate};
use std::{f32::consts::PI, time::Duration};

#[derive(Bundle)]
pub struct PbrVoidSphereBundle {
	pbr_bundle: PbrBundle,
	not_shadow_caster: NotShadowCaster,
	void_sphere_part: VoidSpherePart,
}

impl PbrVoidSphereBundle {
	pub fn new(pbr_bundle: PbrBundle, part: VoidSpherePart) -> Self {
		Self {
			pbr_bundle,
			not_shadow_caster: NotShadowCaster,
			void_sphere_part: part,
		}
	}
}

impl Clone for PbrVoidSphereBundle {
	fn clone(&self) -> Self {
		Self {
			pbr_bundle: self.pbr_bundle.clone(),
			not_shadow_caster: NotShadowCaster,
			void_sphere_part: self.void_sphere_part.clone(),
		}
	}
}

const VOID_SPHERE_INNER_RADIUS: f32 = 0.3;
const VOID_SPHERE_OUTER_RADIUS: f32 = 0.4;
const VOID_SPHERE_TORUS_RADIUS: f32 = 0.35;
const VOID_SPHERE_TORUS_RING_RADIUS: f32 = VOID_SPHERE_OUTER_RADIUS - VOID_SPHERE_TORUS_RADIUS;
const VOID_SPHERE_GROUND_OFFSET: Vec3 = Vec3::new(0., 1.2, 0.);

struct VoidSphereCore;

struct VoidSphereRing;

impl Instantiate for VoidSphere {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut assets: impl AssetHandles,
	) -> Result<(), Error> {
		let core_material = StandardMaterial {
			base_color: Color::BLACK,
			metallic: 1.,
			..default()
		};
		let core_mesh = sphere(VOID_SPHERE_INNER_RADIUS);
		let ring_material = StandardMaterial {
			emissive: Color::rgb_linear(23000.0, 23000.0, 23000.0),
			..default()
		};
		let ring_mesh = Mesh::from(Torus {
			major_radius: VOID_SPHERE_TORUS_RADIUS,
			minor_radius: VOID_SPHERE_TORUS_RING_RADIUS,
		});
		let transform = Transform::from_translation(VOID_SPHERE_GROUND_OFFSET);
		let mut transform_2nd_ring = transform;
		transform_2nd_ring.rotate_axis(Vec3::Z, PI / 2.);

		on.try_insert((
			EffectedBy::<Gravity>::default(),
			GroundOffset(VOID_SPHERE_GROUND_OFFSET),
			RigidBody::Dynamic,
			GravityScale(0.),
			Health::new(5),
			Bar::default(),
			MovementConfig::Constant {
				mode: MovementMode::Slow,
				speed: UnitsPerSecond::new(1.),
			},
			AttackConfig {
				spawn: BeamConfig {
					damage: 1,
					color: Color::BLACK,
					emissive: Color::rgb_linear(23000.0, 23000.0, 23000.0),
					lifetime: Duration::from_secs(1),
					range: VoidSphere::ATTACK_RANGE,
				}
				.to_arc(),
				cool_down: Duration::from_secs(5),
			},
			Enemy {
				aggro_range: VoidSphere::AGGRO_RANGE,
				attack_range: VoidSphere::ATTACK_RANGE,
				foe: Foe::Player,
			},
		));
		on.with_children(|parent| {
			parent.spawn(PbrVoidSphereBundle::new(
				PbrBundle {
					mesh: assets.handle::<VoidSphereCore>(core_mesh),
					material: assets.handle::<VoidSphereCore>(core_material),
					transform,
					..default()
				},
				VoidSpherePart::Core,
			));
			parent.spawn(PbrVoidSphereBundle::new(
				PbrBundle {
					mesh: assets.handle::<VoidSphereRing>(ring_mesh.clone()),
					material: assets.handle::<VoidSphereRing>(ring_material.clone()),
					transform,
					..default()
				},
				VoidSpherePart::RingA(UnitsPerSecond::new(PI / 50.)),
			));
			parent.spawn(PbrVoidSphereBundle::new(
				PbrBundle {
					mesh: assets.handle::<VoidSphereRing>(ring_mesh),
					material: assets.handle::<VoidSphereRing>(ring_material),
					transform: transform_2nd_ring,
					..default()
				},
				VoidSpherePart::RingB(UnitsPerSecond::new(PI / 75.)),
			));
			parent.spawn((
				ColliderTransformBundle {
					transform: TransformBundle::from_transform(transform),
					collider: Collider::ball(VOID_SPHERE_OUTER_RADIUS),
					..default()
				},
				ColliderRoot(parent.parent_entity()),
			));
		});

		Ok(())
	}
}
