use crate::{behaviors::SkillCaster, components::SkillTarget, traits::Execute};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	components::persistent_entity::PersistentEntity,
	tools::collider_info::ColliderInfo,
	traits::{
		accessors::get::GetterRefOptional,
		handles_player::{HandlesPlayerCameras, HandlesPlayerMouse},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> ExecuteSkills for T where T: Component<Mutability = Mutable> + Sized {}

pub(crate) trait ExecuteSkills: Component<Mutability = Mutable> + Sized {
	fn execute_system<TEffects, TSkillBehaviors, TPlayers>(
		cam_ray: Res<TPlayers::TCamRay>,
		mouse_hover: Res<TPlayers::TMouseHover>,
		mut commands: ZyheedaCommands,
		mut agents: Query<(&PersistentEntity, &mut Self), Changed<Self>>,
		transforms: Query<&GlobalTransform>,
	) where
		for<'w, 's> Self: Execute<TEffects, TSkillBehaviors>,
		TPlayers: HandlesPlayerCameras + HandlesPlayerMouse,
	{
		for (entity, mut skill_executer) in &mut agents {
			let Some(target) = get_target(&cam_ray, &mouse_hover, &transforms) else {
				continue;
			};
			skill_executer.execute(&mut commands, &SkillCaster(*entity), &target);
		}
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
	use common::{
		components::outdated::Outdated,
		tools::collider_info::ColliderInfo,
		traits::intersect_at::IntersectAt,
	};
	use std::ops::DerefMut;
	use testing::SingleThreadedApp;

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

	#[derive(Component, Debug, PartialEq)]
	struct _Executor {
		called_with: Vec<(SkillCaster, SkillTarget)>,
	}

	struct _HandlesEffects;

	struct _HandlesSkillBehaviors;

	impl Execute<_HandlesEffects, _HandlesSkillBehaviors> for _Executor {
		fn execute(&mut self, _: &mut ZyheedaCommands, caster: &SkillCaster, target: &SkillTarget) {
			self.called_with.push((*caster, *target));
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
		let execute_system =
			_Executor::execute_system::<_HandlesEffects, _HandlesSkillBehaviors, _Players>;

		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_CamRay>();
		app.init_resource::<_MouseHover>();
		app.add_systems(Update, execute_system);

		app
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ExecutionArgs {
		caster: SkillCaster,
		target: SkillTarget,
	}

	#[test]
	fn execute_skill() {
		let mut app = setup();
		let target = set_target(&mut app);
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
	fn execute_skill_only_once() {
		let mut app = setup();
		let target = set_target(&mut app);
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
		let mut app = setup();
		let target = set_target(&mut app);
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
