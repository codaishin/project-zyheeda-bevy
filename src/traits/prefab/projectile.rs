use super::{flat_model::FlatPrefab, CreatePrefab};
use crate::{
	bundles::ColliderBundle,
	components::{Plasma, Projectile},
	errors::{Error, Level},
	resources::Prefab,
};
use bevy::{
	asset::Assets,
	ecs::system::ResMut,
	math::Vec3,
	pbr::{PbrBundle, StandardMaterial},
	render::{
		color::Color,
		mesh::{shape::Icosphere, Mesh},
	},
	transform::components::Transform,
	utils::default,
};
use bevy_rapier3d::{
	dynamics::RigidBody,
	geometry::{Collider, Sensor},
};

macro_rules! projectile_error {
	($t:expr, $e:expr) => {
		format!("Cannot create prefab for projectile ({}): {}", $t, $e)
	};
}

fn sphere(type_name: &'static str, radius: f32) -> Result<Mesh, Error> {
	Mesh::try_from(Icosphere {
		radius,
		subdivisions: 5,
	})
	.map_err(|err| Error {
		lvl: Level::Error,
		msg: projectile_error!(type_name, err),
	})
}

const PLASMA_RADIUS: f32 = 0.05;

impl CreatePrefab<FlatPrefab<RigidBody, Sensor>, StandardMaterial> for Projectile<Plasma> {
	fn create_prefab(
		mut materials: ResMut<Assets<StandardMaterial>>,
		mut meshes: ResMut<Assets<Mesh>>,
	) -> Result<FlatPrefab<RigidBody, Sensor>, Error> {
		let transform = Transform::from_translation(Vec3::ZERO);

		Ok(Prefab {
			parent: RigidBody::Fixed,
			children: (
				PbrBundle {
					transform,
					material: materials.add(StandardMaterial {
						emissive: Color::rgb_linear(2.0, 13.99, 13.99),
						..default()
					}),
					mesh: meshes.add(sphere("plasma", PLASMA_RADIUS)?),
					..default()
				},
				ColliderBundle::new_static_collider(transform, Collider::ball(PLASMA_RADIUS)),
			),
		})
	}
}
