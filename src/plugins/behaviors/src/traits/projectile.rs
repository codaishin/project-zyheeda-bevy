use super::ProjectileBehavior;
use crate::components::{MovementConfig, MovementMode, Plasma, Projectile};
use bevy::{
	self,
	asset::Handle,
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
use common::{
	bundles::ColliderBundle,
	components::ColliderRoot,
	errors::Error,
	tools::UnitsPerSecond,
};
use interactions::components::{DealsDamage, Fragile};
use prefabs::traits::{sphere, AssetKey, Instantiate, ProjectileType};

impl<T> ProjectileBehavior for Projectile<T> {
	fn direction(&self) -> Vec3 {
		self.direction
	}

	fn range(&self) -> f32 {
		self.range
	}
}

const PLASMA_RADIUS: f32 = 0.05;

impl Instantiate for Projectile<Plasma> {
	fn instantiate(
		&self,
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

		on.insert((
			RigidBody::Fixed,
			DealsDamage(1),
			Fragile,
			MovementConfig::Constant {
				mode: MovementMode::Fast,
				speed: UnitsPerSecond::new(15.),
			},
		))
		.with_children(|parent| {
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
