use crate::{
	behaviors::{SkillCaster, SkillSpawner, Target},
	items::slot_key::SlotKey,
	traits::Execute,
};
use bevy::prelude::*;
use common::{
	errors::Error,
	resources::{CamRay, MouseHover},
	traits::get::Get,
};

type Components<'a, TSkillExecutor, TGetSkillSpawnEntity> = (
	Entity,
	&'a mut TSkillExecutor,
	&'a GlobalTransform,
	&'a TGetSkillSpawnEntity,
);

impl<T> ExecuteSkills for T where T: Component + Execute {}

pub(crate) trait ExecuteSkills
where
	Self: Component + Execute + Sized,
{
	fn execute_on<TGetSkillSpawnEntity>(
		cam_ray: Res<CamRay>,
		mouse_hover: Res<MouseHover>,
		mut commands: Commands,
		mut agents: Query<Components<Self, TGetSkillSpawnEntity>, Changed<Self>>,
		transforms: Query<&GlobalTransform>,
	) -> Vec<Result<(), Error>>
	where
		TGetSkillSpawnEntity: Component + Get<Option<SlotKey>, Entity>,
		Error: From<Self::TError>,
	{
		agents
			.iter_mut()
			.map(|(id, mut skill_executer, transform, skill_spawn)| {
				match get_target(&cam_ray, &mouse_hover, &transforms) {
					None => Ok(()),
					Some(target) => skill_executer.execute(
						&mut commands,
						&SkillCaster(id, *transform),
						|slot_key| {
							skill_spawn
								.get(slot_key)
								.and_then(|entity| Some((entity, transforms.get(*entity).ok()?)))
								.map(|(entity, transform)| SkillSpawner(*entity, *transform))
						},
						&target,
					),
				}
			})
			.filter_map(error)
			.collect()
	}
}

fn error<TError>(result: Result<(), TError>) -> Option<Result<(), Error>>
where
	Error: From<TError>,
{
	match result {
		Ok(()) => None,
		Err(error) => Some(Err(Error::from(error))),
	}
}

fn get_target(
	cam_ray: &Res<CamRay>,
	mouse_hover: &Res<MouseHover>,
	transforms: &Query<&GlobalTransform>,
) -> Option<Target> {
	let get_transform = |entity| transforms.get(entity).ok().cloned();

	Some(Target {
		ray: cam_ray.0?,
		collision_info: mouse_hover
			.0
			.as_ref()
			.and_then(|collider_info| collider_info.with_component(get_transform)),
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::RunSystemOnce,
		math::{Ray3d, Vec3},
		prelude::IntoSystem,
		transform::components::GlobalTransform,
	};
	use common::{
		components::{Outdated, Side},
		errors::Level,
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
	};
	use std::{collections::HashMap, ops::DerefMut};

	type Assert = Option<
		Box<
			dyn FnMut(&mut Commands, &SkillCaster, Option<SkillSpawner>, &Target)
				+ Sync
				+ Send
				+ 'static,
		>,
	>;

	#[derive(Clone, Copy)]
	struct _Error(&'static str);

	impl From<_Error> for Error {
		fn from(_Error(msg): _Error) -> Self {
			Self {
				msg: msg.to_owned(),
				lvl: Level::Error,
			}
		}
	}

	#[derive(Component)]
	struct _Executor {
		result: Result<(), _Error>,
		slot_key: Option<SlotKey>,
		assert: Assert,
	}

	impl _Executor {
		fn with_result(result: Result<(), _Error>) -> Self {
			Self {
				result,
				slot_key: None,
				assert: None,
			}
		}

		fn with_slot_key(self, slot_key: Option<SlotKey>) -> Self {
			Self {
				result: self.result,
				slot_key,
				assert: self.assert,
			}
		}

		fn with(
			self,
			assert: impl FnMut(&mut Commands, &SkillCaster, Option<SkillSpawner>, &Target)
				+ Sync
				+ Send
				+ 'static,
		) -> Self {
			Self {
				result: self.result,
				slot_key: self.slot_key,
				assert: Some(Box::new(assert)),
			}
		}
	}

	impl Execute for _Executor {
		type TError = _Error;

		fn execute(
			&mut self,
			commands: &mut Commands,
			caster: &SkillCaster,
			get_spawner: impl Fn(&Option<SlotKey>) -> Option<SkillSpawner>,
			target: &Target,
		) -> Result<(), Self::TError> {
			if let Some(assert) = &mut self.assert {
				assert(commands, caster, get_spawner(&self.slot_key), target);
			}
			self.result
		}
	}

	#[derive(Component)]
	struct _Spawners(HashMap<Option<SlotKey>, SkillSpawner>);

	impl _Spawners {
		fn new<const N: usize>(spawners: [(Option<SlotKey>, SkillSpawner); N]) -> Self {
			Self(HashMap::from(spawners))
		}
	}

	impl Get<Option<SlotKey>, Entity> for _Spawners {
		fn get<'a>(&'a self, key: &Option<SlotKey>) -> Option<&'a Entity> {
			self.0.get(key).map(|s| &s.0)
		}
	}

	fn set_target(app: &mut App) -> Target {
		let cam_ray = Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.));
		app.world_mut().resource_mut::<CamRay>().0 = Some(cam_ray);

		let collider_transform = GlobalTransform::from_xyz(10., 10., 10.);
		let collider = app.world_mut().spawn(collider_transform).id();
		let root_transform = GlobalTransform::from_xyz(11., 11., 11.);
		let root = app.world_mut().spawn(root_transform).id();

		app.world_mut().resource_mut::<MouseHover>().0 = Some(ColliderInfo {
			collider,
			root: Some(root),
		});

		Target {
			ray: cam_ray,
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity: collider,
					component: collider_transform,
				},
				root: Some(Outdated {
					entity: root,
					component: root_transform,
				}),
			}),
		}
	}

	fn set_spawner(app: &mut App) -> SkillSpawner {
		let transform = GlobalTransform::from_xyz(100., 100., 100.);
		let entity = app.world_mut().spawn(transform).id();

		SkillSpawner(entity, transform)
	}

	fn set_caster(app: &mut App, spawners: _Spawners) -> SkillCaster {
		let transform = GlobalTransform::from_xyz(42., 42., 42.);
		let entity = app.world_mut().spawn((transform, spawners)).id();

		SkillCaster(entity, transform)
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<CamRay>();
		app.init_resource::<MouseHover>();
		app.add_systems(
			Update,
			_Executor::execute_on::<_Spawners>.pipe(|_: In<Vec<Result<(), Error>>>| {}),
		);

		app
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Execution {
		caster: SkillCaster,
		spawner: Option<SkillSpawner>,
		target: Target,
	}

	#[test]
	fn execute_skill() {
		let mut app = setup();
		let target = set_target(&mut app);
		let spawner = set_spawner(&mut app);
		let caster = set_caster(&mut app, _Spawners::new([(None, spawner)]));
		app.world_mut().entity_mut(caster.0).insert(
			_Executor::with_result(Ok(())).with_slot_key(None).with(
				|commands, caster, spawner, target| {
					commands.spawn(_Execution {
						caster: *caster,
						spawner,
						target: *target,
					});
				},
			),
		);

		app.update();

		let execution = app
			.world()
			.iter_entities()
			.find_map(|e| e.get::<_Execution>());

		assert_eq!(
			Some(&_Execution {
				caster,
				spawner: Some(spawner),
				target,
			}),
			execution
		);
	}

	#[test]
	fn execute_skill_on_slot() {
		let mut app = setup();
		let target = set_target(&mut app);
		let spawner = set_spawner(&mut app);
		let caster = set_caster(
			&mut app,
			_Spawners::new([(Some(SlotKey::TopHand(Side::Left)), spawner)]),
		);
		app.world_mut().entity_mut(caster.0).insert(
			_Executor::with_result(Ok(()))
				.with_slot_key(Some(SlotKey::TopHand(Side::Left)))
				.with(|commands, caster, spawner, target| {
					commands.spawn(_Execution {
						caster: *caster,
						spawner,
						target: *target,
					});
				}),
		);

		app.update();

		let execution = app
			.world()
			.iter_entities()
			.find_map(|e| e.get::<_Execution>());

		assert_eq!(
			Some(&_Execution {
				caster,
				spawner: Some(spawner),
				target,
			}),
			execution
		);
	}

	#[test]
	fn execute_skill_only_once() {
		static mut COUNT: usize = 0;

		let mut app = setup();
		_ = set_target(&mut app);
		let spawner = set_spawner(&mut app);
		let caster = set_caster(&mut app, _Spawners::new([(None, spawner)]));
		app.world_mut()
			.entity_mut(caster.0)
			.insert(_Executor::with_result(Ok(())).with(|_, _, _, _| unsafe {
				COUNT += 1;
			}));

		app.update();
		app.update();

		assert_eq!(1, unsafe {
			{
				COUNT
			}
		})
	}

	#[test]
	fn execute_again_after_mutable_deref() {
		static mut COUNT: usize = 0;

		let mut app = setup();
		_ = set_target(&mut app);
		let spawner = set_spawner(&mut app);
		let caster = set_caster(&mut app, _Spawners::new([(None, spawner)]));
		app.world_mut()
			.entity_mut(caster.0)
			.insert(_Executor::with_result(Ok(())).with(|_, _, _, _| unsafe {
				COUNT += 1;
			}));

		app.update();

		app.world_mut()
			.entity_mut(caster.0)
			.get_mut::<_Executor>()
			.unwrap()
			.deref_mut();

		app.update();

		assert_eq!(2, unsafe {
			{
				COUNT
			}
		})
	}

	#[test]
	fn return_error() {
		let mut app = setup();
		_ = set_target(&mut app);
		let caster = set_caster(&mut app, _Spawners::new([]));
		app.world_mut()
			.entity_mut(caster.0)
			.insert(_Executor::with_result(Err(_Error("error"))));

		let errors = app
			.world_mut()
			.run_system_once(_Executor::execute_on::<_Spawners>);

		assert_eq!(
			vec![Err(Error {
				msg: "error".to_owned(),
				lvl: Level::Error
			})],
			errors
		)
	}
}
