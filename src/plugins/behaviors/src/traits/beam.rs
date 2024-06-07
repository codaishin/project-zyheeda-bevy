use crate::components::Beam;
use bevy::{
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
use prefabs::traits::{AssetHandles, Instantiate};
use std::{f32::consts::PI, time::Duration};

impl Instantiate for Beam {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut assets: impl AssetHandles,
	) -> Result<(), Error> {
		let mesh = assets.handle::<Beam>(&mut || {
			Mesh::from(Cylinder {
				radius: 0.01,
				half_height: 0.5,
			})
		});
		let material = assets.handle::<Beam>(&mut || StandardMaterial {
			base_color: self.color,
			emissive: self.emissive,
			alpha_mode: AlphaMode::Add,
			..default()
		});

		on.try_insert(
			DealsDamage(self.damage)
				.after(Duration::from_millis(100))
				.repeat(),
		)
		.with_children(|parent| {
			parent.spawn((
				PbrBundle {
					material,
					mesh,
					transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.)),
					..default()
				},
				NotShadowCaster,
			));
		});

		Ok(())
	}
}
