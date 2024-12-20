use crate::{
	behaviors::{build_skill_shape::OnSkillStop, SkillCaster, SkillSpawner},
	skills::lifetime_definition::LifeTimeDefinition,
};
use behaviors::components::skill_behavior::SkillTarget;
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::traits::handles_lifetime::HandlesLifetime;

pub(crate) trait BuildContact {
	type TContact: Component;

	fn build_contact(
		&self,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) -> Self::TContact;
}

pub(crate) trait BuildProjection {
	type TProjection: Component;

	fn build_projection(
		&self,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) -> Self::TProjection;
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
	fn build<TLifetimes>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TLifetimes: HandlesLifetime;
}

impl<T> SkillBuilder for T
where
	T: BuildContact + BuildProjection + SkillLifetime,
{
	fn build<TLifetimes>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TLifetimes: HandlesLifetime,
	{
		let contact_with_lifetime = contact::<TLifetimes>;
		let entity = commands.spawn(self.build_contact(caster, spawner, target));
		let (contact, on_skill_stop) = match self.lifetime() {
			LifeTimeDefinition::UntilStopped => contact_stoppable(entity),
			LifeTimeDefinition::UntilOutlived(duration) => contact_with_lifetime(entity, duration),
			LifeTimeDefinition::Infinite => contact_infinite(entity),
		};
		let projection = commands
			.spawn(self.build_projection(caster, spawner, target))
			.set_parent(contact)
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
	use std::time::Duration;

	#[derive(Component, Debug, PartialEq)]
	struct _Contact {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Projection {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	}

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

	impl BuildContact for _Skill {
		type TContact = _Contact;

		fn build_contact(
			&self,
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &SkillTarget,
		) -> Self::TContact {
			_Contact {
				caster: *caster,
				spawner: *spawner,
				target: *target,
			}
		}
	}

	impl BuildProjection for _Skill {
		type TProjection = _Projection;

		fn build_projection(
			&self,
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &SkillTarget,
		) -> Self::TProjection {
			_Projection {
				caster: *caster,
				spawner: *spawner,
				target: *target,
			}
		}
	}

	fn build_skill(
		args: In<(_Skill, SkillCaster, SkillSpawner, SkillTarget)>,
		mut commands: Commands,
	) -> SkillShape {
		let In((skill, caster, spawner, target)) = args;
		skill.build::<_HandlesLifetime>(&mut commands, &caster, &spawner, &target)
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn spawn_contact() -> Result<(), RunSystemError> {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill)?;

		assert_eq!(
			Some(&_Contact {
				caster,
				spawner,
				target
			}),
			app.world().entity(shape.contact).get::<_Contact>()
		);
		Ok(())
	}

	#[test]
	fn spawn_projection() -> Result<(), RunSystemError> {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill)?;

		assert_eq!(
			Some(&_Projection {
				caster,
				spawner,
				target
			}),
			app.world().entity(shape.projection).get::<_Projection>()
		);
		Ok(())
	}

	#[test]
	fn projection_is_child_of_contact() -> Result<(), RunSystemError> {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill)?;

		assert_eq!(
			Some(shape.contact),
			app.world()
				.entity(shape.projection)
				.get::<Parent>()
				.map(Parent::get)
		);
		Ok(())
	}

	#[test]
	fn alive_until_stopped() -> Result<(), RunSystemError> {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill)?;

		assert_eq!(OnSkillStop::Stop(shape.contact), shape.on_skill_stop);
		Ok(())
	}

	#[test]
	fn unstoppable_life_time() -> Result<(), RunSystemError> {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilOutlived(Duration::from_nanos(42)),
		};
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill)?;

		assert_eq!(OnSkillStop::Ignore, shape.on_skill_stop);
		Ok(())
	}

	#[test]
	fn add_lifetime_to_unstoppable() -> Result<(), RunSystemError> {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilOutlived(Duration::from_nanos(42)),
		};
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill)?;

		assert_eq!(
			Some(&_Lifetime(Duration::from_nanos(42))),
			app.world().entity(shape.contact).get::<_Lifetime>()
		);
		Ok(())
	}

	#[test]
	fn infinite_life_time() -> Result<(), RunSystemError> {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::Infinite,
		};
		let caster = SkillCaster(Entity::from_raw(42));
		let spawner = SkillSpawner(Entity::from_raw(43));
		let target = SkillTarget {
			ray: Ray3d::new(Vec3::X, Dir3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill)?;

		assert_eq!(OnSkillStop::Ignore, shape.on_skill_stop);
		Ok(())
	}
}
