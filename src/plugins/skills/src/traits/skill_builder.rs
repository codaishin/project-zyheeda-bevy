use crate::{
	behaviors::{SkillCaster, SkillSpawner, build_skill_shape::OnSkillStop},
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::traits::{
	handles_lifetime::HandlesLifetime,
	handles_skill_behaviors::HandlesSkillBehaviors,
};

pub(crate) trait BuildContact {
	fn build_contact<TSkillBehaviors>(
		&self,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) -> TSkillBehaviors::TSkillContact
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static;
}

pub(crate) trait BuildProjection {
	fn build_projection<TSkillBehaviors>(
		&self,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) -> TSkillBehaviors::TSkillProjection
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

pub(crate) trait SkillBuilder {
	fn build<TLifetimes, TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TLifetimes: HandlesLifetime,
		TSkillBehaviors: HandlesSkillBehaviors + 'static;
}

impl<T> SkillBuilder for T
where
	T: BuildContact + BuildProjection + SkillLifetime,
{
	fn build<TLifetimes, TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TLifetimes: HandlesLifetime,
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		let contact_with_lifetime = contact::<TLifetimes>;
		let entity = commands.spawn(self.build_contact::<TSkillBehaviors>(caster, spawner, target));
		let (contact, on_skill_stop) = match self.lifetime() {
			LifeTimeDefinition::UntilStopped => contact_stoppable(entity),
			LifeTimeDefinition::UntilOutlived(duration) => contact_with_lifetime(entity, duration),
			LifeTimeDefinition::Infinite => contact_infinite(entity),
		};
		let projection = commands
			.spawn(self.build_projection::<TSkillBehaviors>(caster, spawner, target))
			.insert(ChildOf(contact))
			.id();

		SkillShape {
			contact,
			projection,
			on_skill_stop,
		}
	}
}

fn contact_stoppable(contact: EntityCommands) -> (Entity, OnSkillStop) {
	let contact = contact.id();
	(contact, OnSkillStop::Stop(contact))
}

fn contact<TLifetimes>(
	mut contact: EntityCommands<'_>,
	duration: std::time::Duration,
) -> (Entity, OnSkillStop)
where
	TLifetimes: HandlesLifetime,
{
	contact.insert(TLifetimes::lifetime(duration));
	(contact.id(), OnSkillStop::Ignore)
}

fn contact_infinite(contact: EntityCommands) -> (Entity, OnSkillStop) {
	let contact = contact.id();
	(contact, OnSkillStop::Ignore)
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
		simple_init,
		traits::{
			handles_skill_behaviors::{Integrity, Motion, ProjectionOffset, Shape},
			mock::Mock,
		},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::time::Duration;

	struct _HandlesSkillBehaviors;

	impl HandlesSkillBehaviors for _HandlesSkillBehaviors {
		type TSkillContact = _Contact;
		type TSkillProjection = _Projection;

		fn skill_contact(_: Shape, _: Integrity, _: Motion) -> Self::TSkillContact {
			panic!("Mock should be called")
		}

		fn skill_projection(_: Shape, _: Option<ProjectionOffset>) -> Self::TSkillProjection {
			panic!("Mock should be called")
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Contact;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Projection;

	#[derive(NestedMocks)]
	struct _Skill {
		lifetime: LifeTimeDefinition,
		mock: Mock_Skill,
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

	impl BuildContact for _Skill {
		fn build_contact<TSkillBehaviors>(
			&self,
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &SkillTarget,
		) -> TSkillBehaviors::TSkillContact
		where
			TSkillBehaviors: HandlesSkillBehaviors + 'static,
		{
			self.mock
				.build_contact::<TSkillBehaviors>(caster, spawner, target)
		}
	}

	impl BuildProjection for _Skill {
		fn build_projection<TSkillBehaviors>(
			&self,
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &SkillTarget,
		) -> TSkillBehaviors::TSkillProjection
		where
			TSkillBehaviors: HandlesSkillBehaviors + 'static,
		{
			self.mock
				.build_projection::<TSkillBehaviors>(caster, spawner, target)
		}
	}

	mock! {
		_Skill {}
		impl BuildContact for _Skill {
			fn build_contact<TSkillBehaviors>(
				&self,
				caster: &SkillCaster,
				spawner: &SkillSpawner,
				target: &SkillTarget,
			) -> TSkillBehaviors::TSkillContact
			where
				TSkillBehaviors: HandlesSkillBehaviors + 'static;
		}
		impl BuildProjection for _Skill {
			fn build_projection<TSkillBehaviors>(
				&self,
				caster: &SkillCaster,
				spawner: &SkillSpawner,
				target: &SkillTarget,
			) -> TSkillBehaviors::TSkillProjection
			where
				TSkillBehaviors: HandlesSkillBehaviors + 'static;
		}
	}

	simple_init!(Mock_Skill);

	fn build_skill(
		args: In<(_Skill, SkillCaster, SkillSpawner, SkillTarget)>,
		mut commands: Commands,
	) -> SkillShape {
		let In((skill, caster, spawner, target)) = args;
		skill.build::<_HandlesLifetime, _HandlesSkillBehaviors>(
			&mut commands,
			&caster,
			&spawner,
			&target,
		)
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn spawn_contact() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
			mock: Mock_Skill::new_mock(|mock| {
				mock.expect_build_contact::<_HandlesSkillBehaviors>()
					.times(1)
					.with(eq(caster), eq(spawner), eq(target))
					.return_const(_Contact);
				mock.expect_build_projection::<_HandlesSkillBehaviors>()
					.return_const(_Projection);
			}),
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
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
			mock: Mock_Skill::new_mock(|mock| {
				mock.expect_build_contact::<_HandlesSkillBehaviors>()
					.return_const(_Contact);
				mock.expect_build_projection::<_HandlesSkillBehaviors>()
					.times(1)
					.with(eq(caster), eq(spawner), eq(target))
					.return_const(_Projection);
			}),
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
	fn projection_is_child_of_contact() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
			mock: Mock_Skill::new_mock(|mock| {
				mock.expect_build_contact::<_HandlesSkillBehaviors>()
					.return_const(_Contact);
				mock.expect_build_projection::<_HandlesSkillBehaviors>()
					.return_const(_Projection);
			}),
		};

		let shape = app
			.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		assert_eq!(
			Some(shape.contact),
			app.world()
				.entity(shape.projection)
				.get::<ChildOf>()
				.map(ChildOf::parent)
		);
		Ok(())
	}

	#[test]
	fn alive_until_stopped() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
			mock: Mock_Skill::new_mock(|mock| {
				mock.expect_build_contact::<_HandlesSkillBehaviors>()
					.return_const(_Contact);
				mock.expect_build_projection::<_HandlesSkillBehaviors>()
					.return_const(_Projection);
			}),
		};

		let shape = app
			.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		assert_eq!(OnSkillStop::Stop(shape.contact), shape.on_skill_stop);
		Ok(())
	}

	#[test]
	fn unstoppable_life_time() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilOutlived(Duration::from_nanos(42)),
			mock: Mock_Skill::new_mock(|mock| {
				mock.expect_build_contact::<_HandlesSkillBehaviors>()
					.return_const(_Contact);
				mock.expect_build_projection::<_HandlesSkillBehaviors>()
					.return_const(_Projection);
			}),
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
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilOutlived(Duration::from_nanos(42)),
			mock: Mock_Skill::new_mock(|mock| {
				mock.expect_build_contact::<_HandlesSkillBehaviors>()
					.return_const(_Contact);
				mock.expect_build_projection::<_HandlesSkillBehaviors>()
					.return_const(_Projection);
			}),
		};

		let shape = app
			.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		assert_eq!(
			Some(&_Lifetime(Duration::from_nanos(42))),
			app.world().entity(shape.contact).get::<_Lifetime>()
		);
		Ok(())
	}

	#[test]
	fn infinite_life_time() -> Result<(), RunSystemError> {
		let mut app = setup();
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};
		let skill = _Skill {
			lifetime: LifeTimeDefinition::Infinite,
			mock: Mock_Skill::new_mock(|mock| {
				mock.expect_build_contact::<_HandlesSkillBehaviors>()
					.return_const(_Contact);
				mock.expect_build_projection::<_HandlesSkillBehaviors>()
					.return_const(_Projection);
			}),
		};

		let shape = app
			.world_mut()
			.run_system_once_with(build_skill, (skill, caster, spawner, target))?;

		assert_eq!(OnSkillStop::Ignore, shape.on_skill_stop);
		Ok(())
	}
}
