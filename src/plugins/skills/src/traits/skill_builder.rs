use crate::behaviors::{build_skill_shape::OnSkillStop, SkillCaster, SkillSpawner, Target};
use behaviors::components::LifeTime;
use bevy::{
	ecs::system::EntityCommands,
	prelude::{BuildChildren, Bundle, Commands, Entity},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub(crate) trait BuildContact {
	fn build_contact(
		&self,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> impl Bundle;
}

pub(crate) trait BuildProjection {
	fn build_projection(
		&self,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> impl Bundle;
}

#[derive(Default, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum LifeTimeDefinition {
	#[default]
	UntilStopped,
	Infinite,
	UntilOutlived(Duration),
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
	fn build(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &Target,
	) -> SkillShape;
}

impl<T> SkillBuilder for T
where
	T: BuildContact + BuildProjection + SkillLifetime,
{
	fn build(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> SkillShape {
		let contact = commands.spawn(self.build_contact(caster, spawner, target));
		let (contact, on_skill_stop) = match self.lifetime() {
			LifeTimeDefinition::UntilStopped => stoppable_contact(contact),
			LifeTimeDefinition::UntilOutlived(duration) => lifetime_contact(contact, duration),
			LifeTimeDefinition::Infinite => infinite_contact(contact),
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

fn stoppable_contact(contact: EntityCommands) -> (Entity, OnSkillStop) {
	let contact = contact.id();
	(contact, OnSkillStop::Stop(contact))
}

fn lifetime_contact(
	mut contact: EntityCommands<'_>,
	duration: std::time::Duration,
) -> (Entity, OnSkillStop) {
	contact.insert(LifeTime(duration));
	(contact.id(), OnSkillStop::Ignore)
}

fn infinite_contact(contact: EntityCommands) -> (Entity, OnSkillStop) {
	let contact = contact.id();
	(contact, OnSkillStop::Ignore)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::App,
		ecs::system::RunSystemOnce,
		math::{Ray3d, Vec3},
		prelude::{Commands, Component, GlobalTransform, In, Parent, Transform},
		utils::default,
	};
	use std::time::Duration;

	#[derive(Component, Debug, PartialEq)]
	struct _Contact {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: Target,
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Projection {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: Target,
	}

	struct _Skill {
		lifetime: LifeTimeDefinition,
	}

	impl SkillLifetime for _Skill {
		fn lifetime(&self) -> LifeTimeDefinition {
			self.lifetime
		}
	}

	impl BuildContact for _Skill {
		fn build_contact(
			&self,
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &Target,
		) -> impl Bundle {
			_Contact {
				caster: *caster,
				spawner: *spawner,
				target: *target,
			}
		}
	}

	impl BuildProjection for _Skill {
		fn build_projection(
			&self,
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &Target,
		) -> impl Bundle {
			_Projection {
				caster: *caster,
				spawner: *spawner,
				target: *target,
			}
		}
	}

	fn build_skill(
		args: In<(_Skill, SkillCaster, SkillSpawner, Target)>,
		mut commands: Commands,
	) -> SkillShape {
		let In((skill, caster, spawner, target)) = args;
		skill.build(&mut commands, &caster, &spawner, &target)
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn spawn_contact() {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};
		let caster = SkillCaster(
			Entity::from_raw(42),
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.)),
		);
		let spawner = SkillSpawner(
			Entity::from_raw(43),
			GlobalTransform::from(Transform::from_xyz(4., 5., 6.)),
		);
		let target = Target {
			ray: Ray3d::new(Vec3::X, Vec3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill);

		assert_eq!(
			Some(&_Contact {
				caster,
				spawner,
				target
			}),
			app.world().entity(shape.contact).get::<_Contact>()
		)
	}

	#[test]
	fn spawn_projection() {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};
		let caster = SkillCaster(
			Entity::from_raw(42),
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.)),
		);
		let spawner = SkillSpawner(
			Entity::from_raw(43),
			GlobalTransform::from(Transform::from_xyz(4., 5., 6.)),
		);
		let target = Target {
			ray: Ray3d::new(Vec3::X, Vec3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill);

		assert_eq!(
			Some(&_Projection {
				caster,
				spawner,
				target
			}),
			app.world().entity(shape.projection).get::<_Projection>()
		)
	}

	#[test]
	fn projection_is_child_of_contact() {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};
		let caster = SkillCaster(
			Entity::from_raw(42),
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.)),
		);
		let spawner = SkillSpawner(
			Entity::from_raw(43),
			GlobalTransform::from(Transform::from_xyz(4., 5., 6.)),
		);
		let target = Target {
			ray: Ray3d::new(Vec3::X, Vec3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill);

		assert_eq!(
			Some(shape.contact),
			app.world()
				.entity(shape.projection)
				.get::<Parent>()
				.map(Parent::get)
		)
	}

	#[test]
	fn alive_until_stopped() {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilStopped,
		};
		let caster = SkillCaster(
			Entity::from_raw(42),
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.)),
		);
		let spawner = SkillSpawner(
			Entity::from_raw(43),
			GlobalTransform::from(Transform::from_xyz(4., 5., 6.)),
		);
		let target = Target {
			ray: Ray3d::new(Vec3::X, Vec3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill);

		assert_eq!(OnSkillStop::Stop(shape.contact), shape.on_skill_stop)
	}

	#[test]
	fn unstoppable_life_time() {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilOutlived(Duration::from_nanos(42)),
		};
		let caster = SkillCaster(
			Entity::from_raw(42),
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.)),
		);
		let spawner = SkillSpawner(
			Entity::from_raw(43),
			GlobalTransform::from(Transform::from_xyz(4., 5., 6.)),
		);
		let target = Target {
			ray: Ray3d::new(Vec3::X, Vec3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill);

		assert_eq!(OnSkillStop::Ignore, shape.on_skill_stop)
	}

	#[test]
	fn add_lifetime_to_unstoppable() {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::UntilOutlived(Duration::from_nanos(42)),
		};
		let caster = SkillCaster(
			Entity::from_raw(42),
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.)),
		);
		let spawner = SkillSpawner(
			Entity::from_raw(43),
			GlobalTransform::from(Transform::from_xyz(4., 5., 6.)),
		);
		let target = Target {
			ray: Ray3d::new(Vec3::X, Vec3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill);

		assert_eq!(
			Some(&LifeTime(Duration::from_nanos(42))),
			app.world().entity(shape.contact).get::<LifeTime>()
		);
	}

	#[test]
	fn infinite_life_time() {
		let mut app = setup();
		let skill = _Skill {
			lifetime: LifeTimeDefinition::Infinite,
		};
		let caster = SkillCaster(
			Entity::from_raw(42),
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.)),
		);
		let spawner = SkillSpawner(
			Entity::from_raw(43),
			GlobalTransform::from(Transform::from_xyz(4., 5., 6.)),
		);
		let target = Target {
			ray: Ray3d::new(Vec3::X, Vec3::Z),
			..default()
		};

		let shape = app
			.world_mut()
			.run_system_once_with((skill, caster, spawner, target), build_skill);

		assert_eq!(OnSkillStop::Ignore, shape.on_skill_stop)
	}
}
