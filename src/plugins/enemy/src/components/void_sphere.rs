use behaviors::{
	components::{
		void_beam::VoidBeamAttack,
		AttackConfig,
		Enemy,
		Foe,
		MovementConfig,
		MovementMode,
	},
	traits::ToArc,
};
use bevy::{
	color::{Color, LinearRgba},
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::{primitives::Torus, Dir3, Vec3},
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
	attributes::{
		affected_by::{Affected, AffectedBy},
		health::Health,
	},
	blocker::Blocker,
	components::{ColliderRoot, GroundOffset},
	effects::{deal_damage::DealDamage, gravity::Gravity},
	errors::Error,
	tools::{Units, UnitsPerSecond},
	traits::{
		cache::GetOrCreateTypeAsset,
		clamp_zero_positive::ClampZeroPositive,
		handles_bars::HandlesBars,
		handles_effect::HandlesEffect,
		prefab::{sphere, GetOrCreateAssets, Prefab},
	},
};
use std::{f32::consts::PI, time::Duration};

#[derive(Component)]
pub struct VoidSphere;

impl VoidSphere {
	pub fn aggro_range() -> Units {
		Units::new(10.)
	}
	pub fn attack_range() -> Units {
		Units::new(5.)
	}
}

#[derive(Component, Clone)]
#[require(Mesh3d, MeshMaterial3d<StandardMaterial>, NotShadowCaster)]
pub enum VoidSpherePart {
	Core,
	RingA(UnitsPerSecond),
	RingB(UnitsPerSecond),
}

const VOID_SPHERE_INNER_RADIUS: f32 = 0.3;
const VOID_SPHERE_OUTER_RADIUS: f32 = 0.4;
const VOID_SPHERE_TORUS_RADIUS: f32 = 0.35;
const VOID_SPHERE_TORUS_RING_RADIUS: f32 = VOID_SPHERE_OUTER_RADIUS - VOID_SPHERE_TORUS_RADIUS;
const VOID_SPHERE_GROUND_OFFSET: Vec3 = Vec3::new(0., 1.2, 0.);

struct VoidSphereCore;

struct VoidSphereRing;

impl<TInteractions, TBars> Prefab<(TInteractions, TBars)> for VoidSphere
where
	TInteractions: HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
	TBars: HandlesBars,
{
	fn instantiate_on<TAfterInstantiation>(
		&self,
		on: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let core_material = assets.get_or_create_for::<VoidSphereCore>(|| StandardMaterial {
			base_color: Color::BLACK,
			metallic: 1.,
			..default()
		});
		let core_mesh =
			assets.get_or_create_for::<VoidSphereCore>(|| sphere(VOID_SPHERE_INNER_RADIUS));
		let ring_material = assets.get_or_create_for::<VoidSphereRing>(|| StandardMaterial {
			emissive: LinearRgba::new(23.0, 23.0, 23.0, 1.),
			..default()
		});
		let ring_mesh = assets.get_or_create_for::<VoidSphereRing>(|| {
			Mesh::from(Torus {
				major_radius: VOID_SPHERE_TORUS_RADIUS,
				minor_radius: VOID_SPHERE_TORUS_RING_RADIUS,
			})
		});
		let transform = Transform::from_translation(VOID_SPHERE_GROUND_OFFSET);
		let mut transform_2nd_ring = transform;
		transform_2nd_ring.rotate_axis(Dir3::Z, PI / 2.);

		on.try_insert((
			Blocker::insert([Blocker::Physical]),
			GroundOffset(VOID_SPHERE_GROUND_OFFSET),
			RigidBody::Dynamic,
			GravityScale(0.),
			Health::new(5.).bundle_via::<TInteractions>(),
			Affected::by::<Gravity>().bundle_via::<TInteractions>(),
			TBars::new_bar(),
			MovementConfig::Constant {
				mode: MovementMode::Slow,
				speed: UnitsPerSecond::new(1.),
			},
			AttackConfig {
				spawn: VoidBeamAttack {
					damage: 10.,
					color: Color::BLACK,
					emissive: LinearRgba::new(23.0, 23.0, 23.0, 1.),
					lifetime: Duration::from_secs(1),
					range: VoidSphere::attack_range(),
				}
				.to_arc(),
				cool_down: Duration::from_secs(5),
			},
			Enemy {
				aggro_range: VoidSphere::aggro_range(),
				attack_range: VoidSphere::attack_range(),
				foe: Foe::Player,
			},
		));
		on.with_children(|parent| {
			parent.spawn((
				VoidSpherePart::Core,
				Mesh3d(core_mesh),
				MeshMaterial3d(core_material),
				transform,
			));
			parent.spawn((
				VoidSpherePart::RingA(UnitsPerSecond::new(PI / 50.)),
				Mesh3d(ring_mesh.clone()),
				MeshMaterial3d(ring_material.clone()),
				transform,
			));
			parent.spawn((
				VoidSpherePart::RingB(UnitsPerSecond::new(PI / 75.)),
				Mesh3d(ring_mesh),
				MeshMaterial3d(ring_material),
				transform_2nd_ring,
			));
			parent.spawn((
				ColliderRoot(parent.parent_entity()),
				Collider::ball(VOID_SPHERE_OUTER_RADIUS),
				transform,
			));
		});

		Ok(())
	}
}
