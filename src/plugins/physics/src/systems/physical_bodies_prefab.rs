use crate::components::{
	blocker_types::BlockerTypes,
	collider::{ColliderOf, ColliderShape, Colliders},
	no_hover::NoMouseHover,
	physical_body::PhysicalBody,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, handles_physics::physical_bodies::PhysicsType},
	zyheeda_commands::ZyheedaCommands,
};
use std::sync::LazyLock;

impl PhysicalBody {
	pub(crate) fn prefab(
		mut commands: ZyheedaCommands,
		bodies: Query<(Entity, &Self, &Colliders), Changed<Self>>,
	) {
		for (entity, body, colliders) in &bodies {
			despawn_current_collider_shapes(&mut commands, colliders);
			insert_rigid_body(&mut commands, entity, body);
			apply_definition(&mut commands, entity, body);
		}
	}
}

const AGENT_GRAVITY_SCALE: GravityScale = GravityScale(0.);
static AGENT_LOCKED_AXES: LazyLock<LockedAxes> =
	LazyLock::new(|| LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y);

fn despawn_current_collider_shapes(commands: &mut ZyheedaCommands, colliders: &Colliders) {
	for entity in colliders.iter() {
		commands.try_apply_on(&entity, |e| {
			e.try_despawn();
		});
	}
}

fn insert_rigid_body(
	commands: &mut ZyheedaCommands,
	entity: Entity,
	PhysicalBody(body): &PhysicalBody,
) {
	commands.try_apply_on(&entity, |mut e| match body.physics_type {
		PhysicsType::Agent => {
			e.try_insert((*AGENT_LOCKED_AXES, AGENT_GRAVITY_SCALE, RigidBody::Dynamic));
		}
		PhysicsType::Terrain => {
			e.try_insert(RigidBody::Fixed);
		}
	});
}

fn apply_definition(
	commands: &mut ZyheedaCommands,
	entity: Entity,
	PhysicalBody(definition): &PhysicalBody,
) {
	commands.try_apply_on(&entity, |mut e| {
		e.try_insert(BlockerTypes(definition.blocker_types.clone()));
	});

	let mut entity = commands.spawn((
		ColliderOf(entity),
		ChildOf(entity),
		Transform::from_translation(definition.center_offset).with_rotation(definition.rotation),
		ColliderShape::from(definition.shape),
	));

	if definition.physics_type != PhysicsType::Terrain {
		return;
	}

	entity.insert(NoMouseHover);
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::Units,
		traits::handles_physics::physical_bodies::{Blocker, Body, Shape},
	};
	use std::f32::consts::PI;
	use test_case::test_case;
	use testing::{SingleThreadedApp, assert_count, get_children};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, PhysicalBody::prefab);

		app
	}

	#[test]
	fn spawn_as_child() {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(Body::from_shape(shape)))
			.id();

		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			Some(&ColliderShape::from(shape)),
			child.get::<ColliderShape>(),
		);
	}

	#[test]
	fn spawn_with_offset() {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let offset = Vec3::ONE;
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_center_offset(offset),
			))
			.id();

		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			Some(&Transform::from_translation(offset)),
			child.get::<Transform>(),
		);
	}

	#[test]
	fn spawn_with_rotation() {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let rotation = Quat::from_rotation_x(PI);
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_rotation(rotation),
			))
			.id();

		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			Some(&Transform::from_rotation(rotation)),
			child.get::<Transform>(),
		);
	}

	#[test_case(PhysicsType::Terrain, RigidBody::Fixed, None, None; "terrain")]
	#[test_case(PhysicsType::Agent, RigidBody::Dynamic, Some(*AGENT_LOCKED_AXES), Some(AGENT_GRAVITY_SCALE); "agent")]
	fn insert_additional_root_components(
		physics_type: PhysicsType,
		rigid_body: RigidBody,
		locked_axes: Option<LockedAxes>,
		gravity_scale: Option<GravityScale>,
	) {
		let mut app = setup();
		let blocks = [Blocker::Force, Blocker::Physical];
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape)
					.with_physics_type(physics_type)
					.with_blocker_types(blocks),
			))
			.id();

		app.update();

		assert_eq!(
			(
				Some(&rigid_body),
				locked_axes,
				gravity_scale,
				Some(&BlockerTypes::from(blocks))
			),
			(
				app.world().entity(entity).get::<RigidBody>(),
				app.world().entity(entity).get::<LockedAxes>().copied(),
				app.world().entity(entity).get::<GravityScale>().copied(),
				app.world().entity(entity).get::<BlockerTypes>(),
			)
		);
	}

	#[test_case(PhysicsType::Terrain, Some(NoMouseHover); "terrain")]
	#[test_case(PhysicsType::Agent, None; "agent")]
	fn insert_additional_collider_components(
		collider_type: PhysicsType,
		no_mouse_hover: Option<NoMouseHover>,
	) {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_physics_type(collider_type),
			))
			.id();

		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(no_mouse_hover, child.get::<NoMouseHover>().copied());
	}

	#[test]
	fn do_nothing_when_not_changed() {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(Body::from_shape(Shape::Sphere {
				radius: Units::from(11.),
			})))
			.id();

		app.update();
		let [child] = assert_count!(1, get_children!(app, entity).map(|e| e.id()));
		app.world_mut()
			.entity_mut(child)
			.insert(ColliderShape::from(shape));
		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			Some(&ColliderShape::from(shape)),
			child.get::<ColliderShape>()
		);
	}

	#[test]
	fn reinsert_collider() {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(Body::from_shape(Shape::Sphere {
				radius: Units::from(11.),
			})))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(PhysicalBody(Body::from(shape)));
		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			Some(&ColliderShape::from(shape)),
			child.get::<ColliderShape>()
		);
	}
}
