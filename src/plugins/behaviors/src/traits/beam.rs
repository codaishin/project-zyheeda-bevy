use crate::components::Beam;
use bevy::{
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::{primitives::Cylinder, Quat},
	pbr::{NotShadowCaster, PbrBundle, StandardMaterial},
	render::{alpha::AlphaMode, mesh::Mesh},
	transform::components::Transform,
	utils::default,
};
use common::{errors::Error, traits::cache::GetOrCreateTypeAsset};
use interactions::components::{DealsDamage, InitDelay, Repeat};
use prefabs::traits::{GetOrCreateAssets, Instantiate};
use std::{f32::consts::PI, time::Duration};

impl Instantiate for Beam {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let mesh = assets.get_or_create_for::<Beam>(|| {
			Mesh::from(Cylinder {
				radius: 0.01,
				half_height: 0.5,
			})
		});
		let material = assets.get_or_create_for::<Beam>(|| StandardMaterial {
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
