use crate::{
	behaviors::{SkillCaster, Target},
	components::skill_spawners::SkillSpawners,
	traits::Execute,
};
use bevy::prelude::*;
use common::{
	errors::Error,
	resources::{CamRay, MouseHover},
};

impl<T> ExecuteSkills for T {}

pub(crate) trait ExecuteSkills {
	fn execute_system(
		cam_ray: Res<CamRay>,
		mouse_hover: Res<MouseHover>,
		mut commands: Commands,
		mut agents: Query<(Entity, &mut Self, &SkillSpawners), Changed<Self>>,
		transforms: Query<&GlobalTransform>,
	) -> Vec<Result<(), Error>>
	where
		for<'w, 's> Self: Component + Execute<Commands<'w, 's>> + Sized,
		for<'w, 's> Error: From<<Self as Execute<Commands<'w, 's>>>::TError>,
	{
		agents
			.iter_mut()
			.map(|(entity, mut skill_executer, skill_spawners)| {
				match get_target(&cam_ray, &mouse_hover, &transforms) {
					None => Ok(()),
					Some(target) => skill_executer.execute(
						&mut commands,
						&SkillCaster(entity),
						skill_spawners,
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
	use crate::{
		behaviors::SkillSpawner,
		components::skill_spawners::SkillSpawners,
		items::slot_key::SlotKey,
	};
	use bevy::ecs::system::RunSystemOnce;
	use common::{
		components::{Outdated, Side},
		errors::Level,
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::mock;
	use std::ops::DerefMut;

	#[derive(Clone, Copy)]
	pub struct _Error(&'static str);

	impl From<_Error> for Error {
		fn from(_Error(msg): _Error) -> Self {
			Self {
				msg: msg.to_owned(),
				lvl: Level::Error,
			}
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Executor {
		mock: Mock_Executor,
	}

	impl<'w, 's> Execute<Commands<'w, 's>> for _Executor {
		type TError = _Error;

		fn execute(
			&mut self,
			commands: &mut Commands,
			caster: &SkillCaster,
			spawners: &SkillSpawners,
			target: &Target,
		) -> Result<(), Self::TError> {
			self.mock.execute(commands, caster, spawners, target)
		}
	}

	mock! {
		_Executor {}
		impl<'w, 's> Execute<Commands<'w, 's>> for _Executor {
			type TError = _Error;

			fn execute<'_w, '_s>(
				&mut self,
				commands: &mut Commands<'_w, '_s>,
				caster: &SkillCaster,
				spawners: &SkillSpawners,
				target: &Target,
			) -> Result<(), _Error>;
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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<CamRay>();
		app.init_resource::<MouseHover>();
		app.add_systems(
			Update,
			_Executor::execute_system.pipe(|_: In<Vec<Result<(), Error>>>| {}),
		);

		app
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ExecutionArgs {
		caster: SkillCaster,
		spawners: SkillSpawners,
		target: Target,
	}

	fn find_execution_args(app: &App) -> Option<&_ExecutionArgs> {
		app.world()
			.iter_entities()
			.find_map(|e| e.get::<_ExecutionArgs>())
	}

	#[test]
	fn execute_skill() {
		let mut app = setup();
		let target = set_target(&mut app);
		let spawners = SkillSpawners::new([
			(
				Some(SlotKey::BottomHand(Side::Left)),
				SkillSpawner(Entity::from_raw(42)),
			),
			(
				Some(SlotKey::TopHand(Side::Right)),
				SkillSpawner(Entity::from_raw(43)),
			),
		]);
		let caster = app.world_mut().spawn(spawners.clone()).id();
		app.world_mut()
			.entity_mut(caster)
			.insert(_Executor::new().with_mock(|mock| {
				mock.expect_execute()
					.returning(|commands, caster, spawners, target| {
						commands.spawn(_ExecutionArgs {
							caster: *caster,
							spawners: spawners.clone(),
							target: *target,
						});
						Ok(())
					});
			}));

		app.update();

		assert_eq!(
			Some(&_ExecutionArgs {
				caster: SkillCaster(caster),
				spawners,
				target,
			}),
			find_execution_args(&app)
		);
	}

	#[test]
	fn execute_skill_only_once() {
		let mut app = setup();
		set_target(&mut app);
		app.world_mut().spawn((
			_Executor::new().with_mock(|mock| {
				mock.expect_execute().times(1).return_const(Ok(()));
			}),
			SkillSpawners::new([]),
		));

		app.update();
		app.update();
	}

	#[test]
	fn execute_again_after_mutable_deref() {
		let mut app = setup();
		set_target(&mut app);
		let caster = app
			.world_mut()
			.spawn((
				_Executor::new().with_mock(|mock| {
					mock.expect_execute().times(2).return_const(Ok(()));
				}),
				SkillSpawners::new([]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(caster)
			.get_mut::<_Executor>()
			.unwrap()
			.deref_mut();
		app.update();
	}

	#[test]
	fn return_error() {
		let mut app = setup();
		set_target(&mut app);
		app.world_mut().spawn((
			_Executor::new().with_mock(|mock| {
				mock.expect_execute().return_const(Err(_Error("error")));
			}),
			SkillSpawners::new([]),
		));

		app.update();

		let errors = app.world_mut().run_system_once(_Executor::execute_system);

		assert_eq!(
			vec![Err(Error {
				msg: "error".to_owned(),
				lvl: Level::Error
			})],
			errors
		)
	}
}
