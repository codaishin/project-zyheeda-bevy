use super::SpawnPrefab;
use crate::{bundles::ColliderBundle, resources::Prefab};
use bevy::{
	ecs::{bundle::Bundle, entity::Entity, system::EntityCommands},
	hierarchy::BuildChildren,
	pbr::PbrBundle,
};

pub type SimpleModelPrefab<TFor, TParent, TColliderExtra> =
	Prefab<TFor, TParent, (PbrBundle, ColliderBundle<TColliderExtra>)>;

impl<
		TFor,
		TParent: Bundle + Clone,
		TChildren: Bundle + Clone,
		TRootReference: From<Entity> + Bundle,
	> SpawnPrefab<TRootReference> for Prefab<TFor, TParent, TChildren>
{
	fn spawn_prefab(&self, parent: &mut EntityCommands) {
		parent.insert(self.parent.clone()).with_children(|parent| {
			parent.spawn((
				self.children.clone(),
				TRootReference::from(parent.parent_entity()),
			));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{
			component::Component,
			entity::Entity,
			query::With,
			system::{Commands, Query, Res, Resource},
		},
		hierarchy::Parent,
	};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Clone, Debug, PartialEq, Default)]
	struct _Parent(&'static str);

	#[derive(Component, Clone, Debug, PartialEq, Default)]
	struct _Children(&'static str);

	#[derive(Component, Debug, PartialEq)]
	struct _RootReference(Entity);

	impl From<Entity> for _RootReference {
		fn from(value: Entity) -> Self {
			Self(value)
		}
	}

	fn run_with_root_reference<T: SpawnPrefab<_RootReference> + Resource>(
		prefab: &Res<T>,
		commands: &mut EntityCommands,
	) {
		prefab.spawn_prefab(commands)
	}

	fn run_spawn(
		mut commands: Commands,
		prefab: Res<Prefab<_Agent, _Parent, _Children>>,
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
		app.insert_resource(Prefab::<_Agent, _Parent, _Children>::new(
			_Parent("parent"),
			_Children::default(),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Parent("parent")), agent.get::<_Parent>());
	}

	#[test]
	fn spawn_children_bundle() {
		let mut app = App::new();
		app.add_systems(Update, run_spawn);
		app.insert_resource(Prefab::<_Agent, _Parent, _Children>::new(
			_Parent::default(),
			_Children("children"),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let children = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.filter(|(_, parent)| parent.get() == agent)
			.find_map(|(entity, _)| entity.get::<_Children>());
		assert_eq!(Some(&_Children("children")), children);
	}

	#[test]
	fn spawn_root_reference() {
		let mut app = App::new();
		app.add_systems(Update, run_spawn);
		app.insert_resource(Prefab::<_Agent, _Parent, _Children>::new(
			_Parent::default(),
			_Children::default(),
		));

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let root_reference = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.filter(|(_, parent)| parent.get() == agent)
			.find_map(|(entity, _)| entity.get::<_RootReference>());
		assert_eq!(Some(&_RootReference(agent)), root_reference);
	}
}
