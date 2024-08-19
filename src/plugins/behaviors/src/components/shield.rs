use bevy::{
	color::Color,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::{primitives::Cuboid, Vec3},
	pbr::{PbrBundle, StandardMaterial},
	prelude::{Component, Entity},
	render::{alpha::AlphaMode, mesh::Mesh},
	transform::bundles::TransformBundle,
	utils::default,
};
use bevy_rapier3d::{dynamics::RigidBody, geometry::Collider};
use common::{components::ColliderRoot, errors::Error, traits::cache::GetOrCreateTypeAsset};
use prefabs::traits::{GetOrCreateAssets, Instantiate};

#[derive(Component, Debug, PartialEq)]
pub struct Shield {
	pub location: Entity,
}

impl Instantiate for Shield {
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
		let base_color = Color::srgb(0.1, 0.1, 0.44);
		let emissive = base_color.to_linear() * 100.;
		let material = assets.get_or_create_for::<Shield>(|| StandardMaterial {
			base_color,
			emissive,
			alpha_mode: AlphaMode::Add,
			..default()
		});
		let mesh = assets.get_or_create_for::<Shield>(|| Mesh::from(Cuboid { half_size }));

		on.insert((RigidBody::Fixed, TransformBundle::default()))
			.with_children(|parent| {
				parent.spawn((
					PbrBundle {
						mesh,
						material,
						..default()
					},
					Collider::cuboid(half_size.x, half_size.y, half_size.z),
					ColliderRoot(parent.parent_entity()),
				));
			});

		Ok(())
	}
}
