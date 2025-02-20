use super::{enemy::Enemy, void_beam::VoidBeamAttack};
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
	components::{asset_component::AssetComponent, ColliderRoot, GroundOffset},
	effects::{deal_damage::DealDamage, gravity::Gravity},
	errors::Error,
	tools::{Units, UnitsPerSecond},
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_effect::HandlesEffect,
		handles_enemies::EnemyTarget,
		prefab::{sphere, Prefab},
	},
};
use std::{f32::consts::PI, sync::Arc, time::Duration};

#[derive(Component)]
#[require(Enemy(VoidSphere::as_enemy))]
pub struct VoidSphere;

impl VoidSphere {
	fn as_enemy() -> Enemy {
		let attack_range = Units::new(5.);

		Enemy {
			speed: UnitsPerSecond::new(1.).into(),
			movement_animation: None,
			aggro_range: Units::new(10.).into(),
			attack_range: attack_range.into(),
			target: EnemyTarget::Player,
			attack: Arc::new(VoidBeamAttack {
				damage: 10.,
				color: Color::BLACK,
				emissive: LinearRgba::new(23.0, 23.0, 23.0, 1.),
				lifetime: Duration::from_secs(1),
				range: attack_range,
			}),
			cool_down: Duration::from_secs(5),
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

const VOID_SPHERE_INNER_RADIUS: f32 = 0.3;
const VOID_SPHERE_OUTER_RADIUS: f32 = 0.4;
const VOID_SPHERE_TORUS_RADIUS: f32 = 0.35;
const VOID_SPHERE_TORUS_RING_RADIUS: f32 = VOID_SPHERE_OUTER_RADIUS - VOID_SPHERE_TORUS_RADIUS;
const VOID_SPHERE_GROUND_OFFSET: Vec3 = Vec3::new(0., 1.2, 0.);

impl<TInteractions> Prefab<TInteractions> for VoidSphere
where
	TInteractions: HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
{
	fn instantiate_on<TAfterInstantiation>(&self, on: &mut EntityCommands) -> Result<(), Error> {
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
		));
		on.with_children(|parent| {
			parent.spawn((VoidSpherePart::Core, VoidSphereCore, transform));
			parent.spawn((
				VoidSpherePart::RingA(UnitsPerSecond::new(PI / 50.)),
				VoidSphereRing,
				transform,
			));
			parent.spawn((
				VoidSpherePart::RingB(UnitsPerSecond::new(PI / 75.)),
				VoidSphereRing,
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

#[derive(Component, Clone)]
#[require(Mesh3d, MeshMaterial3d<StandardMaterial>, NotShadowCaster)]
pub enum VoidSpherePart {
	Core,
	RingA(UnitsPerSecond),
	RingB(UnitsPerSecond),
}

#[derive(Component)]
#[require(
	AssetComponent<StandardMaterial> (Self::material),
	AssetComponent<Mesh> (Self::mesh),
)]
struct VoidSphereCore;

impl VoidSphereCore {
	fn material() -> AssetComponent<StandardMaterial> {
		AssetComponent::shared::<Self>(|| StandardMaterial {
			base_color: Color::BLACK,
			metallic: 1.,
			..default()
		})
	}

	fn mesh() -> AssetComponent<Mesh> {
		AssetComponent::shared::<Self>(|| sphere(VOID_SPHERE_INNER_RADIUS))
	}
}

#[derive(Component)]
#[require(
	AssetComponent<StandardMaterial> (Self::material),
	AssetComponent<Mesh> (Self::mesh),
)]
struct VoidSphereRing;

impl VoidSphereRing {
	fn material() -> AssetComponent<StandardMaterial> {
		AssetComponent::shared::<Self>(|| StandardMaterial {
			emissive: LinearRgba::new(23.0, 23.0, 23.0, 1.),
			..default()
		})
	}

	fn mesh() -> AssetComponent<Mesh> {
		AssetComponent::shared::<Self>(|| {
			Mesh::from(Torus {
				major_radius: VOID_SPHERE_TORUS_RADIUS,
				minor_radius: VOID_SPHERE_TORUS_RING_RADIUS,
			})
		})
	}
}
