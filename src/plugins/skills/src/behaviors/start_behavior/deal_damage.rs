use crate::behaviors::{SkillCaster, SkillTarget};
use bevy::ecs::system::EntityCommands;
use common::{
	effects::deal_damage::DealDamage,
	traits::{handles_effect::HandlesEffect, handles_skill_behaviors::Spawner},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum StartDealingDamage {
	OneTime(f32),
	OverTime(f32),
}

impl StartDealingDamage {
	pub fn apply<TInteractions>(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: Spawner,
		_: &SkillTarget,
	) where
		TInteractions: HandlesEffect<DealDamage>,
	{
		entity.try_insert(TInteractions::effect(match *self {
			Self::OneTime(dmg) => DealDamage::once(dmg),
			Self::OverTime(dmg) => DealDamage::once_per_second(dmg),
		}));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::components::persistent_entity::PersistentEntity;
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	struct _HandlesDamage;

	impl HandlesEffect<DealDamage> for _HandlesDamage {
		type TTarget = ();
		type TEffectComponent = _Effect;

		fn effect(effect: DealDamage) -> _Effect {
			_Effect(effect)
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Effect(DealDamage);

	struct _HandlesShading;

	static CASTER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	fn damage(damage: StartDealingDamage) -> impl Fn(Commands) -> Entity {
		move |mut commands| {
			let mut entity = commands.spawn_empty();
			damage.apply::<_HandlesDamage>(
				&mut entity,
				&SkillCaster::from(*CASTER),
				Spawner::Center,
				&SkillTarget::default(),
			);
			entity.id()
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_single_target_damage() -> Result<(), RunSystemError> {
		let mut app = setup();

		let start_dealing_damage = StartDealingDamage::OneTime(42.);
		let entity = app
			.world_mut()
			.run_system_once(damage(start_dealing_damage))?;

		assert_eq!(
			Some(&_Effect(DealDamage::once(42.))),
			app.world().entity(entity).get::<_Effect>(),
		);
		Ok(())
	}

	#[test]
	fn insert_over_time_damage() -> Result<(), RunSystemError> {
		let mut app = setup();

		let start_dealing_damage = StartDealingDamage::OverTime(42.);
		let entity = app
			.world_mut()
			.run_system_once(damage(start_dealing_damage))?;

		assert_eq!(
			Some(&_Effect(DealDamage::once_per_second(42.))),
			app.world().entity(entity).get::<_Effect>(),
		);
		Ok(())
	}
}
