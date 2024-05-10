use crate::{
	components::{SkillExecution, SkillSpawn},
	skills::{SelectInfo, SkillCaster, SkillSpawner, StartBehaviorFn, Target},
};
use bevy::{
	ecs::{
		entity::Entity,
		system::{Commands, EntityCommands, Query, Res},
	},
	math::Ray3d,
	transform::components::{GlobalTransform, Transform},
};
use common::{
	components::Outdated,
	resources::{CamRay, MouseHover},
};

pub(crate) fn apply_skill_behavior(
	mut commands: Commands,
	cam_ray: Res<CamRay>,
	mouse_hover: Res<MouseHover>,
	agents: Query<(Entity, &Transform, &SkillExecution, &SkillSpawn<Entity>)>,
	transforms: Query<&GlobalTransform>,
) {
	for (id, transform, execution, skill_spawn) in &agents {
		let Some(agent) = &mut commands.get_entity(id) else {
			continue;
		};
		match execution {
			SkillExecution::Start(start_fn) => {
				start_behavior(
					agent,
					&cam_ray,
					&mouse_hover,
					skill_spawn,
					transform,
					&transforms,
					start_fn,
				);
			}
			SkillExecution::Stop(stop_fn) => {
				stop_fn(agent);
			}
		}
		agent.remove::<SkillExecution>();
	}
}

fn start_behavior(
	agent: &mut EntityCommands,
	cam_ray: &Res<CamRay>,
	mouse_hover: &Res<MouseHover>,
	skill_spawn: &SkillSpawn<Entity>,
	skill_caster: &Transform,
	transforms: &Query<&GlobalTransform, ()>,
	start_fn: &StartBehaviorFn,
) {
	let Some(ray) = cam_ray.0 else {
		return;
	};
	let Some(spawner) = get_spawner(skill_spawn, transforms) else {
		return;
	};
	let target = get_target(ray, mouse_hover, transforms);
	start_fn(agent, &SkillCaster(*skill_caster), &spawner, &target);
}

fn get_target(
	ray: Ray3d,
	mouse_hover: &Res<MouseHover>,
	transforms: &Query<&GlobalTransform>,
) -> SelectInfo<Outdated<GlobalTransform>> {
	Target {
		ray,
		collision_info: mouse_hover
			.0
			.as_ref()
			.and_then(|collider_info| collider_info.with_component(transforms)),
	}
}

fn get_spawner(
	skill_spawn: &SkillSpawn<Entity>,
	transforms: &Query<&GlobalTransform>,
) -> Option<SkillSpawner> {
	let Ok(transform) = transforms.get(skill_spawn.0) else {
		return None;
	};

	Some(SkillSpawner(*transform))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::SkillExecution,
		skills::{SkillSpawner, Target},
	};
	use bevy::{
		app::{App, Update},
		ecs::system::EntityCommands,
		math::{Ray3d, Vec3},
		transform::components::{GlobalTransform, Transform},
	};
	use common::{
		components::Outdated,
		resources::{CamRay, ColliderInfo, MouseHover},
		test_tools::utils::SingleThreadedApp,
	};
	use mockall::mock;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<CamRay>();
		app.init_resource::<MouseHover>();
		app.add_systems(Update, apply_skill_behavior);

		app
	}

	trait StartFn {
		fn start(
			agent: &mut EntityCommands,
			transform: &SkillCaster,
			spawner: &SkillSpawner,
			target: &Target,
		);
	}

	trait StopFn {
		fn stop(agent: &mut EntityCommands);
	}

	macro_rules! mock_fns {
		($ident:ident) => {
			mock! {
				$ident {}
				impl StartFn for $ident {
					#[allow(clippy::needless_lifetimes)]
					fn start<'a>(agent: &mut EntityCommands<'a>, caster: &SkillCaster, spawner: &SkillSpawner, target: &Target) {}
				}
				impl StopFn for $ident {
					#[allow(clippy::needless_lifetimes)]
					fn stop<'a>(agent: &mut EntityCommands<'a>) {}
				}
			}
		};
	}

	mock_fns!(_Run);

	#[test]
	fn start_behavior() {
		let mut app = setup();

		let cam_ray = Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.));
		app.world.resource_mut::<CamRay>().0 = Some(cam_ray);

		let collider_transform = GlobalTransform::from_xyz(10., 10., 10.);
		let collider = app.world.spawn(collider_transform).id();
		let root_transform = GlobalTransform::from_xyz(11., 11., 11.);
		let root = app.world.spawn(root_transform).id();
		let collider_info = ColliderInfo {
			collider,
			root: Some(root),
		};
		app.world.resource_mut::<MouseHover>().0 = Some(collider_info);

		let spawner_transform = GlobalTransform::from_xyz(100., 100., 100.);
		let spawner = app.world.spawn(spawner_transform).id();

		let skill_caster = SkillCaster(Transform::from_xyz(42., 42., 42.));
		let agent = app
			.world
			.spawn((
				skill_caster.0,
				SkillSpawn(spawner),
				SkillExecution::Start(Mock_Run::start),
			))
			.id();

		let start_ctx = Mock_Run::start_context();
		start_ctx
			.expect()
			.times(1)
			.withf(move |agent_cmds, transform, spawner, target| {
				assert_eq!(
					(
						agent,
						&skill_caster,
						&SkillSpawner(spawner_transform),
						&Target {
							ray: cam_ray,
							collision_info: Some(ColliderInfo {
								collider: Outdated {
									entity: collider,
									component: collider_transform,
								},
								root: Some(Outdated {
									entity: root,
									component: root_transform
								})
							})
						}
					),
					(agent_cmds.id(), transform, spawner, target)
				);
				true
			})
			.return_const(());

		app.update();
	}

	mock_fns!(_Stop);

	#[test]
	fn stop_behavior() {
		let mut app = setup();

		let agent = app
			.world
			.spawn((
				Transform::default(),
				SkillSpawn(Entity::from_raw(101)),
				SkillExecution::Stop(Mock_Stop::stop),
			))
			.id();

		let stop_ctx = Mock_Stop::stop_context();
		stop_ctx
			.expect()
			.times(1)
			.withf(move |agent_cmds| {
				assert_eq!(agent, agent_cmds.id());
				true
			})
			.return_const(());

		app.update();
	}

	mock_fns!(_RemoveOnStart);

	#[test]
	fn remove_skill_execution_component_on_start() {
		let mut app = setup();

		let agent = app
			.world
			.spawn((
				Transform::default(),
				SkillSpawn(Entity::from_raw(101)),
				SkillExecution::Start(Mock_RemoveOnStart::start),
			))
			.id();

		let start_ctx = Mock_RemoveOnStart::start_context();
		start_ctx.expect().return_const(());

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<SkillExecution>());
	}

	mock_fns!(_RemoveOnStop);

	#[test]
	fn remove_skill_execution_component_on_stop() {
		let mut app = setup();

		let agent = app
			.world
			.spawn((
				Transform::default(),
				SkillSpawn(Entity::from_raw(101)),
				SkillExecution::Stop(Mock_RemoveOnStop::stop),
			))
			.id();

		let stop_ctx = Mock_RemoveOnStop::stop_context();
		stop_ctx.expect().return_const(());

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<SkillExecution>());
	}
}
