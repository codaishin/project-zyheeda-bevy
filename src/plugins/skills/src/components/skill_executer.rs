use crate::{
	behaviors::{spawn_behavior::OnSkillStop, Behavior, SkillCaster, SkillSpawner, Target},
	skills::SkillBehaviors,
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
	Start(SkillBehaviors),
	StartedStoppable(Vec<Entity>),
	Stop(Vec<Entity>),
}

impl Schedule for SkillExecuter {
	fn schedule(&mut self, start: SkillBehaviors) {
		*self = SkillExecuter::Start(start);
	}
}

impl Flush for SkillExecuter {
	fn flush(&mut self) {
		match self {
			SkillExecuter::StartedStoppable(entity) => {
				*self = SkillExecuter::Stop(entity.clone());
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
	skill: &mut SkillBehaviors,
	commands: &mut Commands,
	caster: &SkillCaster,
	spawner: &SkillSpawner,
	target: &Target,
) -> SkillExecuter {
	let skills = [
		spawn_and_execute(&skill.contact, commands, caster, spawner, target),
		spawn_and_execute(&skill.projection, commands, caster, spawner, target),
	];

	SkillExecuter::StartedStoppable(skills.iter().filter_map(stop_on_skill_stop).collect())
}

fn stop_on_skill_stop(skill: &OnSkillStop) -> Option<Entity> {
	match skill {
		OnSkillStop::Stop(entity) => Some(*entity),
		OnSkillStop::Ignore => None,
	}
}

fn spawn_and_execute(
	behavior: &Behavior,
	commands: &mut Commands,
	caster: &SkillCaster,
	spawner: &SkillSpawner,
	target: &Target,
) -> OnSkillStop {
	let (mut entity, on_skill_stop) = behavior.spawn(commands, caster, spawner, target);
	behavior.start(&mut entity, caster, spawner, target);

	on_skill_stop
}

fn stop(skills: &[Entity], commands: &mut Commands) -> SkillExecuter {
	for skill in skills {
		if let Some(entity) = commands.get_entity(*skill) {
			entity.despawn_recursive();
		};
	}
	SkillExecuter::Idle
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::behaviors::{
		spawn_behavior::{OnSkillStop, SpawnBehavior},
		start_behavior::StartBehavior,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{EntityCommands, Query, RunSystemOnce},
		hierarchy::BuildWorldChildren,
		math::{Ray3d, Vec3},
		prelude::BuildChildren,
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
		let skill = SkillBehaviors {
			contact: Behavior::new().with_spawn(SpawnBehavior::Fn(|c, _, _, _| {
				(c.spawn_empty(), OnSkillStop::Ignore)
			})),
			..default()
		};

		let mut executer = SkillExecuter::default();
		executer.schedule(skill.clone());

		assert_eq!(SkillExecuter::Start(skill), executer);
	}

	#[test]
	fn spawn_skill_contact_entity() {
		let (mut app, ..) = setup(SkillExecuter::Start(SkillBehaviors {
			contact: Behavior::new().with_spawn(SpawnBehavior::Fn(
				|cmd, caster, spawner, target| {
					(
						cmd.spawn(_SpawnArgs {
							caster: *caster,
							spawner: *spawner,
							target: *target,
						}),
						OnSkillStop::Ignore,
					)
				},
			)),
			..default()
		}));

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
		fn behavior(e: &mut EntityCommands, c: &SkillCaster, s: &SkillSpawner, t: &Target) {
			e.with_children(|parent| {
				parent.spawn(_BehaviorArgs {
					caster: *c,
					spawner: *s,
					target: *t,
				});
			});
		}
		let (mut app, ..) = setup(SkillExecuter::Start(SkillBehaviors {
			contact: Behavior::new().with_execute(vec![
				StartBehavior::Fn(behavior),
				StartBehavior::Fn(behavior),
			]),
			..default()
		}));

		app.world_mut().run_system_once(execute);

		let spawn_args = app
			.world()
			.iter_entities()
			.filter_map(|e| e.get::<_BehaviorArgs>())
			.collect::<Vec<_>>();

		assert_eq!(
			vec![
				&_BehaviorArgs {
					caster: get_caster(),
					spawner: get_spawner(),
					target: get_target()
				},
				&_BehaviorArgs {
					caster: get_caster(),
					spawner: get_spawner(),
					target: get_target()
				}
			],
			spawn_args
		);
	}

	#[test]
	fn apply_projection_behavior() {
		fn behavior(e: &mut EntityCommands, c: &SkillCaster, s: &SkillSpawner, t: &Target) {
			e.with_children(|parent| {
				parent.spawn(_BehaviorArgs {
					caster: *c,
					spawner: *s,
					target: *t,
				});
			});
		}
		let (mut app, ..) = setup(SkillExecuter::Start(SkillBehaviors {
			projection: Behavior::new().with_execute(vec![
				StartBehavior::Fn(behavior),
				StartBehavior::Fn(behavior),
			]),
			..default()
		}));

		app.world_mut().run_system_once(execute);

		let spawn_args = app
			.world()
			.iter_entities()
			.filter_map(|e| e.get::<_BehaviorArgs>())
			.collect::<Vec<_>>();

		assert_eq!(
			vec![
				&_BehaviorArgs {
					caster: get_caster(),
					spawner: get_spawner(),
					target: get_target()
				},
				&_BehaviorArgs {
					caster: get_caster(),
					spawner: get_spawner(),
					target: get_target()
				}
			],
			spawn_args
		);
	}

	#[test]
	fn spawn_skill_projection_entity() {
		let (mut app, ..) = setup(SkillExecuter::Start(SkillBehaviors {
			projection: Behavior::new().with_spawn(SpawnBehavior::Fn(
				|cmd, caster, spawner, target| {
					(
						cmd.spawn(_SpawnArgs {
							caster: *caster,
							spawner: *spawner,
							target: *target,
						}),
						OnSkillStop::Ignore,
					)
				},
			)),
			..default()
		}));

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
	fn set_to_started_contact_as_stoppable() {
		let (mut app, executer) = setup(SkillExecuter::Start(SkillBehaviors {
			contact: Behavior::new().with_spawn(SpawnBehavior::Fn(|cmd, _, _, _| {
				(
					cmd.spawn_empty(),
					OnSkillStop::Stop(Entity::from_raw(998877)),
				)
			})),
			..default()
		}));

		app.world_mut().run_system_once(execute);

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(
			&SkillExecuter::StartedStoppable(vec![Entity::from_raw(998877)]),
			executer
		);
	}

	#[test]
	fn set_to_started_projection_as_stoppable() {
		let (mut app, executer) = setup(SkillExecuter::Start(SkillBehaviors {
			projection: Behavior::new().with_spawn(SpawnBehavior::Fn(|cmd, _, _, _| {
				(
					cmd.spawn_empty(),
					OnSkillStop::Stop(Entity::from_raw(998877)),
				)
			})),
			..default()
		}));

		app.world_mut().run_system_once(execute);

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(
			&SkillExecuter::StartedStoppable(vec![Entity::from_raw(998877)]),
			executer
		);
	}

	#[test]
	fn set_to_started_projection_and_contact_as_stoppable() {
		let (mut app, executer) = setup(SkillExecuter::Start(SkillBehaviors {
			contact: Behavior::new().with_spawn(SpawnBehavior::Fn(|cmd, _, _, _| {
				(
					cmd.spawn_empty(),
					OnSkillStop::Stop(Entity::from_raw(998877)),
				)
			})),
			projection: Behavior::new().with_spawn(SpawnBehavior::Fn(|cmd, _, _, _| {
				(
					cmd.spawn_empty(),
					OnSkillStop::Stop(Entity::from_raw(112233)),
				)
			})),
		}));

		app.world_mut().run_system_once(execute);

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(
			&SkillExecuter::StartedStoppable(vec![
				Entity::from_raw(998877),
				Entity::from_raw(112233)
			]),
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
		let mut executer = SkillExecuter::StartedStoppable(vec![
			Entity::from_raw(1),
			Entity::from_raw(2),
			Entity::from_raw(3),
		]);

		executer.flush();

		assert_eq!(
			SkillExecuter::Stop(vec![
				Entity::from_raw(1),
				Entity::from_raw(2),
				Entity::from_raw(3)
			]),
			executer
		);
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
		*executer = SkillExecuter::Stop(vec![skill]);

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
		let (mut app, executer) = setup(SkillExecuter::Stop(vec![]));

		app.world_mut().run_system_once(execute);

		assert_eq!(
			Some(&SkillExecuter::Idle),
			app.world().entity(executer).get::<SkillExecuter>()
		);
	}
}
