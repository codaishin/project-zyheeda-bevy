use super::{
	complex_collidable::ComplexCollidablePrefab,
	sphere,
	AssetKey,
	CreatePrefab,
	Instantiate,
	VoidPart,
};
use crate::{
	bundles::ColliderBundle,
	components::{ColliderRoot, UnitsPerSecond, VoidSphere, VoidSpherePart},
	errors::Error,
};
use bevy::{
	asset::{Assets, Handle},
	ecs::{bundle::Bundle, system::ResMut},
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
use bevy_rapier3d::geometry::Collider;
use std::f32::consts::PI;

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

pub type VoidSpherePrefab = ComplexCollidablePrefab<VoidSphere, (), PbrVoidSphereBundle, 3>;

const VOID_SPHERE_INNER_RADIUS: f32 = 0.3;
const VOID_SPHERE_OUTER_RADIUS: f32 = 0.4;
const VOID_SPHERE_TORUS_RADIUS: f32 = 0.35;
const VOID_SPHERE_TORUS_RING_RADIUS: f32 = VOID_SPHERE_OUTER_RADIUS - VOID_SPHERE_TORUS_RADIUS;
const VOID_SPHERE_GROUND_OFFSET: Vec3 = Vec3::new(0., 1.2, 0.);

impl CreatePrefab<VoidSpherePrefab> for VoidSphere {
	fn create_prefab(
		mut materials: ResMut<Assets<StandardMaterial>>,
		mut meshes: ResMut<Assets<Mesh>>,
	) -> Result<VoidSpherePrefab, Error> {
		let core_material = materials.add(StandardMaterial {
			base_color: Color::BLACK,
			metallic: 1.,
			..default()
		});
		let core_mesh = meshes.add(sphere(VOID_SPHERE_INNER_RADIUS, || {
			"Cannot create void sphere core"
		})?);
		let torus_material = materials.add(StandardMaterial {
			emissive: Color::rgb_linear(13.99, 13.99, 13.99),
			..default()
		});
		let torus_mesh = meshes.add(Mesh::from(Torus {
			radius: VOID_SPHERE_TORUS_RADIUS,
			ring_radius: VOID_SPHERE_TORUS_RING_RADIUS,
			..default()
		}));
		let transform = Transform::from_translation(VOID_SPHERE_GROUND_OFFSET);
		let mut transform_2nd_ring = transform;
		transform_2nd_ring.rotate_axis(Vec3::Z, PI / 2.);

		Ok(VoidSpherePrefab::new(
			(),
			(
				[
					PbrVoidSphereBundle::new(
						PbrBundle {
							mesh: core_mesh,
							material: core_material,
							transform,
							..default()
						},
						VoidSpherePart::Core,
					),
					PbrVoidSphereBundle::new(
						PbrBundle {
							mesh: torus_mesh.clone(),
							material: torus_material.clone(),
							transform,
							..default()
						},
						VoidSpherePart::RingA(UnitsPerSecond::new(PI / 50.)),
					),
					PbrVoidSphereBundle::new(
						PbrBundle {
							mesh: torus_mesh.clone(),
							material: torus_material.clone(),
							transform: transform_2nd_ring,
							..default()
						},
						VoidSpherePart::RingB(UnitsPerSecond::new(PI / 75.)),
					),
				],
				ColliderBundle {
					transform: TransformBundle::from_transform(transform),
					collider: Collider::ball(VOID_SPHERE_OUTER_RADIUS),
					..default()
				},
			),
		))
	}
}

impl Instantiate for VoidSphere {
	fn instantiate(
		on: &mut bevy::ecs::system::EntityCommands,
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