use crate::{
	behaviors::SkillCaster,
	components::{skill_spawners::SkillSpawners, SkillTarget},
	traits::Execute,
};
use bevy::prelude::*;
use common::{
	effects::deal_damage::DealDamage,
	errors::Error,
	tools::collider_info::ColliderInfo,
	traits::{
		accessors::get::GetterRefOptional,
		handles_effect::HandlesEffect,
		handles_lifetime::HandlesLifetime,
		handles_player::{HandlesPlayerCameras, HandlesPlayerMouse},
		handles_skill_behaviors::HandlesSkillBehaviors,
	},
};

impl<T> ExecuteSkills for T {}

pub(crate) trait ExecuteSkills {
	fn execute_system<TLifetimes, TEffects, TSkillBehaviors, TPlayers>(
		cam_ray: Res<TPlayers::TCamRay>,
		mouse_hover: Res<TPlayers::TMouseHover>,
		mut commands: Commands,
		mut agents: Query<(Entity, &mut Self, &SkillSpawners), Changed<Self>>,
		transforms: Query<&GlobalTransform>,
	) -> Vec<Result<(), Error>>
	where
		for<'w, 's> Self:
			Component + Execute<Commands<'w, 's>, TLifetimes, TEffects, TSkillBehaviors> + Sized,
		for<'w, 's> Error: From<
			<Self as Execute<Commands<'w, 's>, TLifetimes, TEffects, TSkillBehaviors>>::TError,
		>,
		TLifetimes: HandlesLifetime,
		TEffects: HandlesEffect<DealDamage>,
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
		TPlayers: HandlesPlayerCameras + HandlesPlayerMouse,
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

fn get_target<TCamRay, TMouseHover>(
	cam_ray: &Res<TCamRay>,
	mouse_hover: &Res<TMouseHover>,
	transforms: &Query<&GlobalTransform>,
) -> Option<SkillTarget>
where
	TCamRay: Resource + GetterRefOptional<Ray3d>,
	TMouseHover: Resource + GetterRefOptional<ColliderInfo<Entity>>,
{
	let get_transform = |entity| transforms.get(entity).ok().cloned();

	Some(SkillTarget {
		ray: cam_ray.get().cloned()?,
		collision_info: mouse_hover
			.get()
			.and_then(|collider_info| collider_info.with_component(get_transform)),
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{behaviors::SkillSpawner, components::skill_spawners::SkillSpawners};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		components::Outdated,
		errors::Level,
		test_tools::utils::SingleThreadedApp,
		tools::{
			collider_info::ColliderInfo,
			slot_key::{Side, SlotKey},
		},
		traits::{
			handles_skill_behaviors::{Integrity, Motion, ProjectionOffset, Shape},
			intersect_at::IntersectAt,
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::mock;
	use std::{ops::DerefMut, time::Duration};

	struct _Players;

	impl HandlesPlayerCameras for _Players {
		type TCamRay = _CamRay;
	}

	impl HandlesPlayerMouse for _Players {
		type TMouseHover = _MouseHover;
	}

	#[derive(Resource, Default)]
	pub struct _CamRay(Option<Ray3d>);

	impl GetterRefOptional<Ray3d> for _CamRay {
		fn get(&self) -> Option<&Ray3d> {
			self.0.as_ref()
		}
	}

	impl IntersectAt for _CamRay {
		fn intersect_at(&self, _: f32) -> Option<Vec3> {
			panic!("should not be called")
		}
	}

	#[derive(Resource, Default)]
	pub struct _MouseHover(Option<ColliderInfo<Entity>>);

	impl GetterRefOptional<ColliderInfo<Entity>> for _MouseHover {
		fn get(&self) -> Option<&ColliderInfo<Entity>> {
			self.0.as_ref()
		}
	}

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

	struct _HandlesLife;

	impl HandlesLifetime for _HandlesLife {
		fn lifetime(_: Duration) -> impl Bundle {}
	}

	struct _HandlesEffects;

	impl<T> HandlesEffect<T> for _HandlesEffects {
		type TTarget = ();
		type TEffectComponent = _Effect;

		fn effect(_: T) -> _Effect {
			_Effect
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	#[derive(Component)]
	struct _Effect;

	struct _HandlesSkillBehaviors;

	impl HandlesSkillBehaviors for _HandlesSkillBehaviors {
		type TSkillContact = _Contact;

		type TSkillProjection = _Projection;

		fn skill_contact(_: Shape, _: Integrity, _: Motion) -> Self::TSkillContact {
			_Contact
		}

		fn skill_projection(_: Shape, _: Option<ProjectionOffset>) -> Self::TSkillProjection {
			_Projection
		}
	}

	#[derive(Component)]
	struct _Contact;

	#[derive(Component)]
	struct _Projection;

	impl Execute<Commands<'_, '_>, _HandlesLife, _HandlesEffects, _HandlesSkillBehaviors>
		for _Executor
	{
		type TError = _Error;

		fn execute(
			&mut self,
			commands: &mut Commands,
			caster: &SkillCaster,
			spawners: &SkillSpawners,
			target: &SkillTarget,
		) -> Result<(), Self::TError> {
			self.mock.execute(commands, caster, spawners, target)
		}
	}

	mock! {
		_Executor {}
		impl<'w, 's> Execute<
			Commands<'w, 's>,
			_HandlesLife,
			_HandlesEffects,
			_HandlesSkillBehaviors
		> for _Executor {
			type TError = _Error;

			fn execute<'_w, '_s>(
				&mut self,
				commands: &mut Commands<'_w, '_s>,
				caster: &SkillCaster,
				spawners: &SkillSpawners,
				target: &SkillTarget,
			) -> Result<(), _Error>;
		}
	}

	fn set_target(app: &mut App) -> SkillTarget {
		let cam_ray = Ray3d::new(
			Vec3::new(1., 2., 3.),
			Dir3::new_unchecked(Vec3::new(4., 5., 6.).normalize()),
		);
		app.world_mut().resource_mut::<_CamRay>().0 = Some(cam_ray);

		let collider_transform = GlobalTransform::from_xyz(10., 10., 10.);
		let collider = app.world_mut().spawn(collider_transform).id();
		let root_transform = GlobalTransform::from_xyz(11., 11., 11.);
		let root = app.world_mut().spawn(root_transform).id();

		app.world_mut().resource_mut::<_MouseHover>().0 = Some(ColliderInfo {
			collider,
			root: Some(root),
		});

		SkillTarget {
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
		let execute_system = _Executor::execute_system::<
			_HandlesLife,
			_HandlesEffects,
			_HandlesSkillBehaviors,
			_Players,
		>;

		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_CamRay>();
		app.init_resource::<_MouseHover>();
		app.add_systems(
			Update,
			execute_system.pipe(|_: In<Vec<Result<(), Error>>>| {}),
		);

		app
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ExecutionArgs {
		caster: SkillCaster,
		spawners: SkillSpawners,
		target: SkillTarget,
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
	fn return_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		set_target(&mut app);
		app.world_mut().spawn((
			_Executor::new().with_mock(|mock| {
				mock.expect_execute().return_const(Err(_Error("error")));
			}),
			SkillSpawners::new([]),
		));

		app.update();

		let errors = app.world_mut().run_system_once(
			_Executor::execute_system::<
				_HandlesLife,
				_HandlesEffects,
				_HandlesSkillBehaviors,
				_Players,
			>,
		)?;

		assert_eq!(
			vec![Err(Error {
				msg: "error".to_owned(),
				lvl: Level::Error
			})],
			errors
		);
		Ok(())
	}
}
