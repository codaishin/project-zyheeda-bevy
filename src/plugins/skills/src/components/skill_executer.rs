use crate::{
	behaviors::{
		spawn_behavior::OnSkillStop,
		SkillBehaviorConfig,
		SkillCaster,
		SkillSpawner,
		Target,
	},
	traits::{Execute, Flush, Schedule},
};
use bevy::{
	ecs::{component::Component, entity::Entity, system::Commands},
	hierarchy::DespawnRecursiveExt,
};

#[derive(Component, Debug, PartialEq, Default, Clone)]
pub(crate) enum SkillExecuter {
	#[default]
	Idle,
	Start(SkillBehaviorConfig),
	StartedStoppable(Entity),
	Stop(Entity),
}

impl Schedule for SkillExecuter {
	fn schedule(&mut self, skill: SkillBehaviorConfig) {
		*self = SkillExecuter::Start(skill);
	}
}

impl Flush for SkillExecuter {
	fn flush(&mut self) {
		match self {
			SkillExecuter::StartedStoppable(entity) => {
				*self = SkillExecuter::Stop(*entity);
			}
			SkillExecuter::Start(_) => {
				*self = SkillExecuter::Idle;
			}
			_ => {}
		}
	}
}

impl Execute for SkillExecuter {
	fn execute(
		&mut self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) {
		match self {
			SkillExecuter::Start(skill) => {
				*self = execute(skill, commands, caster, spawner, target);
			}
			SkillExecuter::Stop(skills) => {
				*self = stop(skills, commands);
			}
			_ => {}
		}
	}
}

fn execute(
	skill: &mut SkillBehaviorConfig,
	commands: &mut Commands,
	caster: &SkillCaster,
	spawner: &SkillSpawner,
	target: &Target,
) -> SkillExecuter {
	match spawn_and_execute(skill, commands, caster, spawner, target) {
		OnSkillStop::Ignore => SkillExecuter::Idle,
		OnSkillStop::Stop(entity) => SkillExecuter::StartedStoppable(entity),
	}
}

fn spawn_and_execute(
	behavior: &SkillBehaviorConfig,
	commands: &mut Commands,
	caster: &SkillCaster,
	spawner: &SkillSpawner,
	target: &Target,
) -> OnSkillStop {
	let (contact, projection, on_skill_stop) =
		behavior.spawn_shape(commands, caster, spawner, target);

	if let Some(mut contact) = commands.get_entity(contact) {
		behavior.start_contact_behavior(&mut contact, caster, spawner, target);
	};

	if let Some(mut projection) = commands.get_entity(projection) {
		behavior.start_projection_behavior(&mut projection, caster, spawner, target);
	};

	on_skill_stop
}

fn stop(skill: &Entity, commands: &mut Commands) -> SkillExecuter {
	if let Some(entity) = commands.get_entity(*skill) {
		entity.despawn_recursive();
	};
	SkillExecuter::Idle
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::behaviors::{
		spawn_behavior::{OnSkillStop, SkillShape},
		start_behavior::SkillBehavior,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{EntityCommands, Query, RunSystemOnce},
		hierarchy::BuildWorldChildren,
		math::{Ray3d, Vec3},
		transform::components::GlobalTransform,
		utils::default,
	};
	use common::{
		components::Outdated,
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
	};

	#[derive(Component, Debug, PartialEq)]
	struct _SpawnArgs {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: Target,
	}

	#[derive(Component, Debug, PartialEq)]
	struct _BehaviorArgs {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: Target,
	}

	fn get_caster() -> SkillCaster {
		SkillCaster(
			Entity::from_raw(99),
			GlobalTransform::from_xyz(42., 42., 42.),
		)
	}

	fn get_spawner() -> SkillSpawner {
		SkillSpawner(
			Entity::from_raw(111),
			GlobalTransform::from_xyz(100., 100., 100.),
		)
	}

	fn get_target() -> Target {
		Target {
			ray: Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.)),
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity: Entity::from_raw(11),
					component: GlobalTransform::from_xyz(10., 10., 10.),
				},
				root: Some(Outdated {
					entity: Entity::from_raw(1),
					component: GlobalTransform::from_xyz(11., 11., 11.),
				}),
			}),
		}
	}

	fn setup(executer: SkillExecuter) -> (App, Entity) {
		let mut app = App::new().single_threaded(Update);
		let executer = app.world_mut().spawn(executer).id();

		(app, executer)
	}

	fn execute(mut cmd: Commands, mut executers: Query<&mut SkillExecuter>) {
		let mut executer = executers.single_mut();
		executer.execute(&mut cmd, &get_caster(), &get_spawner(), &get_target())
	}

	#[test]
	fn set_self_to_start_skill() {
		let skill = SkillBehaviorConfig::new().with_shape(SkillShape::Fn(|c, _, _, _| {
			(
				c.spawn_empty().id(),
				c.spawn_empty().id(),
				OnSkillStop::Ignore,
			)
		}));

		let mut executer = SkillExecuter::default();
		executer.schedule(skill.clone());

		assert_eq!(SkillExecuter::Start(skill), executer);
	}

	#[test]
	fn spawn_skill_contact_entity() {
		let (mut app, ..) = setup(SkillExecuter::Start(SkillBehaviorConfig::new().with_shape(
			SkillShape::Fn(|cmd, caster, spawner, target| {
				(
					cmd.spawn(_SpawnArgs {
						caster: *caster,
						spawner: *spawner,
						target: *target,
					})
					.id(),
					cmd.spawn_empty().id(),
					OnSkillStop::Ignore,
				)
			}),
		)));

		app.world_mut().run_system_once(execute);

		let spawn_args = app
			.world()
			.iter_entities()
			.find_map(|e| e.get::<_SpawnArgs>());

		assert_eq!(
			Some(&_SpawnArgs {
				caster: get_caster(),
				spawner: get_spawner(),
				target: get_target()
			}),
			spawn_args
		)
	}

	#[test]
	fn apply_contact_behavior() {
		#[derive(Component)]
		struct _Contact;

		fn behavior(e: &mut EntityCommands, c: &SkillCaster, s: &SkillSpawner, t: &Target) {
			e.try_insert(_BehaviorArgs {
				caster: *c,
				spawner: *s,
				target: *t,
			});
		}

		fn shape(
			cmd: &mut Commands,
			_: &SkillCaster,
			_: &SkillSpawner,
			_: &Target,
		) -> (Entity, Entity, OnSkillStop) {
			(
				cmd.spawn(_Contact).id(),
				cmd.spawn_empty().id(),
				OnSkillStop::Ignore,
			)
		}

		let (mut app, ..) = setup(SkillExecuter::Start(
			SkillBehaviorConfig::new()
				.with_shape(SkillShape::Fn(shape))
				.with_contact_behaviors(vec![SkillBehavior::Fn(behavior)]),
		));

		app.world_mut().run_system_once(execute);

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Contact>())
			.filter_map(|e| e.get::<_BehaviorArgs>())
			.collect::<Vec<_>>();

		assert_eq!(
			vec![&_BehaviorArgs {
				caster: get_caster(),
				spawner: get_spawner(),
				target: get_target()
			}],
			spawn_args
		);
	}

	#[test]
	fn spawn_skill_projection_entity() {
		let (mut app, ..) = setup(SkillExecuter::Start(SkillBehaviorConfig::new().with_shape(
			SkillShape::Fn(|cmd, caster, spawner, target| {
				(
					cmd.spawn(_SpawnArgs {
						caster: *caster,
						spawner: *spawner,
						target: *target,
					})
					.id(),
					cmd.spawn_empty().id(),
					OnSkillStop::Ignore,
				)
			}),
		)));

		app.world_mut().run_system_once(execute);

		let spawn_args = app
			.world()
			.iter_entities()
			.find_map(|e| e.get::<_SpawnArgs>());

		assert_eq!(
			Some(&_SpawnArgs {
				caster: get_caster(),
				spawner: get_spawner(),
				target: get_target()
			}),
			spawn_args
		)
	}

	#[test]
	fn apply_projection_behavior() {
		#[derive(Component)]
		struct _Projection;

		fn behavior(e: &mut EntityCommands, c: &SkillCaster, s: &SkillSpawner, t: &Target) {
			e.try_insert(_BehaviorArgs {
				caster: *c,
				spawner: *s,
				target: *t,
			});
		}

		fn shape(
			cmd: &mut Commands,
			_: &SkillCaster,
			_: &SkillSpawner,
			_: &Target,
		) -> (Entity, Entity, OnSkillStop) {
			(
				cmd.spawn_empty().id(),
				cmd.spawn(_Projection).id(),
				OnSkillStop::Ignore,
			)
		}

		let (mut app, ..) = setup(SkillExecuter::Start(
			SkillBehaviorConfig::new()
				.with_shape(SkillShape::Fn(shape))
				.with_projection_behaviors(vec![SkillBehavior::Fn(behavior)]),
		));

		app.world_mut().run_system_once(execute);

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Projection>())
			.filter_map(|e| e.get::<_BehaviorArgs>())
			.collect::<Vec<_>>();

		assert_eq!(
			vec![&_BehaviorArgs {
				caster: get_caster(),
				spawner: get_spawner(),
				target: get_target()
			}],
			spawn_args
		);
	}

	#[test]
	fn set_to_started_contact_as_stoppable() {
		let (mut app, executer) = setup(SkillExecuter::Start(
			SkillBehaviorConfig::new().with_shape(SkillShape::Fn(|cmd, _, _, _| {
				(
					cmd.spawn_empty().id(),
					cmd.spawn_empty().id(),
					OnSkillStop::Stop(Entity::from_raw(998877)),
				)
			})),
		));

		app.world_mut().run_system_once(execute);

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(
			&SkillExecuter::StartedStoppable(Entity::from_raw(998877)),
			executer
		);
	}

	#[test]
	fn set_to_started_projection_as_stoppable() {
		let (mut app, executer) = setup(SkillExecuter::Start(
			SkillBehaviorConfig::new().with_shape(SkillShape::Fn(|cmd, _, _, _| {
				(
					cmd.spawn_empty().id(),
					cmd.spawn_empty().id(),
					OnSkillStop::Stop(Entity::from_raw(998877)),
				)
			})),
		));

		app.world_mut().run_system_once(execute);

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(
			&SkillExecuter::StartedStoppable(Entity::from_raw(998877)),
			executer
		);
	}

	#[test]
	fn set_to_idle_on_flush_when_set_to_start() {
		let mut executer = SkillExecuter::Start(default());

		executer.flush();

		assert_eq!(SkillExecuter::Idle, executer);
	}

	#[test]
	fn set_to_stop_on_flush_when_set_to_started() {
		let mut executer = SkillExecuter::StartedStoppable(Entity::from_raw(1));

		executer.flush();

		assert_eq!(SkillExecuter::Stop(Entity::from_raw(1)), executer);
	}

	#[test]
	fn despawn_skill_entity_recursively_on_execute_stop() {
		#[derive(Component)]
		struct _Child;

		#[derive(Component)]
		struct _Parent;

		let (mut app, executer) = setup(SkillExecuter::Idle);
		let skill = app
			.world_mut()
			.spawn(_Parent)
			.with_children(|skill| {
				skill.spawn(_Child);
			})
			.id();
		let mut executer = app.world_mut().entity_mut(executer);
		let mut executer = executer.get_mut::<SkillExecuter>().unwrap();
		*executer = SkillExecuter::Stop(skill);

		app.world_mut().run_system_once(execute);

		assert_eq!(
			0,
			app.world()
				.iter_entities()
				.filter(|e| e.contains::<_Parent>() || e.contains::<_Child>())
				.count()
		);
	}

	#[test]
	fn set_to_idle_on_stop_execution() {
		let (mut app, executer) = setup(SkillExecuter::Stop(Entity::from_raw(1)));

		app.world_mut().run_system_once(execute);

		assert_eq!(
			Some(&SkillExecuter::Idle),
			app.world().entity(executer).get::<SkillExecuter>()
		);
	}
}
