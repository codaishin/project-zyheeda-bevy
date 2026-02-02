use crate::components::{
	blockable::Blockable,
	collider::ColliderShape,
	effects::Effects,
	skill::{ContactInteractionTarget, ProjectionInteractionTarget},
	skill_transform::SkillTransformOf,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::{lifetime::Lifetime, model::Model},
	traits::{accessors::get::GetMut, handles_physics::PhysicalObject},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};
use std::time::Duration;

impl<T> SkillPrefab for T where
	T: Component + GetLifetime + GetContactPrefab + GetProjectionPrefab + ApplyMotionPrefab
{
}

pub(crate) trait SkillPrefab:
	Component + GetLifetime + GetContactPrefab + GetProjectionPrefab + ApplyMotionPrefab + Sized
{
	fn prefab(on_insert: On<Insert, Self>, mut commands: ZyheedaCommands, skills: Query<&Self>) {
		let root = on_insert.entity;
		let Some(mut entity) = commands.get_mut(&root) else {
			return;
		};
		let Ok(skill) = skills.get(entity.id()) else {
			return;
		};
		let rigid_body = skill.apply_motion_prefab(&mut entity);
		let (obj, cont_model, cont_collider, cont_effects) = skill.get_contact_prefab();
		let (proj_model, proj_collider, proj_effects) = skill.get_projection_prefab();

		entity.try_insert((
			rigid_body,
			Blockable(obj),
			ContactInteractionTarget,
			cont_effects,
			children![
				(
					SkillTransformOf(root),
					cont_model.transform,
					cont_model.model,
				),
				(
					SkillTransformOf(root),
					cont_collider.transform,
					cont_collider.shape,
					Sensor,
				),
				(
					ProjectionInteractionTarget,
					proj_effects,
					children![
						(
							SkillTransformOf(root),
							proj_model.transform,
							proj_model.model
						),
						(
							SkillTransformOf(root),
							proj_collider.transform,
							proj_collider.shape,
							Sensor,
						),
					]
				)
			],
		));

		let Some(lifetime) = skill.get_lifetime() else {
			return;
		};

		entity.try_insert(Lifetime::from(lifetime));
	}
}

pub(crate) trait GetLifetime {
	fn get_lifetime(&self) -> Option<Duration>;
}

pub(crate) trait GetContactPrefab {
	fn get_contact_prefab(&self) -> (PhysicalObject, SubModel, ContactCollider, Effects);
}

pub(crate) trait GetProjectionPrefab {
	fn get_projection_prefab(&self) -> (SubModel, ProjectionCollider, Effects);
}

pub(crate) trait ApplyMotionPrefab {
	fn apply_motion_prefab(&self, entity: &mut ZyheedaEntityCommands) -> RigidBody;
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SubModel {
	pub(crate) model: Model,
	pub(crate) transform: Transform,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ProjectionCollider {
	pub(crate) shape: ColliderShape,
	pub(crate) transform: Transform,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ContactCollider {
	pub(crate) shape: ColliderShape,
	pub(crate) transform: Transform,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{skill::ContactInteractionTarget, skill_transform::SkillTransformOf};
	use common::{
		components::asset_model::AssetModel,
		effects::force::Force,
		tools::Units,
		traits::{
			handles_physics::{PhysicalObject, physical_bodies::Shape},
			handles_skill_physics::Effect,
		},
	};
	use std::collections::HashSet;
	use testing::{SingleThreadedApp, assert_children_count};

	#[derive(Component, Debug, PartialEq)]
	struct _Motion;

	#[derive(Component)]
	struct _Skill {
		rigid_body: RigidBody,
		lifetime: Option<Duration>,
		contact: (PhysicalObject, SubModel, ContactCollider, Effects),
		projection: (SubModel, ProjectionCollider, Effects),
	}

	impl _Skill {
		fn default_object() -> PhysicalObject {
			PhysicalObject::Fragile {
				destroyed_by: HashSet::from([]),
			}
		}

		fn default_model() -> SubModel {
			SubModel {
				model: Model::Asset(AssetModel::none()),
				transform: Transform::default(),
			}
		}

		fn default_contact_collider() -> ContactCollider {
			ContactCollider {
				shape: ColliderShape::from(Shape::Sphere {
					radius: Units::ZERO,
				}),
				transform: Transform::default(),
			}
		}

		fn default_projection_collider() -> ProjectionCollider {
			ProjectionCollider {
				shape: ColliderShape::from(Shape::Sphere {
					radius: Units::from(1.),
				}),
				transform: Transform::default(),
			}
		}

		fn default_effects() -> Effects {
			Effects(vec![])
		}
	}

	impl Default for _Skill {
		fn default() -> Self {
			Self {
				rigid_body: RigidBody::Dynamic,
				lifetime: None,
				contact: (
					Self::default_object(),
					Self::default_model(),
					Self::default_contact_collider(),
					Effects(vec![]),
				),
				projection: (
					Self::default_model(),
					Self::default_projection_collider(),
					Effects(vec![]),
				),
			}
		}
	}

	impl GetLifetime for _Skill {
		fn get_lifetime(&self) -> Option<Duration> {
			self.lifetime
		}
	}

	impl ApplyMotionPrefab for _Skill {
		fn apply_motion_prefab(&self, entity: &mut ZyheedaEntityCommands) -> RigidBody {
			entity.try_insert(_Motion);

			self.rigid_body
		}
	}

	impl GetContactPrefab for _Skill {
		fn get_contact_prefab(&self) -> (PhysicalObject, SubModel, ContactCollider, Effects) {
			self.contact.clone()
		}
	}
	impl GetProjectionPrefab for _Skill {
		fn get_projection_prefab(&self) -> (SubModel, ProjectionCollider, Effects) {
			self.projection.clone()
		}
	}

	macro_rules! assert_projection_count {
		($count:literal, $app:expr, $skill:expr) => {
			assert_children_count!($count, $app, $skill, |e| {
				if e.contains::<ProjectionInteractionTarget>() {
					Some(e.id())
				} else {
					None
				}
			})
		};
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(_Skill::prefab);

		app
	}

	mod root {
		use super::*;

		#[test]
		fn insert_blockable() {
			let mut app = setup();

			let skill = app.world_mut().spawn(_Skill {
				contact: (
					PhysicalObject::Beam {
						range: Units::from(11.),
						blocked_by: HashSet::from([]),
					},
					_Skill::default_model(),
					_Skill::default_contact_collider(),
					_Skill::default_effects(),
				),
				..default()
			});

			assert_eq!(
				Some(&Blockable(PhysicalObject::Beam {
					range: Units::from(11.),
					blocked_by: HashSet::from([]),
				})),
				skill.get::<Blockable>(),
			);
		}

		#[test]
		fn insert_lifetime() {
			let mut app = setup();

			let skill = app.world_mut().spawn(_Skill {
				lifetime: Some(Duration::from_hours(1)),
				..default()
			});

			assert_eq!(
				Some(&Lifetime::from(Duration::from_hours(1))),
				skill.get::<Lifetime>(),
			);
		}

		#[test]
		fn insert_contact() {
			let mut app = setup();

			let skill = app.world_mut().spawn(_Skill::default());

			assert_eq!(
				Some(&ContactInteractionTarget),
				skill.get::<ContactInteractionTarget>()
			);
		}

		#[test]
		fn spawn_projection_child() {
			let mut app = setup();

			let skill = app.world_mut().spawn(_Skill::default()).id();

			assert_projection_count!(1, app, skill);
		}
	}

	mod contact {
		use super::*;

		#[test]
		fn spawn_model_child() {
			let mut app = setup();

			let skill = app
				.world_mut()
				.spawn(_Skill {
					contact: (
						_Skill::default_object(),
						SubModel {
							model: Model::Asset(AssetModel::from("asset/path")),
							transform: Transform::from_xyz(1., 2., 3.),
						},
						_Skill::default_contact_collider(),
						_Skill::default_effects(),
					),
					..default()
				})
				.id();

			let [model, ..] = assert_children_count!(3, app, skill);
			assert_eq!(
				(
					Some(&Model::Asset(AssetModel::from("asset/path"))),
					Some(&Transform::from_xyz(1., 2., 3.))
				),
				(model.get::<Model>(), model.get::<Transform>(),),
			);
		}

		#[test]
		fn spawn_collider_child() {
			let mut app = setup();

			let skill = app
				.world_mut()
				.spawn(_Skill {
					contact: (
						_Skill::default_object(),
						_Skill::default_model(),
						ContactCollider {
							shape: ColliderShape::from(Shape::Sphere {
								radius: Units::from(42.),
							}),
							transform: Transform::from_xyz(1., 2., 3.),
						},
						_Skill::default_effects(),
					),
					..default()
				})
				.id();

			let [_, collider, _] = assert_children_count!(3, app, skill);
			assert_eq!(
				(
					Some(&ColliderShape::from(Shape::Sphere {
						radius: Units::from(42.),
					})),
					Some(&Transform::from_xyz(1., 2., 3.)),
				),
				(collider.get::<ColliderShape>(), collider.get::<Transform>()),
			);
		}

		#[test]
		fn spawn_collider_rapier_sensor() {
			let mut app = setup();

			let skill = app.world_mut().spawn(_Skill::default()).id();

			let [_, collider, _] = assert_children_count!(3, app, skill);
			assert_eq!(Some(&Sensor), collider.get::<Sensor>());
		}

		#[test]
		fn add_skill_transform_children() {
			let mut app = setup();

			let skill = app.world_mut().spawn(_Skill::default()).id();

			let [model, collider, ..] = assert_children_count!(3, app, skill);
			assert_eq!(
				(
					Some(&SkillTransformOf(skill)),
					Some(&SkillTransformOf(skill))
				),
				(
					model.get::<SkillTransformOf>(),
					collider.get::<SkillTransformOf>()
				),
			);
		}

		#[test]
		fn add_effects() {
			let mut app = setup();

			let skill = app.world_mut().spawn(_Skill {
				contact: (
					_Skill::default_object(),
					_Skill::default_model(),
					_Skill::default_contact_collider(),
					Effects(vec![Effect::Force(Force)]),
				),
				..default()
			});

			assert_eq!(
				Some(&Effects(vec![Effect::Force(Force)])),
				skill.get::<Effects>(),
			);
		}
	}

	mod projection {
		use super::*;

		#[test]
		fn spawn_model_child() {
			let mut app = setup();

			let skill = app
				.world_mut()
				.spawn(_Skill {
					projection: (
						SubModel {
							model: Model::Asset(AssetModel::from("asset/path")),
							transform: Transform::from_xyz(1., 2., 3.),
						},
						_Skill::default_projection_collider(),
						_Skill::default_effects(),
					),
					..default()
				})
				.id();

			let [projection] = assert_projection_count!(1, app, skill);
			let [model, ..] = assert_children_count!(2, app, projection);
			assert_eq!(
				(
					Some(&Model::Asset(AssetModel::from("asset/path"))),
					Some(&Transform::from_xyz(1., 2., 3.))
				),
				(model.get::<Model>(), model.get::<Transform>(),),
			);
		}

		#[test]
		fn spawn_collider_child() {
			let mut app = setup();

			let skill = app
				.world_mut()
				.spawn(_Skill {
					projection: (
						_Skill::default_model(),
						ProjectionCollider {
							shape: ColliderShape::from(Shape::Sphere {
								radius: Units::from(42.),
							}),
							transform: Transform::from_xyz(1., 2., 3.),
						},
						_Skill::default_effects(),
					),
					..default()
				})
				.id();

			let [projection] = assert_projection_count!(1, app, skill);
			let [.., collider] = assert_children_count!(2, app, projection);
			assert_eq!(
				(
					Some(&ColliderShape::from(Shape::Sphere {
						radius: Units::from(42.),
					})),
					Some(&Transform::from_xyz(1., 2., 3.)),
				),
				(collider.get::<ColliderShape>(), collider.get::<Transform>()),
			);
		}

		#[test]
		fn spawn_collider_rapier_sensor() {
			let mut app = setup();

			let skill = app.world_mut().spawn(_Skill::default()).id();

			let [projection] = assert_projection_count!(1, app, skill);
			let [.., collider] = assert_children_count!(2, app, projection);
			assert_eq!(Some(&Sensor), collider.get::<Sensor>());
		}

		#[test]
		fn add_skill_transform_children() {
			let mut app = setup();

			let skill = app.world_mut().spawn(_Skill::default()).id();

			let [projection] = assert_projection_count!(1, app, skill);
			let [model, collider] = assert_children_count!(2, app, projection);
			assert_eq!(
				(
					Some(&SkillTransformOf(skill)),
					Some(&SkillTransformOf(skill))
				),
				(
					model.get::<SkillTransformOf>(),
					collider.get::<SkillTransformOf>()
				),
			);
		}

		#[test]
		fn add_effects() {
			let mut app = setup();

			let skill = app
				.world_mut()
				.spawn(_Skill {
					projection: (
						_Skill::default_model(),
						_Skill::default_projection_collider(),
						Effects(vec![Effect::Force(Force)]),
					),
					..default()
				})
				.id();

			let [.., projection] = assert_children_count!(3, app, skill);
			assert_eq!(
				Some(&Effects(vec![Effect::Force(Force)])),
				projection.get::<Effects>(),
			);
		}
	}

	mod motion {
		use super::*;

		#[test]
		fn apply_motion() {
			let mut app = setup();

			let entity = app.world_mut().spawn(_Skill::default());

			assert_eq!(Some(&_Motion), entity.get::<_Motion>());
		}

		#[test]
		fn insert_rigid_body() {
			let mut app = setup();

			let entity = app.world_mut().spawn(_Skill {
				rigid_body: RigidBody::KinematicPositionBased,
				..default()
			});

			assert_eq!(
				Some(&RigidBody::KinematicPositionBased),
				entity.get::<RigidBody>()
			);
		}
	}
}
