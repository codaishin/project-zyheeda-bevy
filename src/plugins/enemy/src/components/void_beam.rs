use crate::traits::insert_attack::InsertAttack;
use bevy::{ecs::system::EntityCommands, pbr::NotShadowCaster, prelude::*};
use common::{
	components::{
		insert_asset::InsertAsset,
		is_blocker::Blocker,
		lifetime::Lifetime,
		persistent_entity::PersistentEntity,
	},
	effects::deal_damage::DealDamage,
	errors::Error,
	tools::Units,
	traits::{
		handles_effect::HandlesEffect,
		handles_enemies::{Attacker, Target},
		handles_interactions::{BeamEmitter, HandlesInteractions, InteractAble},
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
	damage: f32,
	range: Units,
	attacker: PersistentEntity,
	target: PersistentEntity,
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
		entity.try_insert_if_new((
			TInteractions::TInteraction::from(InteractAble::Beam {
				emitter: BeamEmitter {
					mounted_on: self.attacker,
					range: self.range,
					insert_beam_model: |entity| {
						entity.try_insert(VoidBeamModel);
					},
				},
				blocked_by: Blocker::all(),
			}),
			TInteractions::effect(DealDamage::once_per_second(self.damage)),
		));

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
		entity.insert((
			Lifetime::from(self.lifetime),
			VoidBeam {
				damage: self.damage,
				range: self.range,
				attacker,
				target,
			},
		));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::clamp_zero_positive::ClampZeroPositive;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	fn system(
		entity: Entity,
		attack: VoidBeamAttack,
		attacker: Attacker,
		target: Target,
	) -> impl Fn(Commands) {
		move |mut commands: Commands| {
			let mut entity = commands.entity(entity);
			attack.insert_attack(&mut entity, attacker, target);
		}
	}

	#[test]
	fn insert_void_beam() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let attack = VoidBeamAttack {
			damage: 42.,
			lifetime: Duration::from_millis(42),
			range: Units::new(11.),
		};
		let attacker = PersistentEntity::default();
		let target = PersistentEntity::default();

		app.world_mut().run_system_once(system(
			entity,
			attack,
			Attacker(attacker),
			Target(target),
		))?;

		assert_eq!(
			Some(&VoidBeam {
				damage: attack.damage,
				range: attack.range,
				attacker,
				target,
			}),
			app.world().entity(entity).get::<VoidBeam>()
		);
		Ok(())
	}

	#[test]
	fn insert_void_beam_life_time() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let attack = VoidBeamAttack {
			damage: 42.,
			lifetime: Duration::from_millis(42),
			range: Units::new(11.),
		};
		let attacker = PersistentEntity::default();
		let target = PersistentEntity::default();

		app.world_mut().run_system_once(system(
			entity,
			attack,
			Attacker(attacker),
			Target(target),
		))?;

		assert_eq!(
			Some(&Lifetime::from(attack.lifetime)),
			app.world().entity(entity).get::<Lifetime>()
		);
		Ok(())
	}
}
