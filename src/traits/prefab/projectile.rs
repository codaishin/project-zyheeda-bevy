use super::{simple_collidable::SimpleCollidablePrefab, sphere, CreatePrefab};
use crate::{
	bundles::ColliderBundle,
	components::{Plasma, Projectile},
	errors::Error,
	resources::Prefab,
};
use bevy::{
	asset::Assets,
	ecs::system::ResMut,
	math::Vec3,
	pbr::{PbrBundle, StandardMaterial},
	render::{color::Color, mesh::Mesh},
	transform::components::Transform,
	utils::default,
};
use bevy_rapier3d::{dynamics::RigidBody, geometry::Collider};

pub type ProjectilePrefab<T> = SimpleCollidablePrefab<Projectile<T>, RigidBody, PbrBundle>;

const PLASMA_RADIUS: f32 = 0.05;

impl CreatePrefab<ProjectilePrefab<Plasma>, StandardMaterial> for Projectile<Plasma> {
	fn create_prefab(
		mut materials: ResMut<Assets<StandardMaterial>>,
		mut meshes: ResMut<Assets<Mesh>>,
	) -> Result<ProjectilePrefab<Plasma>, Error> {
		let transform = Transform::from_translation(Vec3::ZERO);

		Ok(Prefab::new(
			RigidBody::Fixed,
			(
				[PbrBundle {
					transform,
					material: materials.add(StandardMaterial {
						emissive: Color::rgb_linear(2.0, 13.99, 13.99),
						..default()
					}),
					mesh: meshes.add(sphere(PLASMA_RADIUS, || "Cannot create plasma projectile")?),
					..default()
				}],
				ColliderBundle::new_static_collider(transform, Collider::ball(PLASMA_RADIUS)),
			),
		))
	}
}
