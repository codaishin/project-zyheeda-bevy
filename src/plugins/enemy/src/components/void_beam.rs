use bevy::{ecs::system::EntityCommands, pbr::NotShadowCaster, prelude::*};
use common::{
	blocker::Blocker,
	effects::deal_damage::DealDamage,
	errors::Error,
	tools::Units,
	traits::{
		cache::GetOrCreateTypeAsset,
		handles_effect::HandlesEffect,
		handles_enemies::{Attacker, Target},
		handles_interactions::{BeamParameters, HandlesInteractions},
		prefab::{AfterInstantiation, GetOrCreateAssets, Prefab},
	},
};
use std::{f32::consts::PI, time::Duration};

use crate::traits::insert_attack::InsertAttack;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct VoidBeam {
	attack: VoidBeamAttack,
	attacker: Entity,
	target: Entity,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct VoidBeamAttack {
	pub damage: f32,
	pub color: Color,
	pub emissive: LinearRgba,
	pub lifetime: Duration,
	pub range: Units,
}

impl BeamParameters for VoidBeam {
	fn source(&self) -> Entity {
		self.attacker
	}

	fn target(&self) -> Entity {
		self.target
	}

	fn range(&self) -> Units {
		self.attack.range
	}

	fn lifetime(&self) -> Duration {
		self.attack.lifetime
	}
}

impl<TInteractions> Prefab<TInteractions> for VoidBeam
where
	TInteractions: HandlesInteractions + HandlesEffect<DealDamage>,
{
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error>
	where
		TAfterInstantiation: AfterInstantiation,
	{
		let mesh = assets.get_or_create_for::<VoidBeam>(|| {
			Mesh::from(Cylinder {
				radius: 0.01,
				half_height: 0.5,
			})
		});
		let material = assets.get_or_create_for::<VoidBeam>(|| StandardMaterial {
			base_color: self.attack.color,
			emissive: self.attack.emissive,
			alpha_mode: AlphaMode::Add,
			..default()
		});

		entity.try_insert((
			TInteractions::beam_from(self),
			TInteractions::is_ray_interrupted_by(&[Blocker::Physical, Blocker::Force]),
			TInteractions::effect(DealDamage::once_per_second(self.attack.damage)),
			TAfterInstantiation::spawn(move |parent: &mut ChildBuilder| {
				parent.spawn((
					Mesh3d(mesh.clone()),
					MeshMaterial3d(material.clone()),
					Transform::from_rotation(Quat::from_rotation_x(PI / 2.)),
					NotShadowCaster,
				));
			}),
		));

		Ok(())
	}
}

impl InsertAttack for VoidBeamAttack {
	fn insert_attack(
		&self,
		entity: &mut EntityCommands,
		Attacker(attacker): Attacker,
		Target(target): Target,
	) {
		entity.insert(VoidBeam {
			attack: *self,
			attacker,
			target,
		});
	}
}
