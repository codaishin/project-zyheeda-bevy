use crate::{
	behaviors::{SkillCaster, SkillSpawner, Target},
	items::slot_key::SlotKey,
	traits::Execute,
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::Changed,
		system::{Commands, Query, Res},
	},
	transform::components::GlobalTransform,
};
use common::{
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
		transforms: Query<(Entity, &GlobalTransform)>,
	) where
		TGetSkillSpawnEntity: Component + Get<Option<SlotKey>, Entity>,
	{
		for (id, mut skill_executer, transform, skill_spawn) in &mut agents {
			let Some(target) = get_target(&cam_ray, &mouse_hover, &transforms) else {
				continue;
			};
			let Some(spawner) = get_spawner(skill_spawn, &transforms) else {
				continue;
			};
			let caster = SkillCaster(id, *transform);
			skill_executer.execute(&mut commands, &caster, &spawner, &target);
		}
	}
}

fn get_target(
	cam_ray: &Res<CamRay>,
	mouse_hover: &Res<MouseHover>,
	transforms: &Query<(Entity, &GlobalTransform)>,
) -> Option<Target> {
	let get_transform = |entity| {
		let Ok((_, transform)) = transforms.get(entity) else {
			return None;
		};
		Some(*transform)
	};

	Some(Target {
		ray: cam_ray.0?,
		collision_info: mouse_hover
			.0
			.as_ref()
			.and_then(|collider_info| collider_info.with_component(get_transform)),
	})
}

fn get_spawner<TGetSkillSpawnEntity>(
	skill_spawn: &TGetSkillSpawnEntity,
	transforms: &Query<(Entity, &GlobalTransform)>,
) -> Option<SkillSpawner>
where
	TGetSkillSpawnEntity: Get<Option<SlotKey>, Entity>,
{
	let skill_spawn = skill_spawn.get(&None)?;
	let Ok((entity, transform)) = transforms.get(*skill_spawn) else {
		return None;
	};

	Some(SkillSpawner(entity, *transform))
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		math::{Ray3d, Vec3},
		transform::components::GlobalTransform,
	};
	use common::{
		components::Outdated,
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::automock;
	use std::{collections::HashMap, ops::DerefMut};

	#[derive(Component, NestedMocks)]
	struct _Executor {
		mock: Mock_Executor,
	}

	#[automock]
	impl Execute for _Executor {
		#[allow(clippy::needless_lifetimes)]
		fn execute<'a, 'b, 'c>(
			&mut self,
			commands: &'a mut Commands<'b, 'c>,
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &Target,
		) {
			self.mock.execute(commands, caster, spawner, target)
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
		app.add_systems(Update, _Executor::execute_on::<_Spawners>);

		app
	}

	#[test]
	fn execute_skill() {
		#[derive(Component, Debug, PartialEq)]
		struct _Execution {
			caster: SkillCaster,
			spawner: SkillSpawner,
			target: Target,
		}

		let mut app = setup();
		let target = set_target(&mut app);
		let spawner = set_spawner(&mut app);
		let caster = set_caster(&mut app, _Spawners::new([(None, spawner)]));
		app.world_mut()
			.entity_mut(caster.0)
			.insert(_Executor::new().with_mock(move |mock| {
				mock.expect_execute()
					.times(1)
					.returning(|commands, caster, spawner, target| {
						commands.spawn(_Execution {
							caster: *caster,
							spawner: *spawner,
							target: *target,
						});
					});
			}));

		app.update();

		let execution = app
			.world()
			.iter_entities()
			.find_map(|e| e.get::<_Execution>());

		assert_eq!(
			Some(&_Execution {
				caster,
				spawner,
				target,
			}),
			execution
		);
	}

	#[test]
	fn execute_skill_only_once() {
		let mut app = setup();
		_ = set_target(&mut app);
		let spawner = set_spawner(&mut app);
		let caster = set_caster(&mut app, _Spawners::new([(None, spawner)]));
		app.world_mut()
			.entity_mut(caster.0)
			.insert(_Executor::new().with_mock(|mock| {
				mock.expect_execute().times(1).return_const(());
			}));

		app.update();
		app.update();
	}

	#[test]
	fn execute_again_after_mutable_deref() {
		let mut app = setup();
		_ = set_target(&mut app);
		let spawner = set_spawner(&mut app);
		let caster = set_caster(&mut app, _Spawners::new([(None, spawner)]));
		app.world_mut()
			.entity_mut(caster.0)
			.insert(_Executor::new().with_mock(|mock| {
				mock.expect_execute().times(2).return_const(());
			}));

		app.update();

		app.world_mut()
			.entity_mut(caster.0)
			.get_mut::<_Executor>()
			.unwrap()
			.deref_mut();

		app.update();
	}
}
