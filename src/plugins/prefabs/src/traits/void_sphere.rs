use super::{sphere, AssetKey, Instantiate, VoidPart};
use bars::components::Bar;
use behaviors::components::{AttackConfig, Beam, Enemy, Foe, MovementConfig, MovementMode};
use bevy::{
	asset::Handle,
	ecs::{bundle::Bundle, system::EntityCommands},
	hierarchy::BuildChildren,
	math::Vec3,
	pbr::{NotShadowCaster, PbrBundle, StandardMaterial},
	render::{
		color::Color,
		mesh::{shape::Torus, Mesh},
	},
	transform::{components::Transform, TransformBundle},
	utils::default,
};
use bevy_rapier3d::{
	dynamics::{GravityScale, RigidBody},
	geometry::Collider,
};
use common::{
	bundles::ColliderBundle,
	components::{ColliderRoot, Health, VoidSphere, VoidSpherePart},
	errors::Error,
	tools::UnitsPerSecond,
};
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

impl Instantiate for VoidSphere {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut get_mesh_handle: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
		mut get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
	) -> Result<(), Error> {
		let core = AssetKey::VoidSphere(VoidPart::Core);
		let ring = AssetKey::VoidSphere(VoidPart::Ring);
		let core_material = StandardMaterial {
			base_color: Color::BLACK,
			metallic: 1.,
			..default()
		};
		let core_mesh = sphere(VOID_SPHERE_INNER_RADIUS, || {
			"Cannot create void sphere core"
		})?;
		let ring_material = StandardMaterial {
			emissive: Color::rgb_linear(13.99, 13.99, 13.99),
			..default()
		};
		let ring_mesh = Mesh::from(Torus {
			radius: VOID_SPHERE_TORUS_RADIUS,
			ring_radius: VOID_SPHERE_TORUS_RING_RADIUS,
			..default()
		});
		let transform = Transform::from_translation(VOID_SPHERE_GROUND_OFFSET);
		let mut transform_2nd_ring = transform;
		transform_2nd_ring.rotate_axis(Vec3::Z, PI / 2.);

		on.insert((
			RigidBody::Dynamic,
			GravityScale(0.),
			Health::new(5),
			Bar::default(),
			MovementConfig::Constant {
				mode: MovementMode::Slow,
				speed: UnitsPerSecond::new(1.),
			},
			AttackConfig {
				attack: Beam::attack,
				cool_down: Duration::from_secs(2),
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
					mesh: get_mesh_handle(core, core_mesh),
					material: get_material_handle(core, core_material),
					transform,
					..default()
				},
				VoidSpherePart::Core,
			));
			parent.spawn(PbrVoidSphereBundle::new(
				PbrBundle {
					mesh: get_mesh_handle(ring, ring_mesh.clone()),
					material: get_material_handle(ring, ring_material.clone()),
					transform,
					..default()
				},
				VoidSpherePart::RingA(UnitsPerSecond::new(PI / 50.)),
			));
			parent.spawn(PbrVoidSphereBundle::new(
				PbrBundle {
					mesh: get_mesh_handle(ring, ring_mesh),
					material: get_material_handle(ring, ring_material),
					transform: transform_2nd_ring,
					..default()
				},
				VoidSpherePart::RingB(UnitsPerSecond::new(PI / 75.)),
			));
			parent.spawn((
				ColliderBundle {
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
