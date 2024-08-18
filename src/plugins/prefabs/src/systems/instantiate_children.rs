use crate::components::WithChildren;
use bevy::prelude::{BuildChildren, Commands, Entity, Query};

pub(crate) fn instantiate_children(
	mut commands: Commands,
	delayed: Query<(Entity, &WithChildren)>,
) {
	for (entity, delayed) in &delayed {
		let Some(mut entity) = commands.get_entity(entity) else {
			continue;
		};
		entity.with_children(|parent| (delayed.spawn)(parent));
		entity.remove::<WithChildren>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::{Component, Parent},
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, instantiate_children);

		app
	}

	#[derive(Component, Clone, Debug, PartialEq)]
	struct _Component;

	#[test]
	fn instantiate() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(WithChildren::delayed(|parent| {
				parent.spawn(_Component);
			}))
			.id();

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter(|e| {
				e.get::<Parent>()
					.map(|p| p.get() == entity)
					.unwrap_or(false)
			})
			.filter_map(|e| e.get::<_Component>());

		assert_eq!(vec![&_Component], children.collect::<Vec<_>>());
	}

	#[test]
	fn remove_with_children_component() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(WithChildren::delayed(|parent| {
				parent.spawn(_Component);
			}))
			.id();

		app.update();

		let entity = app.world().entity(entity);

		assert!(!entity.contains::<WithChildren>());
	}
}
