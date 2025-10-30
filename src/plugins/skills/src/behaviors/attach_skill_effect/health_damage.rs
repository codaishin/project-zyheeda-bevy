use crate::behaviors::{SkillCaster, SkillTarget};
use common::{
	effects::health_damage::HealthDamage,
	traits::handles_physics::HandlesPhysicalEffect,
	zyheeda_commands::ZyheedaEntityCommands,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AttachHealthDamage {
	OneTime(f32),
	OverTime(f32),
}

impl AttachHealthDamage {
	pub fn attach<TPhysics>(
		&self,
		entity: &mut ZyheedaEntityCommands,
		_: SkillCaster,
		_: SkillTarget,
	) where
		TPhysics: HandlesPhysicalEffect<HealthDamage>,
	{
		entity.try_insert(TPhysics::into_effect_component(match *self {
			Self::OneTime(dmg) => HealthDamage::once(dmg),
			Self::OverTime(dmg) => HealthDamage::per_second(dmg),
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

	impl HandlesPhysicalEffect<HealthDamage> for _HandlesDamage {
		type TEffectComponent = _Effect;
		type TAffectedComponent = _Affected;

		fn into_effect_component(effect: HealthDamage) -> _Effect {
			_Effect(effect)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Effect(HealthDamage);

	#[derive(Component)]
	struct _Affected;

	struct _HandlesShading;

	static CASTER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	fn damage(damage: AttachHealthDamage) -> impl Fn(Commands) -> Entity {
		move |mut commands| {
			let mut entity = commands.spawn(()).into();
			damage.attach::<_HandlesDamage>(
				&mut entity,
				SkillCaster(*CASTER),
				SkillTarget::default(),
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

		let start_dealing_damage = AttachHealthDamage::OneTime(42.);
		let entity = app
			.world_mut()
			.run_system_once(damage(start_dealing_damage))?;

		assert_eq!(
			Some(&_Effect(HealthDamage::once(42.))),
			app.world().entity(entity).get::<_Effect>(),
		);
		Ok(())
	}

	#[test]
	fn insert_over_time_damage() -> Result<(), RunSystemError> {
		let mut app = setup();

		let start_dealing_damage = AttachHealthDamage::OverTime(42.);
		let entity = app
			.world_mut()
			.run_system_once(damage(start_dealing_damage))?;

		assert_eq!(
			Some(&_Effect(HealthDamage::per_second(42.))),
			app.world().entity(entity).get::<_Effect>(),
		);
		Ok(())
	}
}
