use crate::{
	components::{Attacker, Target},
	traits::{DespawnFn, SpawnAttack},
};
use bevy::{ecs::system::EntityCommands, pbr::NotShadowCaster, prelude::*};
use common::{
	blocker::Blocker,
	errors::Error,
	tools::Units,
	traits::{
		cache::GetOrCreateTypeAsset,
		handles_beams::{BeamParameters, HandlesBeams},
		prefab::{AfterInstantiation, GetOrCreateAssets, Prefab},
		try_despawn_recursive::TryDespawnRecursive,
	},
};
use interactions::components::{
	deals_damage::DealsDamage,
	is::{InterruptableRay, Is},
};
use std::{f32::consts::PI, sync::Arc, time::Duration};

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
	TInteractions: HandlesBeams,
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
			Is::<InterruptableRay>::interacting_with([Blocker::Physical, Blocker::Force]),
			DealsDamage::once_per_second(self.attack.damage),
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

impl SpawnAttack for VoidBeamAttack {
	fn spawn(
		&self,
		commands: &mut Commands,
		Attacker(attacker): Attacker,
		Target(target): Target,
	) -> DespawnFn {
		despawn(
			commands
				.spawn(VoidBeam {
					attack: *self,
					attacker,
					target,
				})
				.id(),
		)
	}
}

fn despawn(entity: Entity) -> DespawnFn {
	Arc::new(move |commands| {
		commands.try_despawn_recursive(entity);
	})
}
