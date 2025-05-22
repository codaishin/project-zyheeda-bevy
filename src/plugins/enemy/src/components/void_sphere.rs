use super::{enemy::Enemy, void_beam::VoidBeamAttack};
use bevy::{
	color::{Color, LinearRgba},
	ecs::relationship::RelatedSpawnerCommands,
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
	blocker::{Blocker, BlockerInsertCommand},
	components::{
		GroundOffset,
		collider_relationship::InteractionTarget,
		insert_asset::InsertAsset,
		spawn_children::SpawnChildren,
	},
	effects::{deal_damage::DealDamage, gravity::Gravity},
	errors::Error,
	tools::{Units, UnitsPerSecond, collider_radius::ColliderRadius},
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_effect::HandlesEffect,
		handles_enemies::EnemyTarget,
		prefab::{Prefab, sphere},
	},
};
use std::{f32::consts::PI, sync::Arc, time::Duration};

#[derive(Component)]
#[require(
	Enemy = VoidSphere::as_enemy(),
	BlockerInsertCommand = Self::blockers(),
	GroundOffset = Self::ground_offset(),
	RigidBody = Self::rigid_body(),
	GravityScale = Self::gravity_scale(),
	SpawnChildren(Self::void_sphere_parts),
	InteractionTarget,
)]
pub struct VoidSphere;

impl VoidSphere {
	fn collider_radius() -> ColliderRadius {
		ColliderRadius(Units::new(VOID_SPHERE_OUTER_RADIUS))
	}

	const fn ground_offset() -> Vec3 {
		Vec3::new(0., 1.2, 0.)
	}

	fn blockers() -> BlockerInsertCommand {
		Blocker::insert([Blocker::Physical])
	}

	fn rigid_body() -> RigidBody {
		RigidBody::Dynamic
	}

	fn gravity_scale() -> GravityScale {
		GravityScale(0.)
	}

	fn void_sphere_parts(parent: &mut RelatedSpawnerCommands<ChildOf>) {
		let transform = Transform::from_translation(Self::ground_offset());
		let mut transform_2nd_ring = transform;
		transform_2nd_ring.rotate_axis(Dir3::Z, PI / 2.);

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
		parent.spawn((Collider::ball(VOID_SPHERE_OUTER_RADIUS), transform));
	}

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
	fn instantiate_on(&self, on: &mut EntityCommands) -> Result<(), Error> {
		let transform = Transform::from_translation(Self::ground_offset());
		let mut transform_2nd_ring = transform;
		transform_2nd_ring.rotate_axis(Dir3::Z, PI / 2.);

		on.try_insert((
			Health::new(5.).bundle_via::<TInteractions>(),
			Affected::by::<Gravity>().bundle_via::<TInteractions>(),
		));

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
		InsertAsset::shared::<Self>(|| sphere(VOID_SPHERE_INNER_RADIUS))
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
