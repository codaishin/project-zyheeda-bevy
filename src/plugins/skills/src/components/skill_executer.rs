use crate::{
	behaviors::{
		build_skill_shape::OnSkillStop,
		spawn_on::SpawnOn,
		SkillBehaviorConfig,
		SkillCaster,
		SkillSpawner,
		Target,
	},
	items::slot_key::SlotKey,
	skills::{lifetime::LifeTimeDefinition, RunSkillBehavior},
	traits::{Execute, Flush, Schedule},
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	traits::get::Get,
};

use super::skill_spawners::SkillSpawners;

#[derive(Component, Debug, PartialEq, Default, Clone)]
pub(crate) enum SkillExecuter {
	#[default]
	Idle,
	Start {
		slot_key: SlotKey,
		shape: RunSkillBehavior,
	},
	StartedStoppable(Entity),
	Stop(Entity),
}

impl Schedule for SkillExecuter {
	fn schedule(&mut self, slot_key: SlotKey, shape: RunSkillBehavior) {
		*self = SkillExecuter::Start { slot_key, shape };
	}
}

impl Flush for SkillExecuter {
	fn flush(&mut self) {
		match self {
			SkillExecuter::StartedStoppable(entity) => {
				*self = SkillExecuter::Stop(*entity);
			}
			SkillExecuter::Start { .. } => {
				*self = SkillExecuter::Idle;
			}
			_ => {}
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct NoSkillSpawner(Option<SlotKey>);

impl From<NoSkillSpawner> for Error {
	fn from(NoSkillSpawner(slot): NoSkillSpawner) -> Self {
		Error {
			msg: format!("Could not find spawner for Slot: {slot:?}"),
			lvl: Level::Error,
		}
	}
}

impl Execute for SkillExecuter {
	type TError = NoSkillSpawner;

	fn execute(
		&mut self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawners: &SkillSpawners,
		target: &Target,
	) -> Result<(), Self::TError> {
		match self {
			SkillExecuter::Start {
				shape: RunSkillBehavior::OnActive(skill),
				slot_key,
			} => {
				let slot_key = match skill.spawn_on {
					SpawnOn::Center => None,
					SpawnOn::Slot => Some(*slot_key),
				};
				let Some(spawner) = spawners.get(&slot_key) else {
					return Err(NoSkillSpawner(slot_key));
				};
				*self = execute(skill, commands, caster, spawner, target);
			}

			SkillExecuter::Start {
				shape: RunSkillBehavior::OnAim(skill),
				slot_key,
			} => {
				let slot_key = match skill.spawn_on {
					SpawnOn::Center => None,
					SpawnOn::Slot => Some(*slot_key),
				};
				let Some(spawner) = spawners.get(&slot_key) else {
					return Err(NoSkillSpawner(slot_key));
				};
				*self = execute(skill, commands, caster, spawner, target);
			}

			SkillExecuter::Stop(skills) => {
				*self = stop(skills, commands);
			}
			_ => {}
		};

		Ok(())
	}
}

fn execute<T>(
	skill: &mut SkillBehaviorConfig<T>,
	commands: &mut Commands,
	caster: &SkillCaster,
	spawner: &SkillSpawner,
	target: &Target,
) -> SkillExecuter
where
	LifeTimeDefinition: From<T>,
	T: Clone,
{
	match spawn_and_execute(skill, commands, caster, spawner, target) {
		OnSkillStop::Ignore => SkillExecuter::Idle,
		OnSkillStop::Stop(entity) => SkillExecuter::StartedStoppable(entity),
	}
}

fn spawn_and_execute<T>(
	behavior: &SkillBehaviorConfig<T>,
	commands: &mut Commands,
	caster: &SkillCaster,
	spawner: &SkillSpawner,
	target: &Target,
) -> OnSkillStop
where
	LifeTimeDefinition: From<T>,
	T: Clone,
{
	let shape = behavior.spawn_shape(commands, caster, spawner, target);

	if let Some(mut contact) = commands.get_entity(shape.contact) {
		behavior.start_contact_behavior(&mut contact, caster, spawner, target);
	};

	if let Some(mut projection) = commands.get_entity(shape.projection) {
		behavior.start_projection_behavior(&mut projection, caster, spawner, target);
	};

	shape.on_skill_stop
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
	use crate::{
		behaviors::{
			build_skill_shape::{BuildSkillShape, OnSkillStop},
			spawn_on::SpawnOn,
			start_behavior::SkillBehavior,
		},
		components::skill_spawners::SkillSpawners,
		traits::skill_builder::SkillShape,
	};
	use bevy::ecs::system::{EntityCommands, RunSystemOnce};
	use common::{
		components::{Outdated, Side},
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
	};

	#[derive(Component, Debug, PartialEq)]
	struct _Args {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: Target,
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

	fn execute(
		In((caster, spawners, target)): In<(SkillCaster, SkillSpawners, Target)>,
		mut cmd: Commands,
		mut executers: Query<&mut SkillExecuter>,
	) -> Result<(), NoSkillSpawner> {
		let mut executer = executers.single_mut();
		executer.execute(&mut cmd, &caster, &spawners, &target)
	}

	fn spawned_args(app: &App, predicate: fn(&EntityRef) -> bool) -> Vec<&_Args> {
		app.world()
			.iter_entities()
			.filter(predicate)
			.filter_map(|e| e.get::<_Args>())
			.collect()
	}

	fn no_filter(_: &EntityRef) -> bool {
		true
	}

	fn filter<T: Component>(entity: &EntityRef) -> bool {
		entity.contains::<T>()
	}

	#[test]
	fn set_self_to_start_skill() {
		let shape = RunSkillBehavior::OnActive(SkillBehaviorConfig::from_shape(
			BuildSkillShape::Fn(|cmd, _, _, _| SkillShape {
				contact: cmd.spawn_empty().id(),
				projection: cmd.spawn_empty().id(),
				on_skill_stop: OnSkillStop::Ignore,
			}),
		));
		let slot_key = SlotKey::BottomHand(Side::Left);

		let mut executer = SkillExecuter::default();
		executer.schedule(slot_key, shape.clone());

		assert_eq!(SkillExecuter::Start { slot_key, shape }, executer);
	}

	#[test]
	fn spawn_skill_contact_entity_on_active() {
		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnActive(
				SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(
					|cmd, caster, spawner, target| SkillShape {
						contact: cmd
							.spawn(_Args {
								caster: *caster,
								spawner: *spawner,
								target: *target,
							})
							.id(),
						projection: cmd.spawn_empty().id(),
						on_skill_stop: OnSkillStop::Ignore,
					},
				))
				.spawning_on(SpawnOn::Slot),
			),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(Some(SlotKey::BottomHand(Side::Right)), spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		assert_eq!(
			(
				Ok(()),
				vec![&_Args {
					caster,
					spawner,
					target
				}]
			),
			(ok, spawned_args(&app, no_filter))
		)
	}

	#[test]
	fn spawn_skill_contact_entity_on_active_centered() {
		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnActive(
				SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(
					|cmd, caster, spawner, target| SkillShape {
						contact: cmd
							.spawn(_Args {
								caster: *caster,
								spawner: *spawner,
								target: *target,
							})
							.id(),
						projection: cmd.spawn_empty().id(),
						on_skill_stop: OnSkillStop::Ignore,
					},
				))
				.spawning_on(SpawnOn::Center),
			),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		assert_eq!(
			(
				Ok(()),
				vec![&_Args {
					caster,
					spawner,
					target
				}]
			),
			(ok, spawned_args(&app, no_filter))
		);
	}

	#[test]
	fn apply_contact_behavior_on_active() {
		#[derive(Component)]
		struct _Contact;

		fn behavior(e: &mut EntityCommands, c: &SkillCaster, s: &SkillSpawner, t: &Target) {
			e.try_insert(_Args {
				caster: *c,
				spawner: *s,
				target: *t,
			});
		}

		fn shape(cmd: &mut Commands, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> SkillShape {
			SkillShape {
				contact: cmd.spawn(_Contact).id(),
				projection: cmd.spawn_empty().id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnActive(
				SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(shape))
					.with_contact_behaviors(vec![SkillBehavior::Fn(behavior)]),
			),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		assert_eq!(
			(
				Ok(()),
				vec![&_Args {
					caster,
					spawner,
					target
				}]
			),
			(ok, spawned_args(&app, filter::<_Contact>))
		);
	}

	#[test]
	fn spawn_skill_projection_entity_on_active() {
		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnActive(SkillBehaviorConfig::from_shape(
				BuildSkillShape::Fn(|cmd, caster, spawner, target| SkillShape {
					contact: cmd
						.spawn(_Args {
							caster: *caster,
							spawner: *spawner,
							target: *target,
						})
						.id(),
					projection: cmd.spawn_empty().id(),
					on_skill_stop: OnSkillStop::Ignore,
				}),
			)),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		assert_eq!(
			(
				Ok(()),
				vec![&_Args {
					caster,
					spawner,
					target
				}]
			),
			(ok, spawned_args(&app, no_filter))
		);
	}

	#[test]
	fn apply_projection_behavior_on_active() {
		#[derive(Component)]
		struct _Projection;

		fn behavior(e: &mut EntityCommands, c: &SkillCaster, s: &SkillSpawner, t: &Target) {
			e.try_insert(_Args {
				caster: *c,
				spawner: *s,
				target: *t,
			});
		}

		fn shape(cmd: &mut Commands, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> SkillShape {
			SkillShape {
				contact: cmd.spawn_empty().id(),
				projection: cmd.spawn(_Projection).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnActive(
				SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(shape))
					.with_projection_behaviors(vec![SkillBehavior::Fn(behavior)]),
			),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Projection>())
			.filter_map(|e| e.get::<_Args>())
			.collect::<Vec<_>>();

		assert_eq!(
			(
				Ok(()),
				vec![&_Args {
					caster,
					spawner,
					target
				}]
			),
			(ok, spawn_args)
		);
	}

	#[test]
	fn set_to_started_contact_as_stoppable_on_active() {
		let (mut app, executer) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnActive(SkillBehaviorConfig::from_shape(
				BuildSkillShape::Fn(|cmd, _, _, _| SkillShape {
					contact: cmd.spawn_empty().id(),
					projection: cmd.spawn_empty().id(),
					on_skill_stop: OnSkillStop::Stop(Entity::from_raw(998877)),
				}),
			)),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(
			(
				Ok(()),
				&SkillExecuter::StartedStoppable(Entity::from_raw(998877))
			),
			(ok, executer)
		);
	}

	#[test]
	fn set_to_started_projection_as_stoppable_on_active() {
		let (mut app, executer) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnActive(SkillBehaviorConfig::from_shape(
				BuildSkillShape::Fn(|cmd, _, _, _| SkillShape {
					contact: cmd.spawn_empty().id(),
					projection: cmd.spawn_empty().id(),
					on_skill_stop: OnSkillStop::Stop(Entity::from_raw(998877)),
				}),
			)),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		assert_eq!(
			(
				Ok(()),
				&SkillExecuter::StartedStoppable(Entity::from_raw(998877))
			),
			(
				ok,
				app.world().entity(executer).get::<SkillExecuter>().unwrap()
			)
		);
	}

	#[test]
	fn set_to_idle_on_flush_when_set_to_start_on_active() {
		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnActive(SkillBehaviorConfig::from_shape(
				BuildSkillShape::NO_SHAPE,
			)),
		};

		executer.flush();

		assert_eq!(SkillExecuter::Idle, executer);
	}

	#[test]
	fn result_on_active_error() {
		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnActive(
				SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(
					|cmd, caster, spawner, target| SkillShape {
						contact: cmd
							.spawn(_Args {
								caster: *caster,
								spawner: *spawner,
								target: *target,
							})
							.id(),
						projection: cmd.spawn_empty().id(),
						on_skill_stop: OnSkillStop::Ignore,
					},
				))
				.spawning_on(SpawnOn::Slot),
			),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		assert_eq!(
			Err(NoSkillSpawner(Some(SlotKey::BottomHand(Side::Right)))),
			app.world_mut()
				.run_system_once_with((caster, spawners, target), execute),
		);
	}

	#[test]
	fn spawn_skill_contact_entity_on_aim() {
		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnAim(
				SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(
					|cmd, caster, spawner, target| SkillShape {
						contact: cmd
							.spawn(_Args {
								caster: *caster,
								spawner: *spawner,
								target: *target,
							})
							.id(),
						projection: cmd.spawn_empty().id(),
						on_skill_stop: OnSkillStop::Ignore,
					},
				))
				.spawning_on(SpawnOn::Slot),
			),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(Some(SlotKey::BottomHand(Side::Right)), spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		assert_eq!(
			(
				Ok(()),
				vec![&_Args {
					caster,
					spawner,
					target
				}]
			),
			(ok, spawned_args(&app, no_filter))
		);
	}

	#[test]
	fn spawn_skill_contact_entity_on_aim_centered() {
		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnAim(
				SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(
					|cmd, caster, spawner, target| SkillShape {
						contact: cmd
							.spawn(_Args {
								caster: *caster,
								spawner: *spawner,
								target: *target,
							})
							.id(),
						projection: cmd.spawn_empty().id(),
						on_skill_stop: OnSkillStop::Ignore,
					},
				))
				.spawning_on(SpawnOn::Center),
			),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		assert_eq!(
			(
				Ok(()),
				vec![&_Args {
					caster,
					spawner,
					target
				}]
			),
			(ok, spawned_args(&app, no_filter))
		);
	}

	#[test]
	fn apply_contact_behavior_on_aim() {
		#[derive(Component)]
		struct _Contact;

		fn behavior(e: &mut EntityCommands, c: &SkillCaster, s: &SkillSpawner, t: &Target) {
			e.try_insert(_Args {
				caster: *c,
				spawner: *s,
				target: *t,
			});
		}

		fn shape(cmd: &mut Commands, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> SkillShape {
			SkillShape {
				contact: cmd.spawn(_Contact).id(),
				projection: cmd.spawn_empty().id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnAim(
				SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(shape))
					.with_contact_behaviors(vec![SkillBehavior::Fn(behavior)]),
			),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Contact>())
			.filter_map(|e| e.get::<_Args>())
			.collect::<Vec<_>>();

		assert_eq!(
			(
				Ok(()),
				vec![&_Args {
					caster,
					spawner,
					target
				}]
			),
			(ok, spawn_args)
		);
	}

	#[test]
	fn spawn_skill_projection_entity_on_aim() {
		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnAim(SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(
				|cmd, caster, spawner, target| SkillShape {
					contact: cmd
						.spawn(_Args {
							caster: *caster,
							spawner: *spawner,
							target: *target,
						})
						.id(),
					projection: cmd.spawn_empty().id(),
					on_skill_stop: OnSkillStop::Ignore,
				},
			))),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		assert_eq!(
			(
				Ok(()),
				vec![&_Args {
					caster,
					spawner,
					target
				}]
			),
			(ok, spawned_args(&app, no_filter))
		);
	}

	#[test]
	fn apply_projection_behavior_on_aim() {
		#[derive(Component)]
		struct _Projection;

		fn behavior(e: &mut EntityCommands, c: &SkillCaster, s: &SkillSpawner, t: &Target) {
			e.try_insert(_Args {
				caster: *c,
				spawner: *s,
				target: *t,
			});
		}

		fn shape(cmd: &mut Commands, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> SkillShape {
			SkillShape {
				contact: cmd.spawn_empty().id(),
				projection: cmd.spawn(_Projection).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}
		}

		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnAim(
				SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(shape))
					.with_projection_behaviors(vec![SkillBehavior::Fn(behavior)]),
			),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		let spawn_args = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Projection>())
			.filter_map(|e| e.get::<_Args>())
			.collect::<Vec<_>>();

		assert_eq!(
			(
				Ok(()),
				vec![&_Args {
					caster,
					spawner,
					target
				}]
			),
			(ok, spawn_args)
		);
	}

	#[test]
	fn set_to_started_contact_as_stoppable_on_aim() {
		let (mut app, executer) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnAim(SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(
				|cmd, _, _, _| SkillShape {
					contact: cmd.spawn_empty().id(),
					projection: cmd.spawn_empty().id(),
					on_skill_stop: OnSkillStop::Stop(Entity::from_raw(998877)),
				},
			))),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(
			(
				Ok(()),
				&SkillExecuter::StartedStoppable(Entity::from_raw(998877))
			),
			(ok, executer)
		);
	}

	#[test]
	fn set_to_started_projection_as_stoppable_on_aim() {
		let (mut app, executer) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnAim(SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(
				|cmd, _, _, _| SkillShape {
					contact: cmd.spawn_empty().id(),
					projection: cmd.spawn_empty().id(),
					on_skill_stop: OnSkillStop::Stop(Entity::from_raw(998877)),
				},
			))),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		let executer = app.world().entity(executer).get::<SkillExecuter>().unwrap();

		assert_eq!(
			(
				Ok(()),
				&SkillExecuter::StartedStoppable(Entity::from_raw(998877))
			),
			(ok, executer)
		);
	}

	#[test]
	fn result_on_aim_error() {
		let (mut app, ..) = setup(SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnAim(
				SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(
					|cmd, caster, spawner, target| SkillShape {
						contact: cmd
							.spawn(_Args {
								caster: *caster,
								spawner: *spawner,
								target: *target,
							})
							.id(),
						projection: cmd.spawn_empty().id(),
						on_skill_stop: OnSkillStop::Ignore,
					},
				))
				.spawning_on(SpawnOn::Slot),
			),
		});
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		assert_eq!(
			Err(NoSkillSpawner(Some(SlotKey::BottomHand(Side::Right)))),
			app.world_mut()
				.run_system_once_with((caster, spawners, target), execute)
		);
	}

	#[test]
	fn set_to_idle_on_flush_when_set_to_start_on_aim() {
		let mut executer = SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Right),
			shape: RunSkillBehavior::OnAim(SkillBehaviorConfig::from_shape(
				BuildSkillShape::NO_SHAPE,
			)),
		};

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
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		assert_eq!(
			(Ok(()), 0),
			(
				ok,
				app.world()
					.iter_entities()
					.filter(|e| e.contains::<_Parent>() || e.contains::<_Child>())
					.count()
			)
		);
	}

	#[test]
	fn set_to_idle_on_stop_execution() {
		let (mut app, executer) = setup(SkillExecuter::Stop(Entity::from_raw(1)));
		let caster = SkillCaster(Entity::from_raw(1));
		let spawner = SkillSpawner(Entity::from_raw(2));
		let spawners = SkillSpawners::new([(None, spawner)]);
		let target = get_target();

		let ok = app
			.world_mut()
			.run_system_once_with((caster, spawners, target), execute);

		assert_eq!(
			(Ok(()), Some(&SkillExecuter::Idle)),
			(ok, app.world().entity(executer).get::<SkillExecuter>())
		);
	}
}
