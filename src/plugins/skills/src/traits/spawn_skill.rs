use super::{SkillBundleConfig, SpawnSkill};
use crate::behaviors::{spawn_behavior::OnSkillStop, SkillCaster, SkillSpawner, Target};
use bevy::ecs::system::{Commands, EntityCommands};

impl<T: SkillBundleConfig> SpawnSkill for T {
	fn spawn_skill<'a>(
		commands: &'a mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> (EntityCommands<'a>, OnSkillStop) {
		let entity = commands.spawn(T::new_skill_bundle(caster, spawner, target));

		if Self::STOPPABLE {
			let id = entity.id();
			(entity, OnSkillStop::Stop(id))
		} else {
			(entity, OnSkillStop::Ignore)
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
		transform::components::GlobalTransform,
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
				target: *target,
			}
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Entity, OnSkillStop);

	fn setup<const STOPPABLE: bool>(spawner: SkillSpawner, target: Target) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			move |mut commands: Commands, casters: Query<(Entity, &GlobalTransform)>| {
				for (caster, transform) in &casters {
					let (entity, on_skill_stop) = _Skill::<STOPPABLE>::spawn_skill(
						&mut commands,
						&SkillCaster(caster, *transform),
						&spawner,
						&target,
					);
					let entity_id = entity.id();
					commands.insert_resource(_Result(entity_id, on_skill_stop));
				}
			},
		);

		app
	}

	#[test]
	fn spawn_not_on_agent() {
		let caster_transform = GlobalTransform::from_xyz(1., 2., 3.);
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
		let mut app = setup::<true>(spawner, target);
		let caster = app.world_mut().spawn(caster_transform).id();

		app.update();

		let skill = app
			.world()
			.iter_entities()
			.find(|e| e.id() != caster)
			.unwrap();

		assert_eq!(
			Some(&_Skill {
				caster: SkillCaster(caster, caster_transform),
				spawner,
				target,
			}),
			skill.get::<_Skill<true>>()
		);
	}

	#[test]
	fn returned_spawned_entity() {
		let caster_transform = GlobalTransform::from_xyz(1., 2., 3.);
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
		let mut app = setup::<true>(spawner, target);
		app.world_mut().spawn(caster_transform);

		app.update();

		let skill = app
			.world()
			.iter_entities()
			.find(|e| e.contains::<_Skill<true>>())
			.unwrap();
		let result = app.world().get_resource::<_Result>().unwrap();

		assert_eq!(&_Result(skill.id(), OnSkillStop::Stop(skill.id())), result);
	}

	#[test]
	fn do_not_return_spawned_entity_when_stoppable_false() {
		let caster_transform = GlobalTransform::from_xyz(1., 2., 3.);
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
		let mut app = setup::<false>(spawner, target);
		app.world_mut().spawn(caster_transform);

		app.update();

		let skill = app
			.world()
			.iter_entities()
			.find(|e| e.contains::<_Skill<false>>())
			.unwrap();
		let result = app.world().get_resource::<_Result>().unwrap();

		assert_eq!(&_Result(skill.id(), OnSkillStop::Ignore), result);
	}
}
