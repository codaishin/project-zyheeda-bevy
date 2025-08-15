use crate::{
	behaviors::{SkillCaster, spawn_skill::OnSkillStop},
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
};
use bevy::prelude::*;
use common::{
	components::{lifetime::Lifetime, persistent_entity::PersistentEntity},
	traits::{
		accessors::get::TryApplyOn,
		handles_skill_behaviors::{HandlesSkillBehaviors, SkillEntities, SkillSpawner},
	},
	zyheeda_commands::ZyheedaCommands,
};

pub(crate) trait SkillBuilder {
	fn build<TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: &SkillCaster,
		spawner: SkillSpawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static;
}

impl<T> SkillBuilder for T
where
	T: SpawnShape + SkillLifetime,
{
	fn build<TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: &SkillCaster,
		spawner: SkillSpawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		let skill_entities = self.spawn_shape::<TSkillBehaviors>(commands, caster, spawner, target);
		let on_skill_stop = match self.lifetime() {
			LifeTimeDefinition::UntilStopped => stoppable(skill_entities.root.persistent_entity),
			LifeTimeDefinition::UntilOutlived(duration) => {
				lifetime(commands, skill_entities.root.entity, duration)
			}
			LifeTimeDefinition::Infinite => infinite(),
		};

		SkillShape {
			contact: skill_entities.contact,
			projection: skill_entities.projection,
			on_skill_stop,
		}
	}
}

fn stoppable(skill: PersistentEntity) -> OnSkillStop {
	OnSkillStop::Stop(skill)
}

fn lifetime(
	commands: &mut ZyheedaCommands,
	skill: Entity,
	duration: std::time::Duration,
) -> OnSkillStop {
	commands.try_apply_on(&skill, |mut e| {
		e.try_insert(Lifetime::from(duration));
	});
	OnSkillStop::Ignore
}

fn infinite() -> OnSkillStop {
	OnSkillStop::Ignore
}

pub(crate) trait SpawnShape {
	fn spawn_shape<TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: &SkillCaster,
		spawner: SkillSpawner,
		target: &SkillTarget,
	) -> SkillEntities
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static;
}

pub(crate) trait SkillLifetime {
	fn lifetime(&self) -> LifeTimeDefinition;
}

pub(crate) struct SkillShape {
	pub(crate) contact: Entity,
	pub(crate) projection: Entity,
	pub(crate) on_skill_stop: OnSkillStop,
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::App,
		ecs::system::{RunSystemError, RunSystemOnce},
		math::{Ray3d, Vec3},
		utils::default,
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::action_key::slot::SlotKey,
		traits::handles_skill_behaviors::{
			Contact,
			HoldSkills,
			Projection,
			SkillEntities,
			SkillRoot,
		},
	};
	use std::{any::type_name, array::IntoIter, sync::LazyLock, time::Duration};

	struct _HandlesSkillBehaviors;

	impl HandlesSkillBehaviors for _HandlesSkillBehaviors {
		type TSkillContact = _Contact;
		type TSkillProjection = _Projection;
		type TSkillUsage = _SkillUsage;

		fn spawn_skill(_: &mut ZyheedaCommands, _: Contact, _: Projection) -> SkillEntities {
			panic!("SHOULD NOT BE CALLED")
		}
	}

	static ROOT: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[derive(Component, Debug, PartialEq, Clone)]
	#[require(PersistentEntity = *ROOT)]
	struct _Root {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Contact;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Projection;

	#[derive(Component)]
	struct _SkillUsage;

	impl HoldSkills for _SkillUsage {
		type Iter<'a> = IntoIter<SlotKey, 0>;

		fn holding(&self) -> Self::Iter<'_> {
			[].into_iter()
		}

		fn started_holding(&self) -> Self::Iter<'_> {
			[].into_iter()
		}
	}

	struct _Skill {
		lifetime: LifeTimeDefinition,
	}

	impl SkillLifetime for _Skill {
		fn lifetime(&self) -> LifeTimeDefinition {
			self.lifetime
		}
	}

	impl SpawnShape for _Skill {
		fn spawn_shape<TSkillBehaviors>(
			&self,
			commands: &mut ZyheedaCommands,
			caster: &SkillCaster,
			spawner: SkillSpawner,
			target: &SkillTarget,
		) -> SkillEntities
		where
			TSkillBehaviors: HandlesSkillBehaviors + 'static,
		{
			let root = commands
				.spawn((_Root {
					caster: *caster,
					spawner,
					target: *target,
				},))
				.id();
			let contact = commands.spawn(_Contact).id();
			let projection = commands.spawn(_Projection).id();

			SkillEntities {
				root: SkillRoot {
					entity: root,
					persistent_entity: *ROOT,
				},
				contact,
				projection,
			}
		}
	}

	fn build_skill(
		args: In<(_Skill, SkillCaster, SkillSpawner, SkillTarget)>,
		mut commands: ZyheedaCommands,
	) -> SkillShape {
		let In((skill, caster, spawner, target)) = args;
		skill.build::<_HandlesSkillBehaviors>(&mut commands, &caster, spawner, &target)
	}

	macro_rules! find_entity_with {
		($ty:ty, $app:expr $(,)?) => {
			$app.world()
				.iter_entities()
				.find(|e| e.contains::<$ty>())
				.unwrap_or_else(|| {
					panic!("NO {} IN WORLD", type_name::<$ty>());
				})
		};
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn spawn_contact() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};

		let shape = app
			.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		assert_eq!(
			Some(&_Contact),
			app.world().entity(shape.contact).get::<_Contact>()
		);
		Ok(())
	}

	#[test]
	fn spawn_projection() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};

		let shape = app
			.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		assert_eq!(
			Some(&_Projection),
			app.world().entity(shape.projection).get::<_Projection>()
		);
		Ok(())
	}

	#[test]
	fn use_correct_args() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};

		app.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		assert_eq!(
			Some(&_Root {
				caster,
				spawner,
				target
			}),
			find_entity_with!(_Root, app).get::<_Root>()
		);
		Ok(())
	}

	#[test]
	fn alive_until_stopped() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};

		let shape = app
			.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		assert_eq!(OnSkillStop::Stop(*ROOT), shape.on_skill_stop);
		Ok(())
	}

	#[test]
	fn unstoppable_life_time() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilOutlived(Duration::from_nanos(42)),
		};

		let shape = app
			.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		assert_eq!(OnSkillStop::Ignore, shape.on_skill_stop);
		Ok(())
	}

	#[test]
	fn add_lifetime_to_unstoppable() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilOutlived(Duration::from_nanos(42)),
		};

		app.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		let root = find_entity_with!(_Root, app).id();
		assert_eq!(
			Some(&Lifetime::from(Duration::from_nanos(42))),
			app.world().entity(root).get::<Lifetime>()
		);
		Ok(())
	}

	#[test]
	fn infinite_life_time() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = SkillSpawner::Neutral;
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::Infinite,
		};

		let shape = app
			.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		assert_eq!(OnSkillStop::Ignore, shape.on_skill_stop);
		Ok(())
	}
}
