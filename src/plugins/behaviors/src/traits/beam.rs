use crate::components::Beam;
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
	errors::Error,
	traits::{
		cache::GetOrCreateTypeAsset,
		prefab::{AfterInstantiation, GetOrCreateAssets, Instantiate},
	},
};
use interactions::components::{
	blocker::Blocker,
	deals_damage::DealsDamage,
	is::{InterruptableRay, Is},
};
use std::f32::consts::PI;

impl Instantiate for Beam {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error>
	where
		TAfterInstantiation: AfterInstantiation,
	{
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

		entity.try_insert((
			Is::<InterruptableRay>::interacting_with([Blocker::Physical, Blocker::Force]),
			DealsDamage::once_per_second(self.damage),
			TAfterInstantiation::spawn(move |parent: &mut ChildBuilder| {
				parent.spawn((
					PbrBundle {
						material: material.clone(),
						mesh: mesh.clone(),
						transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.)),
						..default()
					},
					NotShadowCaster,
				));
			}),
		));

		Ok(())
	}
}
