use super::{
	simple_collidable::SimpleCollidablePrefab,
	sphere,
	AssetKey,
	CreatePrefab,
	Instantiate,
	ProjectileType,
};
use crate::{
	bundles::ColliderBundle,
	components::{ColliderRoot, Plasma, Projectile},
	errors::Error,
	resources::Prefab,
};
use bevy::{
	asset::{Assets, Handle},
	ecs::system::ResMut,
	hierarchy::BuildChildren,
	math::Vec3,
	pbr::{PbrBundle, StandardMaterial},
	render::{color::Color, mesh::Mesh},
	transform::components::Transform,
	utils::default,
};
use bevy_rapier3d::{
	dynamics::RigidBody,
	geometry::{Collider, Sensor},
};

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

impl Instantiate for Projectile<Plasma> {
	fn instantiate(
		on: &mut bevy::ecs::system::EntityCommands,
		mut get_mesh_handle: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
		mut get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
	) -> Result<(), Error> {
		let key = AssetKey::Projectile(ProjectileType::Plasma);
		let transform = Transform::from_translation(Vec3::ZERO);
		let mesh = sphere(PLASMA_RADIUS, || "Cannot create plasma projectile")?;
		let material = StandardMaterial {
			emissive: Color::rgb_linear(2.0, 13.99, 13.99),
			..default()
		};

		on.insert(RigidBody::Fixed).with_children(|parent| {
			parent.spawn(PbrBundle {
				transform,
				mesh: get_mesh_handle(key, mesh),
				material: get_material_handle(key, material),
				..default()
			});
			parent.spawn((
				ColliderBundle::new_static_collider(transform, Collider::ball(PLASMA_RADIUS)),
				Sensor,
				ColliderRoot(parent.parent_entity()),
			));
		});

		Ok(())
	}
}
