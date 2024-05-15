use crate::components::ForceShield;
use bevy::{
	asset::Handle,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::{primitives::Cuboid, Vec3},
	pbr::{AlphaMode, PbrBundle, StandardMaterial},
	render::{color::Color, mesh::Mesh},
	transform::TransformBundle,
	utils::default,
};
use bevy_rapier3d::{dynamics::RigidBody, geometry::Collider};
use common::{bundles::ColliderBundle, errors::Error};
use prefabs::traits::{AssetKey, Instantiate};

impl Instantiate for ForceShield {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut get_mesh_handle: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
		mut get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
	) -> Result<(), Error> {
		let half_size = Vec3 {
			x: 0.6,
			y: 0.6,
			z: 0.01,
		};
		let base_color = Color::MIDNIGHT_BLUE;
		let emissive = base_color * 100.;
		let material = get_material_handle(
			AssetKey::ForceShield,
			StandardMaterial {
				base_color,
				emissive,
				alpha_mode: AlphaMode::Add,
				..default()
			},
		);
		let mesh = get_mesh_handle(AssetKey::ForceShield, Mesh::from(Cuboid { half_size }));

		on.insert((RigidBody::Fixed, TransformBundle::default()))
			.with_children(|parent| {
				parent.spawn((
					PbrBundle {
						mesh,
						material,
						..default()
					},
					ColliderBundle::new_static_collider(Collider::cuboid(
						half_size.x,
						half_size.y,
						half_size.z,
					)),
				));
			});

		Ok(())
	}
}