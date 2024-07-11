use super::{RunSkill, SkillBundleConfig};
use crate::skills::{OnSkillStop, SkillCaster, SkillSpawner, Target};
use bevy::ecs::system::Commands;

impl<T: SkillBundleConfig> RunSkill for T {
	fn run_skill(
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> OnSkillStop {
		let entity = commands
			.spawn(T::new_skill_bundle(caster, spawner, target))
			.id();

		if Self::STOPPABLE {
			OnSkillStop::Stop(entity)
		} else {
			OnSkillStop::Ignore
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::skills::SelectInfo;
	use bevy::{
		app::{App, Update},
		ecs::{
			bundle::Bundle,
			component::Component,
			entity::Entity,
			system::{Commands, Query, Resource},
		},
		math::{Ray3d, Vec3},
		transform::components::{GlobalTransform, Transform},
	};
	use common::{
		components::Outdated,
		resources::ColliderInfo,
		test_tools::utils::SingleThreadedApp,
	};

	#[derive(Component, Debug, PartialEq)]
	struct _Skill<const STOPPABLE: bool> {
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: Target,
	}

	impl<const STOPPABLE: bool> SkillBundleConfig for _Skill<STOPPABLE> {
		const STOPPABLE: bool = STOPPABLE;

		fn new_skill_bundle(
			caster: &SkillCaster,
			spawner: &SkillSpawner,
			target: &Target,
		) -> impl Bundle {
			_Skill::<STOPPABLE> {
				caster: *caster,
				spawner: *spawner,
				target: target.clone(),
			}
		}
	}

	#[derive(Resource)]
	struct _Result(OnSkillStop);

	fn setup<const STOPPABLE: bool>(
		caster: Transform,
		spawner: SkillSpawner,
		target: Target,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			move |mut commands: Commands, query: Query<Entity>| {
				for id in &query {
					let id = _Skill::<STOPPABLE>::run_skill(
						&mut commands,
						&SkillCaster(id, caster),
						&spawner,
						&target,
					);
					commands.insert_resource(_Result(id));
				}
			},
		);

		app
	}

	#[test]
	fn spawn_not_on_agent() {
		let caster_transform = Transform::from_xyz(1., 2., 3.);
		let spawner = SkillSpawner(
			Entity::from_raw(1000),
			GlobalTransform::from_xyz(4., 5., 6.),
		);
		let target = SelectInfo {
			ray: Ray3d::new(Vec3::ONE, Vec3::ONE),
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity: Entity::from_raw(42),
					component: GlobalTransform::from_xyz(7., 8., 9.),
				},
				root: None,
			}),
		};
		let mut app = setup::<true>(caster_transform, spawner, target.clone());
		let agent = app.world_mut().spawn_empty().id();

		app.update();

		let skill = app
			.world()
			.iter_entities()
			.find(|e| e.id() != agent)
			.unwrap();

		assert_eq!(
			Some(&_Skill {
				caster: SkillCaster(agent, caster_transform),
				spawner,
				target,
			}),
			skill.get::<_Skill<true>>()
		);
	}

	#[test]
	fn returned_spawned_entity() {
		let caster = Transform::from_xyz(1., 2., 3.);
		let spawner = SkillSpawner(
			Entity::from_raw(1000),
			GlobalTransform::from_xyz(4., 5., 6.),
		);
		let target = SelectInfo {
			ray: Ray3d::new(Vec3::ONE, Vec3::ONE),
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity: Entity::from_raw(42),
					component: GlobalTransform::from_xyz(7., 8., 9.),
				},
				root: None,
			}),
		};
		let mut app = setup::<true>(caster, spawner, target.clone());
		app.world_mut().spawn_empty();

		app.update();

		let skill = app
			.world()
			.iter_entities()
			.find(|e| e.contains::<_Skill<true>>())
			.unwrap();
		let result = app.world().get_resource::<_Result>().unwrap();

		assert_eq!(OnSkillStop::Stop(skill.id()), result.0);
	}

	#[test]
	fn do_not_return_spawned_entity_when_stoppable_false() {
		let caster = Transform::from_xyz(1., 2., 3.);
		let spawner = SkillSpawner(
			Entity::from_raw(1000),
			GlobalTransform::from_xyz(4., 5., 6.),
		);
		let target = SelectInfo {
			ray: Ray3d::new(Vec3::ONE, Vec3::ONE),
			collision_info: Some(ColliderInfo {
				collider: Outdated {
					entity: Entity::from_raw(42),
					component: GlobalTransform::from_xyz(7., 8., 9.),
				},
				root: None,
			}),
		};
		let mut app = setup::<false>(caster, spawner, target.clone());
		app.world_mut().spawn_empty();

		app.update();

		let result = app.world().get_resource::<_Result>().unwrap();

		assert_eq!(OnSkillStop::Ignore, result.0);
	}
}
