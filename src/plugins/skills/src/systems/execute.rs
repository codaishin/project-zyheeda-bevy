use crate::traits::Execute;
use bevy::{
	ecs::{
		component::Mutable,
		system::{StaticSystemParam, SystemParam},
	},
	prelude::*,
};
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{
		handles_physics::{MouseHover, MouseHoversOver, Raycast},
		handles_skill_behaviors::{SkillCaster, SkillTarget},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> ExecuteSkills for T where T: Component<Mutability = Mutable> + Sized {}

pub(crate) trait ExecuteSkills: Component<Mutability = Mutable> + Sized {
	fn execute_system<TPhysics, TRaycast>(
		mut ray_cast: StaticSystemParam<TRaycast>,
		mut commands: ZyheedaCommands,
		mut agents: Query<(Entity, &mut Self), Changed<Self>>,
		persistent_entities: Query<&PersistentEntity>,
	) where
		Self: Execute<TPhysics>,
		TRaycast: for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseHover>>,
	{
		for (entity, mut skill_executer) in &mut agents {
			let Some(target) = get_target(&mut ray_cast, &persistent_entities) else {
				continue;
			};
			let Ok(entity) = persistent_entities.get(entity) else {
				continue;
			};
			skill_executer.execute(&mut commands, SkillCaster(*entity), target);
		}
	}
}

fn get_target<TRaycast>(
	ray_cast: &mut StaticSystemParam<TRaycast>,
	persistent_entities: &Query<&PersistentEntity>,
) -> Option<SkillTarget>
where
	TRaycast: for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseHover>>,
{
	match ray_cast.raycast(MouseHover::NO_EXCLUDES) {
		Some(MouseHoversOver::Ground { point }) => Some(SkillTarget::from(point)),
		Some(MouseHoversOver::Object { entity, .. }) => persistent_entities
			.get(entity)
			.ok()
			.map(|e| SkillTarget::from(*e)),
		None => None,
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::ops::DerefMut;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _RayCaster {
		mock: Mock_RayCaster,
	}

	impl _RayCaster {
		fn returning(hover: MouseHoversOver) -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_raycast().return_const(hover);
			})
		}
	}

	#[automock]
	impl Raycast<MouseHover> for _RayCaster {
		fn raycast(&mut self, args: MouseHover) -> Option<MouseHoversOver> {
			self.mock.raycast(args)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Executor {
		called_with: Vec<(SkillCaster, SkillTarget)>,
	}

	struct _HandlesPhysics;

	impl Execute<_HandlesPhysics> for _Executor {
		fn execute(&mut self, _: &mut ZyheedaCommands, caster: SkillCaster, target: SkillTarget) {
			self.called_with.push((caster, target));
		}
	}

	fn setup(ray_caster: _RayCaster) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(ray_caster);
		app.add_systems(
			Update,
			_Executor::execute_system::<_HandlesPhysics, ResMut<_RayCaster>>,
		);

		app
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ExecutionArgs {
		caster: SkillCaster,
		target: SkillTarget,
	}

	#[test]
	fn execute_skill_ground_targeted() {
		let hover = MouseHoversOver::Ground {
			point: Vec3::new(1., 2., 3.),
		};
		let target = SkillTarget::from(Vec3::new(1., 2., 3.));
		let mut app = setup(_RayCaster::new().with_mock(|mock| {
			mock.expect_raycast()
				.times(1)
				.with(eq(MouseHover::NO_EXCLUDES))
				.return_const(hover);
		}));
		let persistent_entity = PersistentEntity::default();
		let entity = app
			.world_mut()
			.spawn((
				persistent_entity,
				_Executor {
					called_with: vec![],
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Executor {
				called_with: vec![(SkillCaster(persistent_entity), target)]
			}),
			app.world().entity(entity).get::<_Executor>()
		);
	}

	#[test]
	fn execute_skill_entity_targeted() {
		let mut app = setup(_RayCaster::new());
		let persistent_entity = PersistentEntity::default();
		let entity = app
			.world_mut()
			.spawn((
				persistent_entity,
				_Executor {
					called_with: vec![],
				},
			))
			.id();
		let persistent_target = PersistentEntity::default();
		let target = app.world_mut().spawn(persistent_target).id();
		let hover = MouseHoversOver::Object {
			entity: target,
			point: Vec3::default(),
		};
		let target = SkillTarget::Entity(persistent_target);
		app.insert_resource(_RayCaster::new().with_mock(|mock| {
			mock.expect_raycast()
				.times(1)
				.with(eq(MouseHover::NO_EXCLUDES))
				.return_const(hover);
		}));

		app.update();

		assert_eq!(
			Some(&_Executor {
				called_with: vec![(SkillCaster(persistent_entity), target)]
			}),
			app.world().entity(entity).get::<_Executor>()
		);
	}

	#[test]
	fn execute_skill_only_once() {
		let hover = MouseHoversOver::Ground {
			point: Vec3::new(1., 2., 3.),
		};
		let target = SkillTarget::from(Vec3::new(1., 2., 3.));
		let mut app = setup(_RayCaster::returning(hover));
		let persistent_entity = PersistentEntity::default();
		let entity = app
			.world_mut()
			.spawn((
				persistent_entity,
				_Executor {
					called_with: vec![],
				},
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&_Executor {
				called_with: vec![(SkillCaster(persistent_entity), target)]
			}),
			app.world().entity(entity).get::<_Executor>()
		);
	}

	#[test]
	fn execute_again_after_mutable_deref() {
		let hover = MouseHoversOver::Ground {
			point: Vec3::new(1., 2., 3.),
		};
		let target = SkillTarget::from(Vec3::new(1., 2., 3.));
		let mut app = setup(_RayCaster::returning(hover));
		let persistent_entity = PersistentEntity::default();
		let entity = app
			.world_mut()
			.spawn((
				persistent_entity,
				_Executor {
					called_with: vec![],
				},
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Executor>()
			.unwrap()
			.deref_mut();
		app.update();

		assert_eq!(
			Some(&_Executor {
				called_with: vec![
					(SkillCaster(persistent_entity), target),
					(SkillCaster(persistent_entity), target)
				]
			}),
			app.world().entity(entity).get::<_Executor>()
		);
	}
}
