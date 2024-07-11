use crate::{
	skills::{OnSkillStop, SkillCaster, SkillSpawner, StartBehaviorFn, Target},
	traits::{Execute, Flush, Schedule},
};
use bevy::{
	ecs::{component::Component, entity::Entity, system::Commands},
	hierarchy::DespawnRecursiveExt,
};

#[derive(Component, Debug, PartialEq, Default, Clone, Copy)]
pub(crate) enum SkillExecuter {
	#[default]
	Idle,
	Start(StartBehaviorFn),
	StartedStoppable(Entity),
	Stop(Entity),
}

impl Schedule for SkillExecuter {
	fn schedule(&mut self, start: StartBehaviorFn) {
		*self = SkillExecuter::Start(start);
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
			SkillExecuter::Start(start) => {
				*self = execute_start(start, commands, caster, spawner, target);
			}
			SkillExecuter::Stop(entity) => {
				*self = execute_stop(entity, commands);
			}
			_ => {}
		}
	}
}

fn execute_start(
	start: &mut StartBehaviorFn,
	commands: &mut Commands,
	caster: &SkillCaster,
	spawner: &SkillSpawner,
	target: &Target,
) -> SkillExecuter {
	if let OnSkillStop::Stop(entity) = start(commands, caster, spawner, target) {
		SkillExecuter::StartedStoppable(entity)
	} else {
		SkillExecuter::Idle
	}
}

fn execute_stop(entity: &Entity, commands: &mut Commands) -> SkillExecuter {
	if let Some(entity) = commands.get_entity(*entity) {
		entity.despawn_recursive();
	};
	SkillExecuter::Idle
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skills::OnSkillStop;
	use bevy::{
		app::{App, Update},
		ecs::system::Query,
		hierarchy::BuildWorldChildren,
		math::{Ray3d, Vec3},
		transform::components::{GlobalTransform, Transform},
	};
	use common::{
		components::Outdated,
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
	};
	use mockall::mock;

	trait _Run {
		fn run(
			commands: &mut Commands,
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &Target,
		) -> OnSkillStop;
	}

	macro_rules! mock_run_fn {
		($ident:ident) => {
			mock! {
				$ident {}
				impl _Run for $ident {
					#[allow(clippy::needless_lifetimes)]
					fn run<'a, 'b, 'c>(
						commands: &'a mut Commands<'b, 'c>,
						caster: &SkillCaster,
						spawner: &SkillSpawner,
						target: &Target,
					) -> OnSkillStop;
				}
			}
		};
	}

	fn caster() -> SkillCaster {
		SkillCaster(Entity::from_raw(99), Transform::from_xyz(42., 42., 42.))
	}

	fn spawner() -> SkillSpawner {
		SkillSpawner(
			Entity::from_raw(111),
			GlobalTransform::from_xyz(100., 100., 100.),
		)
	}

	fn target() -> Target {
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

	#[test]
	fn schedule() {
		let run_fn: StartBehaviorFn = |_, _, _, _| OnSkillStop::Ignore;

		let mut executer = SkillExecuter::default();
		executer.schedule(run_fn);

		assert_eq!(SkillExecuter::Start(run_fn), executer);
	}

	mock_run_fn!(_ExecuteArgs);

	#[test]
	fn execute_args() {
		let ctx = Mock_ExecuteArgs::run_context();
		ctx.expect()
			.times(1)
			.withf(move |_, c, s, t| {
				assert_eq!((&caster(), &spawner(), &target()), (c, s, t));
				true
			})
			.return_const(OnSkillStop::Ignore);

		let mut executer = SkillExecuter::Start(Mock_ExecuteArgs::run);
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, move |mut commands: Commands| {
			executer.execute(&mut commands, &caster(), &spawner(), &target())
		});
		app.update();
	}

	mock_run_fn!(_Stoppable);

	#[test]
	fn set_to_started_on_execute_stoppable() {
		let ctx = Mock_Stoppable::run_context();
		ctx.expect()
			.return_const(OnSkillStop::Stop(Entity::from_raw(998877)));

		let mut app = App::new().single_threaded(Update);
		let executer = app
			.world_mut()
			.spawn(SkillExecuter::Start(Mock_Stoppable::run))
			.id();
		app.add_systems(
			Update,
			move |mut commands: Commands, mut executers: Query<&mut SkillExecuter>| {
				let mut executer = executers.single_mut();
				executer.execute(&mut commands, &caster(), &spawner(), &target())
			},
		);

		app.update();

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(
			&SkillExecuter::StartedStoppable(Entity::from_raw(998877)),
			executer
		);
	}

	mock_run_fn!(_NonStoppable);

	#[test]
	fn set_to_idle_on_execute_non_stoppable() {
		let ctx = Mock_NonStoppable::run_context();
		ctx.expect().return_const(OnSkillStop::Ignore);

		let mut app = App::new().single_threaded(Update);
		let executer = app
			.world_mut()
			.spawn(SkillExecuter::Start(Mock_NonStoppable::run))
			.id();

		app.add_systems(
			Update,
			|mut commands: Commands, mut executers: Query<&mut SkillExecuter>| {
				let mut executer = executers.single_mut();
				executer.execute(&mut commands, &caster(), &spawner(), &target())
			},
		);

		app.update();

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(&SkillExecuter::Idle, executer);
	}

	#[test]
	fn set_to_idle_on_flush_when_set_to_start() {
		let mut executer = SkillExecuter::Start(|_, _, _, _| OnSkillStop::Ignore);

		executer.flush();

		assert_eq!(SkillExecuter::Idle, executer);
	}

	#[test]
	fn set_to_stop_on_flush_when_set_to_started() {
		let mut executer = SkillExecuter::StartedStoppable(Entity::from_raw(556677));

		executer.flush();

		assert_eq!(SkillExecuter::Stop(Entity::from_raw(556677)), executer);
	}

	#[test]
	fn despawn_skill_entity_recursively_on_execute_stop() {
		let mut app = App::new().single_threaded(Update);
		let skill = app.world_mut().spawn_empty().id();
		app.world_mut().spawn_empty().set_parent(skill);
		let mut executer = SkillExecuter::Stop(skill);

		app.add_systems(Update, move |mut commands: Commands| {
			executer.execute(&mut commands, &caster(), &spawner(), &target());
		});
		app.update();

		assert_eq!(0, app.world().iter_entities().count());
	}

	#[test]
	fn set_to_idle_on_stop_execution() {
		let mut app = App::new().single_threaded(Update);
		let executer = app
			.world_mut()
			.spawn(SkillExecuter::Stop(Entity::from_raw(42)))
			.id();

		app.add_systems(
			Update,
			|mut commands: Commands, mut executers: Query<&mut SkillExecuter>| {
				let mut executer = executers.single_mut();
				executer.execute(&mut commands, &caster(), &spawner(), &target());
			},
		);
		app.update();

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(&SkillExecuter::Idle, executer);
	}
}
