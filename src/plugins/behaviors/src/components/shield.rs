use bevy::{
	color::Color,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::{primitives::Cuboid, Vec3},
	pbr::{PbrBundle, StandardMaterial},
	prelude::{Component, Entity, Transform},
	render::{alpha::AlphaMode, mesh::Mesh},
	utils::default,
};
use bevy_rapier3d::{dynamics::RigidBody, geometry::Collider, prelude::Sensor};
use common::{
	bundles::ColliderTransformBundle,
	components::ColliderRoot,
	errors::Error,
	traits::cache::GetOrCreateTypeAsset,
};
use prefabs::traits::{GetOrCreateAssets, Instantiate};

#[derive(Component, Debug, PartialEq)]
pub struct ShieldContact {
	pub location: Entity,
}

impl Instantiate for ShieldContact {
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
		let material = assets.get_or_create_for::<ShieldContact>(|| StandardMaterial {
			base_color,
			emissive,
			alpha_mode: AlphaMode::Add,
			..default()
		});
		let mesh = assets.get_or_create_for::<ShieldContact>(|| Mesh::from(Cuboid { half_size }));

		on.insert((
			RigidBody::Fixed,
			PbrBundle {
				mesh,
				material,
				..default()
			},
		))
		.with_children(|parent| {
			parent.spawn((
				ColliderTransformBundle::new_static_collider(
					default(),
					Collider::cuboid(half_size.x, half_size.y, half_size.z),
				),
				Sensor,
				ColliderRoot(parent.parent_entity()),
			));
		});

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct ShieldProjection;

impl Instantiate for ShieldProjection {
	fn instantiate(&self, on: &mut EntityCommands, _: impl GetOrCreateAssets) -> Result<(), Error> {
		let half_size = Vec3 {
			x: 0.6,
			y: 0.6,
			z: 0.6,
		};
		on.try_insert((
			ColliderTransformBundle::new_static_collider(
				Transform::from_xyz(0., 0., -half_size.z),
				Collider::cuboid(half_size.x, half_size.y, half_size.z),
			),
			Sensor,
		));

		Ok(())
	}
}
