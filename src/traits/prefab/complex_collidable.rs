use super::SpawnPrefab;
use crate::{bundles::ColliderBundle, components::ColliderRoot, resources::Prefab};
use bevy::{
	ecs::{bundle::Bundle, system::EntityCommands},
	hierarchy::BuildChildren,
};
use bevy_rapier3d::geometry::Sensor;

pub type ComplexCollidablePrefab<TFor, TParent, TChildren, const N: usize> =
	Prefab<TFor, TParent, ([TChildren; N], ColliderBundle)>;

impl<TFor, TParent: Bundle + Clone, TChildren: Bundle + Clone, const N: usize> SpawnPrefab<TFor>
	for ComplexCollidablePrefab<TFor, TParent, TChildren, N>
{
	fn spawn_prefab(&self, parent: &mut EntityCommands) {
		parent.insert(self.parent.clone()).with_children(|parent| {
			for child in self.children.0.iter() {
				parent.spawn(child.clone());
			}
			parent.spawn((
				self.children.1.clone(),
				Sensor,
				ColliderRoot(parent.parent_entity()),
			));
		});
	}
}

#[cfg(test)]
mod tests {
	use crate::components::ColliderRoot;

	use super::*;
	use bevy::{
		self,
		app::{App, Update},
		ecs::{
			component::Component,
			entity::Entity,
			query::With,
			system::{Commands, Query, Res, Resource},
		},
		hierarchy::Parent,
		transform::{
			components::{GlobalTransform, Transform},
			TransformBundle,
		},
	};
	use bevy_rapier3d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Clone, Debug, PartialEq, Default)]
	struct _Parent(&'static str);

	#[derive(Component, Clone, Debug, PartialEq, Default)]
	struct _Child(&'static str);

	fn run_with_root_reference<T: SpawnPrefab<_Agent> + Resource>(
		prefab: &Res<T>,
		commands: &mut EntityCommands,
	) {
		prefab.spawn_prefab(commands)
	}

	type TestPrefab = ComplexCollidablePrefab<_Agent, _Parent, _Child, 2>;

	fn run_spawn(
		mut commands: Commands,
		prefab: Res<TestPrefab>,
		agents: Query<Entity, With<_Agent>>,
	) {
		for agent in &agents {
			run_with_root_reference(&prefab, &mut commands.entity(agent));
		}
	}

	#[test]
	fn spawn_parent_bundle() {
		let mut app = App::new();
		app.add_systems(Update, run_spawn);
		app.insert_resource(TestPrefab::new(
			_Parent("parent"),
			(
				[_Child::default(), _Child::default()],
				ColliderBundle::default(),
			),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Parent("parent")), agent.get::<_Parent>());
	}

	#[test]
	fn spawn_children() {
		let mut app = App::new();
		app.add_systems(Update, run_spawn);
		app.insert_resource(TestPrefab::new(
			_Parent::default(),
			(
				[_Child("child a"), _Child("child b")],
				ColliderBundle::default(),
			),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let children = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.filter(|(_, parent)| parent.get() == agent)
			.filter_map(|(entity, _)| entity.get::<_Child>())
			.collect::<Vec<_>>();
		assert_eq!(vec![&_Child("child a"), &_Child("child b")], children);
	}

	#[test]
	fn spawn_collider_bundle_on_third_child() {
		let mut app = App::new();
		let bundle = ColliderBundle {
			collider: Collider::ball(4.),
			transform: TransformBundle::from_transform(Transform::from_xyz(1., 2., 3.)),
			active_events: ActiveEvents::CONTACT_FORCE_EVENTS,
			active_collision_types: ActiveCollisionTypes::DYNAMIC_KINEMATIC,
		};
		app.add_systems(Update, run_spawn);
		app.insert_resource(TestPrefab::new(
			_Parent::default(),
			([_Child::default(), _Child::default()], bundle.clone()),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let (child, ..) = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.skip(2)
			.find(|(_, parent)| parent.get() == agent)
			.unwrap();
		assert_eq!(
			(
				bundle.collider.as_ball().map(|b| b.raw),
				Some(&bundle.transform.local),
				Some(&bundle.transform.global),
				Some(&bundle.active_events),
				Some(&bundle.active_collision_types),
			),
			(
				child
					.get::<Collider>()
					.and_then(|c| c.as_ball().map(|b| b.raw)),
				child.get::<Transform>(),
				child.get::<GlobalTransform>(),
				child.get::<ActiveEvents>(),
				child.get::<ActiveCollisionTypes>(),
			)
		);
	}

	#[test]
	fn spawn_sensor_on_second_third() {
		let mut app = App::new();
		app.add_systems(Update, run_spawn);
		app.insert_resource(TestPrefab::new(
			_Parent::default(),
			(
				[_Child::default(), _Child::default()],
				ColliderBundle::default(),
			),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let (child, ..) = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.skip(2)
			.find(|(_, parent)| parent.get() == agent)
			.unwrap();
		assert!(child.contains::<Sensor>());
	}

	#[test]
	fn spawn_collider_root_on_third_child() {
		let mut app = App::new();
		app.add_systems(Update, run_spawn);
		app.insert_resource(TestPrefab::new(
			_Parent::default(),
			(
				[_Child::default(), _Child::default()],
				ColliderBundle::default(),
			),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let (child, ..) = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.skip(2)
			.find(|(_, parent)| parent.get() == agent)
			.unwrap();
		assert_eq!(Some(&ColliderRoot(agent)), child.get::<ColliderRoot>());
	}
}
