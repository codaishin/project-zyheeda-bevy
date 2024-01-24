use crate::{
	components::ColliderRoot,
	traits::model::{GetCollider, GetRigidBody, Offset},
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::Added,
		system::{Commands, Query},
	},
	hierarchy::{BuildChildren, ChildBuilder},
	transform::components::Transform,
};
use bevy_rapier3d::{
	geometry::{ActiveEvents, Sensor},
	prelude::ActiveCollisionTypes,
};

pub fn collider<TSource: Component + GetCollider + Offset + GetRigidBody>(
	mut commands: Commands,
	agents: Query<Entity, Added<TSource>>,
) {
	for agent in &agents {
		commands
			.entity(agent)
			.insert(TSource::rigid_body())
			.with_children(child::<TSource>);
	}
}

fn child<TSource: GetCollider + Offset + GetRigidBody>(parent: &mut ChildBuilder) {
	parent.spawn((
		Transform::from_translation(TSource::offset()),
		ColliderRoot(parent.parent_entity()),
		TSource::collider(),
		Sensor,
		ActiveEvents::COLLISION_EVENTS,
		ActiveCollisionTypes::STATIC_STATIC,
	));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::ColliderRoot,
		test_tools::utils::{GetImmediateChildComponents, GetImmediateChildren},
	};
	use bevy::{
		app::{App, Update},
		math::Vec3,
		transform::components::Transform,
	};
	use bevy_rapier3d::{dynamics::RigidBody, geometry::Collider};

	#[derive(Component)]
	struct _Agent;

	impl GetCollider for _Agent {
		fn collider() -> Collider {
			Collider::cuboid(1., 2., 3.)
		}
	}

	impl Offset for _Agent {
		fn offset() -> Vec3 {
			Vec3::new(42., 43., 44.)
		}
	}

	impl GetRigidBody for _Agent {
		fn rigid_body() -> RigidBody {
			RigidBody::KinematicVelocityBased
		}
	}

	fn setup() -> (App, Entity) {
		let mut app = App::new();
		let agent = app.world.spawn(_Agent).id();

		app.add_systems(Update, collider::<_Agent>);

		(app, agent)
	}

	#[test]
	fn insert_rigid_body() {
		let (mut app, agent) = setup();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Agent::rigid_body()), agent.get::<RigidBody>());
	}

	#[test]
	fn add_child_transform_with_offset() {
		let (mut app, agent) = setup();

		app.update();

		let transforms = Transform::get_immediate_children(&agent, &app);
		let transform = transforms.first();

		assert_eq!(Some(&&Transform::from_xyz(42., 43., 44.)), transform);
	}

	#[test]
	fn add_child_with_collider() {
		let (mut app, agent) = setup();

		app.update();

		let colliders = Collider::get_immediate_children(&agent, &app);
		let collider = colliders.first();

		assert_eq!(
			Collider::cuboid(1., 2., 3.).as_cuboid().map(|c| c.raw),
			collider.and_then(|c| c.as_cuboid()).map(|c| c.raw)
		);
	}

	#[test]
	fn add_child_with_sensor() {
		let (mut app, agent) = setup();

		app.update();

		let sensors = Sensor::get_immediate_children(&agent, &app);
		let sensor = sensors.first();

		assert!(sensor.is_some());
	}

	#[test]
	fn add_child_with_collision_events_set() {
		let (mut app, agent) = setup();

		app.update();

		let events = ActiveEvents::get_immediate_children(&agent, &app);
		let event = events.first();

		assert_eq!(Some(&&ActiveEvents::COLLISION_EVENTS), event);
	}

	#[test]
	fn add_child_with_collision_type_set() {
		let (mut app, agent) = setup();

		app.update();

		let collision_types = ActiveCollisionTypes::get_immediate_children(&agent, &app);
		let collision_type = collision_types.first();

		assert_eq!(Some(&&ActiveCollisionTypes::STATIC_STATIC), collision_type);
	}

	#[test]
	fn add_child_with_collider_root() {
		let (mut app, agent) = setup();

		app.update();

		let roots = ColliderRoot::get_immediate_children(&agent, &app);
		let root = roots.first();

		assert_eq!(Some(&&ColliderRoot(agent)), root);
	}

	#[test]
	fn act_only_once() {
		let (mut app, agent) = setup();

		app.update();

		app.world.entity_mut(agent).remove::<RigidBody>();

		app.update();

		let children = Entity::get_immediate_children(&agent, &app);
		let agent = app.world.entity(agent);

		assert_eq!((1, false), (children.len(), agent.contains::<RigidBody>()));
	}
}
