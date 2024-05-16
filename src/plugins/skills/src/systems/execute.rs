use crate::{
	components::SkillSpawn,
	skills::{SkillCaster, SkillSpawner, Target},
	traits::Execute,
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::Changed,
		system::{Commands, Query, Res},
	},
	transform::components::{GlobalTransform, Transform},
};
use common::resources::{CamRay, MouseHover};

type Components<'a, TSkillExecutor> = (
	Entity,
	&'a mut TSkillExecutor,
	&'a Transform,
	&'a SkillSpawn<Entity>,
);

pub(crate) fn execute<TSkillExecutor: Component + Execute>(
	cam_ray: Res<CamRay>,
	mouse_hover: Res<MouseHover>,
	mut commands: Commands,
	mut agents: Query<Components<TSkillExecutor>, Changed<TSkillExecutor>>,
	transforms: Query<(Entity, &GlobalTransform)>,
) {
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

fn get_spawner(
	skill_spawn: &SkillSpawn<Entity>,
	transforms: &Query<(Entity, &GlobalTransform)>,
) -> Option<SkillSpawner> {
	let Ok((entity, transform)) = transforms.get(skill_spawn.0) else {
		return None;
	};

	Some(SkillSpawner(entity, *transform))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skills::{SkillCaster, SkillSpawner, Target};
	use bevy::{
		app::{App, Update},
		math::{Ray3d, Vec3},
		transform::components::GlobalTransform,
	};
	use common::{
		components::Outdated,
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
	};
	use mockall::automock;

	#[derive(Component, Default)]
	struct _Executor {
		value: usize,
		mock: Mock_Executor,
	}

	impl _Executor {
		fn change(&mut self) {
			self.value += 1;
		}
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

	fn set_target(app: &mut App) -> Target {
		let cam_ray = Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.));
		app.world.resource_mut::<CamRay>().0 = Some(cam_ray);

		let collider_transform = GlobalTransform::from_xyz(10., 10., 10.);
		let collider = app.world.spawn(collider_transform).id();
		let root_transform = GlobalTransform::from_xyz(11., 11., 11.);
		let root = app.world.spawn(root_transform).id();

		app.world.resource_mut::<MouseHover>().0 = Some(ColliderInfo {
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
		let entity = app.world.spawn(transform).id();

		SkillSpawner(entity, transform)
	}

	fn set_caster(app: &mut App, spawner: &SkillSpawner) -> SkillCaster {
		let transform = Transform::from_xyz(42., 42., 42.);
		let entity = app.world.spawn((transform, SkillSpawn(spawner.0))).id();

		SkillCaster(entity, transform)
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<CamRay>();
		app.init_resource::<MouseHover>();
		app.add_systems(Update, execute::<_Executor>);

		app
	}

	#[test]
	fn execute_skill() {
		let mut app = setup();
		let target = set_target(&mut app);
		let spawner = set_spawner(&mut app);
		let caster = set_caster(&mut app, &spawner);

		let mut executer = _Executor::default();
		executer
			.mock
			.expect_execute()
			.times(1)
			.withf(move |_, caster_a, spawner_a, target_a| {
				assert_eq!(
					(&caster, &spawner, &target),
					(caster_a, spawner_a, target_a)
				);
				true
			})
			.return_const(());

		app.world.entity_mut(caster.0).insert(executer);

		app.update();
	}

	#[test]
	fn execute_skill_only_once() {
		let mut app = setup();
		_ = set_target(&mut app);
		let spawner = set_spawner(&mut app);
		let caster = set_caster(&mut app, &spawner);

		let mut executer = _Executor::default();
		executer.mock.expect_execute().times(1).return_const(());

		app.world.entity_mut(caster.0).insert(executer);

		app.update();
		app.update();
	}

	#[test]
	fn execute_again_after_change() {
		let mut app = setup();
		_ = set_target(&mut app);
		let spawner = set_spawner(&mut app);
		let caster = set_caster(&mut app, &spawner);

		let mut executer = _Executor::default();
		executer.mock.expect_execute().times(2).return_const(());

		app.world.entity_mut(caster.0).insert(executer);

		app.update();

		let mut caster = app.world.entity_mut(caster.0);
		let mut executer = caster.get_mut::<_Executor>().unwrap();
		executer.change();

		app.update();
	}
}
