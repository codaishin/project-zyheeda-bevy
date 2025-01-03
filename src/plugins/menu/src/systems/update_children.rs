use crate::traits::insert_ui_content::InsertUiContent;
use bevy::prelude::*;

pub(crate) fn update_children<TComponent: InsertUiContent + Component>(
	mut commands: Commands,
	components: Query<(Entity, &TComponent), Changed<TComponent>>,
) {
	for (entity, component) in &components {
		let Some(mut entity) = commands.get_entity(entity) else {
			continue;
		};
		entity.despawn_descendants();
		entity.with_children(|parent| component.insert_ui_content(parent));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Child(&'static str);

	#[derive(Component)]
	struct _Component(&'static str);

	impl InsertUiContent for _Component {
		fn insert_ui_content(&self, parent: &mut ChildBuilder) {
			parent.spawn(_Child("A"));
			parent.spawn(_Child("B"));
			parent.spawn(_Child("C"));
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, update_children::<_Component>);

		app
	}

	#[test]
	fn render_children() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Component("My Component")).id();

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter_map(|e| Some((e.get::<Parent>()?.get(), e.get::<_Child>()?)));

		assert_eq!(
			vec![
				(parent, &_Child("A")),
				(parent, &_Child("B")),
				(parent, &_Child("C")),
			],
			children.collect::<Vec<_>>()
		)
	}

	#[test]
	fn remove_previous_children() {
		let mut app = setup();
		app.world_mut()
			.spawn(_Component("My Component"))
			.with_children(|parent| {
				parent.spawn(_Child("Previous A"));
				parent.spawn(_Child("Previous B"));
			});

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter_map(|e| e.get::<_Child>());

		assert_eq!(
			vec![&_Child("A"), &_Child("B"), &_Child("C"),],
			children.collect::<Vec<_>>()
		)
	}

	#[test]
	fn remove_previous_children_recursively() {
		let mut app = setup();
		app.world_mut()
			.spawn(_Component("My Component"))
			.with_children(|parent| {
				parent.spawn(_Child("Previous A")).with_children(|parent| {
					parent.spawn(_Child("Previous A Child"));
				});
			});

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter_map(|e| e.get::<_Child>());

		assert_eq!(
			vec![&_Child("A"), &_Child("B"), &_Child("C"),],
			children.collect::<Vec<_>>()
		)
	}

	#[test]
	fn only_work_when_added() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Component("My Component")).id();

		app.update();

		app.world_mut().entity_mut(parent).with_children(|parent| {
			parent.spawn(_Child("Do not remove"));
		});

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter_map(|e| e.get::<_Child>());

		assert_eq!(
			vec![
				&_Child("A"),
				&_Child("B"),
				&_Child("C"),
				&_Child("Do not remove"),
			],
			children.collect::<Vec<_>>()
		)
	}

	#[test]
	fn work_when_changed() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Component("My Component")).id();

		app.update();

		let mut parent = app.world_mut().entity_mut(parent);
		parent.get_mut::<_Component>().unwrap().0 = "My changed Component";
		parent.with_children(|parent| {
			parent.spawn(_Child("Do remove"));
		});

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter_map(|e| e.get::<_Child>());

		assert_eq!(
			vec![&_Child("A"), &_Child("B"), &_Child("C"),],
			children.collect::<Vec<_>>()
		)
	}
}
