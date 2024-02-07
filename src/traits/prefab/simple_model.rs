use super::SpawnPrefab;
use crate::{bundles::ColliderBundle, components::ColliderRoot, resources::Prefab};
use bevy::{
	ecs::{bundle::Bundle, system::EntityCommands},
	hierarchy::BuildChildren,
	pbr::PbrBundle,
};
use bevy_rapier3d::geometry::Sensor;

pub type SimpleModelPrefab<TFor, TParent> = Prefab<TFor, TParent, (PbrBundle, ColliderBundle)>;

impl<TFor, TParent: Bundle + Clone> SpawnPrefab<TFor> for SimpleModelPrefab<TFor, TParent> {
	fn spawn_prefab(&self, parent: &mut EntityCommands) {
		parent.insert(self.parent.clone()).with_children(|parent| {
			parent.spawn(self.children.0.clone());
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
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		ecs::{
			component::Component,
			entity::Entity,
			query::With,
			system::{Commands, Query, Res, Resource},
		},
		hierarchy::Parent,
		pbr::StandardMaterial,
		render::{
			mesh::Mesh,
			view::{InheritedVisibility, ViewVisibility, Visibility},
		},
		transform::{
			components::{GlobalTransform, Transform},
			TransformBundle,
		},
		utils::Uuid,
	};
	use bevy_rapier3d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Clone, Debug, PartialEq, Default)]
	struct _Parent(&'static str);

	fn run_with_root_reference<T: SpawnPrefab<_Agent> + Resource>(
		prefab: &Res<T>,
		commands: &mut EntityCommands,
	) {
		prefab.spawn_prefab(commands)
	}

	fn run_spawn(
		mut commands: Commands,
		prefab: Res<SimpleModelPrefab<_Agent, _Parent>>,
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
		app.insert_resource(SimpleModelPrefab::<_Agent, _Parent>::new(
			_Parent("parent"),
			(PbrBundle::default(), ColliderBundle::default()),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Parent("parent")), agent.get::<_Parent>());
	}

	#[test]
	fn spawn_pbr_bundle_on_first_child() {
		let mut app = App::new();
		let bundle = PbrBundle {
			mesh: Handle::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}),
			material: Handle::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}),
			transform: Transform::from_xyz(1., 2., 3.),
			global_transform: GlobalTransform::from_xyz(4., 5., 6.),
			visibility: Visibility::Hidden,
			inherited_visibility: InheritedVisibility::HIDDEN,
			view_visibility: ViewVisibility::HIDDEN,
		};
		app.add_systems(Update, run_spawn);
		app.insert_resource(SimpleModelPrefab::<_Agent, _Parent>::new(
			_Parent::default(),
			(bundle.clone(), ColliderBundle::default()),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let (child, ..) = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.find(|(_, parent)| parent.get() == agent)
			.unwrap();
		assert_eq!(
			(
				Some(&bundle.mesh),
				Some(&bundle.material),
				Some(&bundle.transform),
				Some(&bundle.global_transform),
				Some(&bundle.visibility),
				Some(&bundle.inherited_visibility),
				Some(&bundle.view_visibility),
			),
			(
				child.get::<Handle<Mesh>>(),
				child.get::<Handle<StandardMaterial>>(),
				child.get::<Transform>(),
				child.get::<GlobalTransform>(),
				child.get::<Visibility>(),
				child.get::<InheritedVisibility>(),
				child.get::<ViewVisibility>(),
			)
		);
	}

	#[test]
	fn spawn_collider_bundle_on_second_child() {
		let mut app = App::new();
		let bundle = ColliderBundle {
			collider: Collider::ball(4.),
			transform: TransformBundle::from_transform(Transform::from_xyz(1., 2., 3.)),
			active_events: ActiveEvents::CONTACT_FORCE_EVENTS,
			active_collision_types: ActiveCollisionTypes::DYNAMIC_KINEMATIC,
		};
		app.add_systems(Update, run_spawn);
		app.insert_resource(SimpleModelPrefab::<_Agent, _Parent>::new(
			_Parent::default(),
			(PbrBundle::default(), bundle.clone()),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let (child, ..) = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.skip(1)
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
	fn spawn_sensor_on_second_child() {
		let mut app = App::new();
		app.add_systems(Update, run_spawn);
		app.insert_resource(SimpleModelPrefab::<_Agent, _Parent>::new(
			_Parent::default(),
			(PbrBundle::default(), ColliderBundle::default()),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let (child, ..) = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.skip(1)
			.find(|(_, parent)| parent.get() == agent)
			.unwrap();
		assert!(child.contains::<Sensor>());
	}

	#[test]
	fn spawn_collider_root_on_second_child() {
		let mut app = App::new();
		app.add_systems(Update, run_spawn);
		app.insert_resource(SimpleModelPrefab::<_Agent, _Parent>::new(
			_Parent::default(),
			(PbrBundle::default(), ColliderBundle::default()),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let (child, ..) = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.skip(1)
			.find(|(_, parent)| parent.get() == agent)
			.unwrap();
		assert_eq!(Some(&ColliderRoot(agent)), child.get::<ColliderRoot>());
	}
}
