use crate::components::{Beam, Force};
use bevy::{
	ecs::system::EntityCommands,
	math::{primitives::Cylinder, Quat},
	pbr::{NotShadowCaster, PbrBundle, StandardMaterial},
	prelude::ChildBuilder,
	render::{alpha::AlphaMode, mesh::Mesh},
	transform::components::Transform,
	utils::default,
};
use common::{
	components::{PhysicalEntity, Wall},
	errors::Error,
	traits::cache::GetOrCreateTypeAsset,
};
use interactions::{
	components::{is::Is, DealsDamage, InitDelay, Repeat},
	traits::ConcatBlockers,
};
use prefabs::{
	components::WithChildren,
	traits::{GetOrCreateAssets, Instantiate},
};
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
		let render = move |parent: &mut ChildBuilder| {
			parent.spawn((
				PbrBundle {
					material: material.clone(),
					mesh: mesh.clone(),
					transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.)),
					..default()
				},
				NotShadowCaster,
			));
		};

		on.try_insert((
			Is::beam()
				.blocked_by::<PhysicalEntity>()
				.and::<Wall>()
				.and::<Force>(),
			DealsDamage(self.damage)
				.after(Duration::from_millis(100))
				.repeat(),
			WithChildren::delayed(render),
		));

		Ok(())
	}
}
