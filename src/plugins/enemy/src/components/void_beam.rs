use crate::traits::insert_attack::InsertAttack;
use bevy::{ecs::system::EntityCommands, pbr::NotShadowCaster, prelude::*};
use common::{
	blocker::Blocker,
	components::insert_asset::{InsertAsset, InsertAssetFromSource},
	effects::deal_damage::DealDamage,
	errors::Error,
	tools::Units,
	traits::{
		handles_effect::HandlesEffect,
		handles_enemies::{Attacker, Target},
		handles_interactions::{BeamParameters, HandlesInteractions},
		prefab::Prefab,
	},
};
use std::{f32::consts::PI, time::Duration};

#[derive(Component, Debug, PartialEq)]
#[require(Visibility, Transform)]
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
	fn insert_prefab_components(&self, entity: &mut EntityCommands) -> Result<(), Error> {
		entity
			.try_insert((
				TInteractions::beam_from(self),
				TInteractions::is_ray_interrupted_by(&[Blocker::Physical, Blocker::Force]),
				TInteractions::effect(DealDamage::once_per_second(self.attack.damage)),
			))
			.with_child(VoidBeamModel {
				color: self.attack.color,
				emissive: self.attack.emissive,
			});

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(
	Visibility,
	Transform = Self::transform(),
	InsertAsset::<Mesh> = Self::model(),
	InsertAssetFromSource::<StandardMaterial, Self> = Self::material(),
	NotShadowCaster,
)]
pub(crate) struct VoidBeamModel {
	pub color: Color,
	pub emissive: LinearRgba,
}

impl VoidBeamModel {
	fn transform() -> Transform {
		Transform::from_rotation(Quat::from_rotation_x(PI / 2.))
	}

	fn model() -> InsertAsset<Mesh> {
		InsertAsset::shared::<Self>(|| {
			Mesh::from(Cylinder {
				radius: 0.01,
				half_height: 0.5,
			})
		})
	}

	fn material() -> InsertAssetFromSource<StandardMaterial, Self> {
		InsertAssetFromSource::shared(|model| StandardMaterial {
			base_color: model.color,
			emissive: model.emissive,
			alpha_mode: AlphaMode::Add,
			..default()
		})
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
