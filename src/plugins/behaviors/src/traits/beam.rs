use crate::components::Beam;
use bevy::{
	asset::Handle,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::{primitives::Cylinder, Quat},
	pbr::{AlphaMode, NotShadowCaster, PbrBundle, StandardMaterial},
	render::mesh::Mesh,
	transform::components::Transform,
	utils::default,
};
use common::errors::Error;
use interactions::components::{DealsDamage, InitDelay, Repeat};
use prefabs::traits::{AssetKey, Instantiate};
use std::{f32::consts::PI, time::Duration};

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
			half_height: 0.5,
		});
		let material = StandardMaterial {
			base_color: self.color,
			emissive: self.emissive,
			alpha_mode: AlphaMode::Add,
			..default()
		};

		on.try_insert(
			DealsDamage(self.damage)
				.after(Duration::from_millis(100))
				.repeat(),
		)
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
