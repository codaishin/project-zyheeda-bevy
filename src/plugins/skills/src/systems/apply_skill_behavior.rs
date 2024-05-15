use crate::{
	components::{SkillExecution, SkillRunningOn, SkillSpawn},
	skills::{OnSkillStop, SelectInfo, SkillCaster, SkillSpawner, StartBehaviorFn, Target},
};
use bevy::{
	ecs::{
		entity::Entity,
		system::{Commands, Query, Res},
	},
	hierarchy::DespawnRecursiveExt,
	transform::components::{GlobalTransform, Transform},
};
use common::{
	components::Outdated,
	resources::{CamRay, MouseHover},
	traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};

type Components<'a> = (
	Entity,
	&'a Transform,
	&'a SkillExecution,
	&'a SkillSpawn<Entity>,
	Option<&'a SkillRunningOn>,
);

pub(crate) fn apply_skill_behavior(
	mut commands: Commands,
	cam_ray: Res<CamRay>,
	mouse_hover: Res<MouseHover>,
	agents: Query<Components>,
	transforms: Query<(Entity, &GlobalTransform)>,
) {
	for (id, transform, execution, skill_spawn, skill_running_on) in &agents {
		match (execution, skill_running_on) {
			(SkillExecution::Start(start_fn), ..) => {
				let spawner = get_spawner(skill_spawn, &transforms);
				let target = get_target(&cam_ray, &mouse_hover, &transforms);
				start_behavior(&mut commands, id, start_fn, transform, spawner, target);
			}
			(SkillExecution::Stop, Some(SkillRunningOn(skill_id))) => {
				stop_behavior(&mut commands, id, skill_id);
			}
			_ => {}
		}
		commands.try_remove_from::<SkillExecution>(id);
	}
}

fn start_behavior(
	commands: &mut Commands,
	id: Entity,
	start_fn: &StartBehaviorFn,
	caster_transform: &Transform,
	spawner: Option<SkillSpawner>,
	target: Option<Target>,
) {
	let Some(spawner) = spawner else {
		return;
	};
	let Some(target) = target else {
		return;
	};
	let OnSkillStop::Stop(skill_id) = start_fn(
		commands,
		&SkillCaster(id, *caster_transform),
		&spawner,
		&target,
	) else {
		return;
	};
	commands.try_insert_on(id, SkillRunningOn(skill_id));
}

fn stop_behavior(commands: &mut Commands, caster: Entity, skill_id: &Entity) {
	if let Some(skill) = commands.get_entity(*skill_id) {
		skill.despawn_recursive();
	}
	commands.try_remove_from::<SkillRunningOn>(caster);
}

fn get_target(
	cam_ray: &Res<CamRay>,
	mouse_hover: &Res<MouseHover>,
	transforms: &Query<(Entity, &GlobalTransform)>,
) -> Option<SelectInfo<Outdated<GlobalTransform>>> {
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
	use crate::{
		components::SkillExecution,
		skills::{OnSkillStop, SkillSpawner, Target},
	};
	use bevy::{
		app::{App, Update},
		hierarchy::BuildWorldChildren,
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
			agent: &mut Commands,
			transform: &SkillCaster,
			spawner: &SkillSpawner,
			target: &Target,
		) -> OnSkillStop;
	}

	macro_rules! mock_fns {
		($ident:ident) => {
			mock! {
				$ident {}
				impl StartFn for $ident {
					#[allow(clippy::needless_lifetimes)]
					fn start<'a, 'b>(agent: &mut Commands<'a, 'b>, caster: &SkillCaster, spawner: &SkillSpawner, target: &Target) -> OnSkillStop {}
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
		let spawner_entity = app.world.spawn(spawner_transform).id();

		let skill_caster_transform = Transform::from_xyz(42., 42., 42.);
		let agent_entity = app
			.world
			.spawn((
				skill_caster_transform,
				SkillSpawn(spawner_entity),
				SkillExecution::Start(Mock_Run::start),
			))
			.id();

		let start_ctx = Mock_Run::start_context();
		start_ctx
			.expect()
			.times(1)
			.withf(move |_, caster, spawner, target| {
				assert_eq!(
					(
						&SkillCaster(agent_entity, skill_caster_transform),
						&SkillSpawner(spawner_entity, spawner_transform),
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
					(caster, spawner, target)
				);
				true
			})
			.return_const(OnSkillStop::Stop(Entity::from_raw(42)));

		app.update();

		let agent = app.world.entity(agent_entity);

		assert_eq!(
			Some(&SkillRunningOn(Entity::from_raw(42))),
			agent.get::<SkillRunningOn>()
		);
	}

	#[test]
	fn stop_behavior_by_recursively_despawning_the_skill_entity() {
		let mut app = setup();
		let skill = app.world.spawn_empty().id();
		let skill_child = app.world.spawn_empty().set_parent(skill).id();

		app.world.spawn((
			Transform::default(),
			SkillSpawn(Entity::from_raw(101)),
			SkillExecution::Stop,
			SkillRunningOn(skill),
		));

		app.update();

		let skill_or_skill_child: Vec<_> = app
			.world
			.iter_entities()
			.filter(|e| e.id() == skill || e.id() == skill_child)
			.map(|e| e.id())
			.collect();

		assert_eq!(vec![] as Vec<Entity>, skill_or_skill_child);
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
		start_ctx
			.expect()
			.return_const(OnSkillStop::Stop(Entity::from_raw(42)));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<SkillExecution>());
	}

	#[test]
	fn remove_skill_execution_component_on_stop() {
		let mut app = setup();

		let agent = app
			.world
			.spawn((
				Transform::default(),
				SkillSpawn(Entity::from_raw(101)),
				SkillExecution::Stop,
				SkillRunningOn(Entity::from_raw(2029)),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<SkillRunningOn>());
	}

	#[test]
	fn remove_skill_running_component_on_stop() {
		let mut app = setup();

		let agent = app
			.world
			.spawn((
				Transform::default(),
				SkillSpawn(Entity::from_raw(101)),
				SkillExecution::Stop,
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<SkillExecution>());
	}
}
