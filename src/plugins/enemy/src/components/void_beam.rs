use crate::traits::insert_attack::InsertAttack;
use bevy::{ecs::system::EntityCommands, pbr::NotShadowCaster, prelude::*};
use common::{
	blocker::Blocker,
	components::{insert_asset::InsertAsset, persistent_entity::PersistentEntity},
	effects::deal_damage::DealDamage,
	errors::Error,
	tools::Units,
	traits::{
		handles_effect::HandlesEffect,
		handles_enemies::{Attacker, Target},
		handles_interactions::{BeamParameters, HandlesInteractions},
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, time::Duration};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(PersistentEntity, Visibility, Transform)]
pub(crate) struct VoidBeam {
	attack: VoidBeamAttack,
	attacker: PersistentEntity,
	target: PersistentEntity,
}

impl BeamParameters for VoidBeam {
	fn source(&self) -> PersistentEntity {
		self.attacker
	}

	fn target(&self) -> PersistentEntity {
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
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Error> {
		entity
			.try_insert_if_new((
				TInteractions::beam_from(self),
				TInteractions::is_ray_interrupted_by(Blocker::all()),
				TInteractions::effect(DealDamage::once_per_second(self.attack.damage)),
			))
			.with_child(VoidBeamModel);

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(
	Visibility,
	Transform = Self::transform(),
	InsertAsset::<Mesh> = Self::model(),
	InsertAsset::<StandardMaterial> = Self::material(),
	NotShadowCaster,
)]
pub(crate) struct VoidBeamModel;

impl VoidBeamModel {
	const COLOR: Color = Color::BLACK;
	const EMISSIVE: LinearRgba = LinearRgba::new(23.0, 23.0, 23.0, 1.);

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

	fn material() -> InsertAsset<StandardMaterial> {
		InsertAsset::shared::<VoidBeamModel>(|| StandardMaterial {
			base_color: Self::COLOR,
			emissive: Self::EMISSIVE,
			alpha_mode: AlphaMode::Add,
			..default()
		})
	}
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoidBeamAttack {
	pub damage: f32,
	pub lifetime: Duration,
	pub range: Units,
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
