use crate::components::ForceShield;
use bevy::{
	color::LinearRgba,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::{primitives::Cuboid, Vec3},
	pbr::{PbrBundle, StandardMaterial},
	render::{alpha::AlphaMode, mesh::Mesh},
	transform::bundles::TransformBundle,
	utils::default,
};
use bevy_rapier3d::{dynamics::RigidBody, geometry::Collider};
use common::{bundles::ColliderBundle, errors::Error, traits::cache::GetOrCreateTypeAsset};
use prefabs::traits::{GetOrCreateAssets, Instantiate};

impl Instantiate for ForceShield {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let half_size = Vec3 {
			x: 0.6,
			y: 0.6,
			z: 0.01,
		};
		let base_color = LinearRgba::new(0.1, 0.1, 0.44, 1.);
		let emissive = base_color * 100.;
		let material = assets.get_or_create_for::<ForceShield>(|| StandardMaterial {
			base_color: base_color.into(),
			emissive,
			alpha_mode: AlphaMode::Add,
			..default()
		});
		let mesh = assets.get_or_create_for::<ForceShield>(|| Mesh::from(Cuboid { half_size }));

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
