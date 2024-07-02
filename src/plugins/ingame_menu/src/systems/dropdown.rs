use crate::{components::dropdown::Dropdown, tools::Layout};
use bevy::{
	hierarchy::{BuildChildren, ChildBuilder, Parent},
	prelude::{Added, Commands, Query},
	ui::{node_bundles::NodeBundle, Display, RepeatedGridTrack, Style},
	utils::default,
};

pub(crate) fn dropdown(
	mut commands: Commands,
	dropdowns: Query<(&Parent, &Dropdown), Added<Dropdown>>,
) {
	for (container, dropdown) in &dropdowns {
		let Some(mut container) = commands.get_entity(container.get()) else {
			continue;
		};

		container.with_children(|container_node| spawn_dropdown(container_node, dropdown));
	}
}

fn spawn_dropdown(container_node: &mut ChildBuilder, dropdown: &Dropdown) {
	container_node
		.spawn(NodeBundle {
			style: get_style(dropdown),
			..default()
		})
		.with_children(|dropdown_node| spawn_items(dropdown_node, dropdown));
}

fn get_style(dropdown: &Dropdown) -> Style {
	match &dropdown.layout {
		Layout::MaxColumn(max_index) => Style {
			display: Display::Grid,
			grid_template_columns: RepeatedGridTrack::auto(max_index.0 + 1),
			..default()
		},
		Layout::MaxRow(max_index) => Style {
			display: Display::Grid,
			grid_template_rows: RepeatedGridTrack::auto(max_index.0 + 1),
			..default()
		},
	}
}

fn spawn_items(dropdown_node: &mut ChildBuilder, dropdown: &Dropdown) {
	for item in &dropdown.items {
		dropdown_node
			.spawn(item.node())
			.with_children(|item_node| item.instantiate_content_on(item_node));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		tools::Layout,
		traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn},
	};
	use bevy::{
		app::{App, Update},
		hierarchy::{BuildWorldChildren, ChildBuilder, Parent},
		prelude::Component,
		ui::{node_bundles::NodeBundle, Display, RepeatedGridTrack, Style, Val},
		utils::default,
	};
	use common::{assert_bundle, test_tools::utils::SingleThreadedApp, tools::Index};
	use mockall::mock;

	mock! {
		_Item {}
		impl GetNode for _Item {
			fn node(&self) -> NodeBundle;
		}
		impl InstantiateContentOn for _Item {
			fn instantiate_content_on<'a>(&self, parent: &mut ChildBuilder<'a>);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, dropdown);

		app
	}

	macro_rules! spawn_dropdown {
		($app:expr, $dropdown:expr) => {{
			let container = $app.world.spawn_empty().id();
			let dropdown = $app.world.spawn($dropdown).set_parent(container).id();

			(container, dropdown)
		}};
	}

	macro_rules! last_child_of {
		($app:expr, $entity:expr) => {
			$app.world
				.iter_entities()
				.filter_map(|e| {
					if e.get::<Parent>()?.get() == $entity {
						Some(e)
					} else {
						None
					}
				})
				.last()
				.unwrap_or_else(|| panic!("Entity {:?} has no child", $entity))
		};
	}

	#[test]
	fn spawn_dropdown_node_as_next_child_of_container() {
		let mut app = setup();

		let (container, dropdown) = spawn_dropdown!(app, Dropdown::default());

		app.update();

		let dropdown_node = last_child_of!(app, container);

		assert_ne!(dropdown, dropdown_node.id());
		assert_bundle!(NodeBundle, &app, dropdown_node);
	}

	#[test]
	fn spawn_only_one_dropdown_node() {
		let mut app = setup();

		let (container, ..) = spawn_dropdown!(app, Dropdown::default());

		app.update();

		let first_node = last_child_of!(app, container).id();

		app.update();

		let second_node = last_child_of!(app, container).id();

		assert_eq!(first_node, second_node);
	}

	#[test]
	fn spawn_dropdown_item_node() {
		let mut app = setup();
		let mut item = Box::new(Mock_Item::default());
		item.expect_node().return_const(NodeBundle {
			style: Style {
				top: Val::Px(42.),
				..default()
			},
			..default()
		});
		item.expect_instantiate_content_on().return_const(());

		let (container, ..) = spawn_dropdown!(
			app,
			Dropdown {
				items: vec![item],
				..default()
			}
		);

		app.update();

		let dropdown_node = last_child_of!(app, container);
		let item_node = last_child_of!(app, dropdown_node.id());

		assert_bundle!(
			NodeBundle,
			&app,
			item_node,
			With::assert(|style: &Style| assert_eq!(
				&Style {
					top: Val::Px(42.),
					..default()
				},
				style
			))
		)
	}

	#[test]
	fn instantiate_dropdown_item_content() {
		#[derive(Component, Debug, PartialEq)]
		struct _Content(&'static str);

		let mut app = setup();
		let mut item = Box::new(Mock_Item::default());
		item.expect_node().return_const(NodeBundle::default());
		item.expect_instantiate_content_on().returning(|item_node| {
			item_node.spawn(_Content("My Content"));
		});

		let (container, ..) = spawn_dropdown!(
			app,
			Dropdown {
				items: vec![item],
				..default()
			}
		);

		app.update();

		let dropdown_node = last_child_of!(app, container);
		let item_node = last_child_of!(app, dropdown_node.id());
		let item_content = last_child_of!(app, item_node.id());

		assert_eq!(
			Some(&_Content("My Content")),
			item_content.get::<_Content>(),
		);
	}

	struct _Item;

	impl GetNode for _Item {
		fn node(&self) -> NodeBundle {
			NodeBundle::default()
		}
	}

	impl InstantiateContentOn for _Item {
		fn instantiate_content_on(&self, _: &mut ChildBuilder) {}
	}

	#[test]
	fn set_grid_for_column_limited_size_3() {
		let mut app = setup();

		let (container, ..) = spawn_dropdown!(
			app,
			Dropdown {
				items: vec![
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
				],
				layout: Layout::MaxColumn(Index(2)),
			}
		);

		app.update();

		let dropdown_node = last_child_of!(app, container);

		assert_bundle!(
			NodeBundle,
			&app,
			dropdown_node,
			With::assert(|style: &Style| assert_eq!(
				&Style {
					display: Display::Grid,
					grid_template_columns: RepeatedGridTrack::auto(3),
					..default()
				},
				style
			))
		);
	}

	#[test]
	fn set_grid_for_column_limited_size_2() {
		let mut app = setup();

		let (container, ..) = spawn_dropdown!(
			app,
			Dropdown {
				items: vec![
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
				],
				layout: Layout::MaxColumn(Index(1)),
			}
		);

		app.update();

		let dropdown_node = last_child_of!(app, container);

		assert_bundle!(
			NodeBundle,
			&app,
			dropdown_node,
			With::assert(|style: &Style| assert_eq!(
				&Style {
					display: Display::Grid,
					grid_template_columns: RepeatedGridTrack::auto(2),
					..default()
				},
				style
			))
		);
	}

	#[test]
	fn set_grid_for_row_limited_size_3() {
		let mut app = setup();

		let (container, ..) = spawn_dropdown!(
			app,
			Dropdown {
				items: vec![
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
				],
				layout: Layout::MaxRow(Index(2)),
			}
		);

		app.update();

		let dropdown_node = last_child_of!(app, container);

		assert_bundle!(
			NodeBundle,
			&app,
			dropdown_node,
			With::assert(|style: &Style| assert_eq!(
				&Style {
					display: Display::Grid,
					grid_template_rows: RepeatedGridTrack::auto(3),
					..default()
				},
				style
			))
		);
	}

	#[test]
	fn set_grid_for_row_limited_size_2() {
		let mut app = setup();

		let (container, ..) = spawn_dropdown!(
			app,
			Dropdown {
				items: vec![
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
				],
				layout: Layout::MaxRow(Index(1)),
			}
		);

		app.update();

		let dropdown_node = last_child_of!(app, container);

		assert_bundle!(
			NodeBundle,
			&app,
			dropdown_node,
			With::assert(|style: &Style| assert_eq!(
				&Style {
					display: Display::Grid,
					grid_template_rows: RepeatedGridTrack::auto(2),
					..default()
				},
				style
			))
		);
	}
}
