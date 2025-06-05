use crate::{
	behaviors::{SkillCaster, build_skill_shape::OnSkillStop},
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
};
use bevy::prelude::*;
use common::traits::{
	handles_lifetime::HandlesLifetime,
	handles_skill_behaviors::{HandlesSkillBehaviors, SkillEntities, Spawner},
	try_insert_on::TryInsertOn,
};

pub(crate) trait SkillBuilder {
	fn build<TLifetimes, TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: Spawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TLifetimes: HandlesLifetime,
		TSkillBehaviors: HandlesSkillBehaviors + 'static;
}

impl<T> SkillBuilder for T
where
	T: SpawnShape + SkillLifetime,
{
	fn build<TLifetimes, TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: Spawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TLifetimes: HandlesLifetime,
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		let lifetime = lifetime::<TLifetimes>;
		let skill_entities = self.spawn_shape::<TSkillBehaviors>(commands, caster, spawner, target);
		let skill = skill_entities.root;
		let on_skill_stop = match self.lifetime() {
			LifeTimeDefinition::UntilStopped => stoppable(skill),
			LifeTimeDefinition::UntilOutlived(duration) => lifetime(commands, skill, duration),
			LifeTimeDefinition::Infinite => infinite(),
		};

		SkillShape {
			contact: skill_entities.contact,
			projection: skill_entities.projection,
			on_skill_stop,
		}
	}
}

fn stoppable(skill: Entity) -> OnSkillStop {
	OnSkillStop::Stop(skill)
}

fn lifetime<TLifetimes>(
	commands: &mut Commands,
	skill: Entity,
	duration: std::time::Duration,
) -> OnSkillStop
where
	TLifetimes: HandlesLifetime,
{
	commands.try_insert_on(skill, TLifetimes::lifetime(duration));
	OnSkillStop::Ignore
}

fn infinite() -> OnSkillStop {
	OnSkillStop::Ignore
}

pub(crate) trait SpawnShape {
	fn spawn_shape<TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: Spawner,
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
		traits::handles_skill_behaviors::{Contact, Projection, SkillEntities},
	};
	use std::{any::type_name, time::Duration};

	struct _HandlesSkillBehaviors;

	impl HandlesSkillBehaviors for _HandlesSkillBehaviors {
		type TSkillContact = _Contact;
		type TSkillProjection = _Projection;

		fn spawn_skill(_: &mut Commands, _: Contact, _: Projection) -> SkillEntities {
			panic!("SHOULD NOT BE CALLED")
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Root {
		caster: SkillCaster,
		spawner: Spawner,
		target: SkillTarget,
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Contact;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Projection;

	struct _Skill {
		lifetime: LifeTimeDefinition,
	}

	struct _HandlesLifetime;

	impl HandlesLifetime for _HandlesLifetime {
		fn lifetime(duration: Duration) -> impl Bundle {
			_Lifetime(duration)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Lifetime(Duration);

	impl From<Duration> for _Lifetime {
		fn from(duration: Duration) -> Self {
			_Lifetime(duration)
		}
	}

	impl SkillLifetime for _Skill {
		fn lifetime(&self) -> LifeTimeDefinition {
			self.lifetime
		}
	}

	impl SpawnShape for _Skill {
		fn spawn_shape<TSkillBehaviors>(
			&self,
			commands: &mut Commands,
			caster: &SkillCaster,
			spawner: Spawner,
			target: &SkillTarget,
		) -> SkillEntities
		where
			TSkillBehaviors: HandlesSkillBehaviors + 'static,
		{
			let root = commands
				.spawn(_Root {
					caster: *caster,
					spawner,
					target: *target,
				})
				.id();
			let contact = commands.spawn(_Contact).id();
			let projection = commands.spawn(_Projection).id();

			SkillEntities {
				root,
				contact,
				projection,
			}
		}
	}

	fn build_skill(
		args: In<(_Skill, SkillCaster, Spawner, SkillTarget)>,
		mut commands: Commands,
	) -> SkillShape {
		let In((skill, caster, spawner, target)) = args;
		skill.build::<_HandlesLifetime, _HandlesSkillBehaviors>(
			&mut commands,
			&caster,
			spawner,
			&target,
		)
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
		let spawner = Spawner::Center;
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
		let spawner = Spawner::Center;
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
		let spawner = Spawner::Center;
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
		let spawner = Spawner::Center;
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

		let root = find_entity_with!(_Root, app).id();
		assert_eq!(OnSkillStop::Stop(root), shape.on_skill_stop);
		Ok(())
	}

	#[test]
	fn unstoppable_life_time() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = Spawner::Center;
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
		let spawner = Spawner::Center;
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
			Some(&_Lifetime(Duration::from_nanos(42))),
			app.world().entity(root).get::<_Lifetime>()
		);
		Ok(())
	}

	#[test]
	fn infinite_life_time() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(PersistentEntity::default());
		let spawner = Spawner::Center;
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
