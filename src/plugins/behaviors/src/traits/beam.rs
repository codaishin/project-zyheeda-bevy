use crate::components::Beam;
use bevy::{
	asset::Handle,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::{Quat, Vec3},
	pbr::{AlphaMode, NotShadowCaster, PbrBundle, StandardMaterial},
	prelude::SpatialBundle,
	render::mesh::{shape::Cylinder, Mesh},
	transform::components::Transform,
	utils::default,
};
use common::errors::Error;
use prefabs::traits::{AssetKey, Instantiate};
use std::f32::consts::PI;

impl Instantiate for Beam {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut get_mesh_handle: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
		mut get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
	) -> Result<(), Error> {
		let key = AssetKey::Beam;
		let mesh = Mesh::from(Cylinder {
			radius: 0.01,
			height: 1.,
			..default()
		});
		let material = StandardMaterial {
			base_color: self.color,
			emissive: self.emissive,
			alpha_mode: AlphaMode::Add,
			..default()
		};
		let direction = self.to - self.from;
		let height = direction.length();
		let mut transform =
			Transform::from_translation((self.from + self.to) / 2.).looking_at(self.to, Vec3::Y);
		transform.scale.z = height;

		on.insert(SpatialBundle::from_transform(transform))
			.with_children(|parent| {
				parent.spawn((
					PbrBundle {
						material: get_material_handle(key, material),
						mesh: get_mesh_handle(key, mesh),
						transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.)),
						..default()
					},
					NotShadowCaster,
				));
			});

		Ok(())
	}
}
