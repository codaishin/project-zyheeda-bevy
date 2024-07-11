use crate::{components::dropdown::Dropdown, tools::Layout};
use bevy::{
	hierarchy::{BuildChildren, ChildBuilder},
	prelude::{Commands, Component, Entity, In, Query},
	ui::{node_bundles::NodeBundle, Display, RepeatedGridTrack, Style, ZIndex},
	utils::default,
};
use common::tools::Focus;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct DropdownUI {
	pub(crate) source: Entity,
}

pub(crate) fn dropdown_spawn_focused(
	focus: In<Focus>,
	mut commands: Commands,
	dropdowns: Query<(Entity, &Dropdown)>,
) {
	let Focus::New(new_focus) = focus.0 else {
		return;
	};

	for (source, dropdown) in &dropdowns {
		if !new_focus.contains(&source) {
			continue;
		}
		let Some(mut entity) = commands.get_entity(source) else {
			continue;
		};

		entity.with_children(|entity_node| {
			entity_node
				.spawn((
					DropdownUI { source },
					NodeBundle {
						style: dropdown.style.clone(),
						z_index: ZIndex::Global(1),
						..default()
					},
				))
				.with_children(|container_node| {
					container_node
						.spawn(NodeBundle {
							style: get_style(dropdown),
							..default()
						})
						.with_children(|dropdown_node| spawn_items(dropdown_node, dropdown));
				});
		});
	}
}

fn get_style(dropdown: &Dropdown) -> Style {
	match &dropdown.layout {
		Layout::LastColumn(max_index) => {
			let (limit, auto) = repetitions(dropdown.items.len(), max_index.0);
			Style {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(limit),
				grid_template_rows: RepeatedGridTrack::auto(auto),
				..default()
			}
		}
		Layout::LastRow(max_index) => {
			let (limit, auto) = repetitions(dropdown.items.len(), max_index.0);
			Style {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(auto),
				grid_template_rows: RepeatedGridTrack::auto(limit),
				..default()
			}
		}
	}
}

fn repetitions(count: usize, max_index: u16) -> (u16, u16) {
	let count = count + 1;
	let limit = max_index + 1;

	(limit, count as u16 / limit)
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
		hierarchy::{ChildBuilder, Parent},
		prelude::{Component, IntoSystem, Res, Resource},
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

	#[derive(Resource, Default)]
	struct _In(Focus);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_In>();
		app.add_systems(
			Update,
			(|focus: Res<_In>| focus.0.clone()).pipe(dropdown_spawn_focused),
		);

		app
	}

	macro_rules! try_last_child_of {
		($app:expr, $entity:expr) => {
			$app.world()
				.iter_entities()
				.filter_map(|e| {
					if e.get::<Parent>()?.get() == $entity {
						Some(e)
					} else {
						None
					}
				})
				.last()
		};
	}

	macro_rules! last_child_of {
		($app:expr, $entity:expr) => {
			try_last_child_of!($app, $entity)
				.unwrap_or_else(|| panic!("Entity {:?} has no child", $entity))
		};
	}

	#[test]
	fn spawn_dropdown_ui_as_child_of_self() {
		let mut app = setup();

		let dropdown = app.world_mut().spawn(Dropdown::default()).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown);

		assert_bundle!(NodeBundle, &app, dropdown_ui);
	}

	#[test]
	fn spawn_dropdown_ui_with_dropdown_ui_marker() {
		let mut app = setup();

		let dropdown = app.world_mut().spawn(Dropdown::default()).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown);

		assert_eq!(
			Some(&DropdownUI { source: dropdown }),
			dropdown_ui.get::<DropdownUI>()
		);
	}

	#[test]
	fn spawn_dropdown_ui_with_dropdown_style() {
		let mut app = setup();

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				style: Style {
					top: Val::Px(404.),
					..default()
				},
				..default()
			})
			.id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown);

		assert_eq!(
			Some(&Style {
				top: Val::Px(404.),
				..default()
			}),
			dropdown_ui.get::<Style>(),
		);
	}

	#[test]
	fn spawn_dropdown_ui_with_global_z_index_1() {
		let mut app = setup();

		let dropdown = app.world_mut().spawn(Dropdown::default()).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown);

		assert_eq!(
			Some(1),
			dropdown_ui.get::<ZIndex>().map(|index| match index {
				ZIndex::Global(index) => *index,
				_ => -1,
			}),
		);
	}

	#[test]
	fn do_not_spawn_dropdown_ui_when_not_new_active() {
		let mut app = setup();

		let dropdown = app.world_mut().spawn(Dropdown::default()).id();
		app.world_mut().insert_resource(_In(Focus::New(vec![])));

		app.update();

		let last_child = try_last_child_of!(app, dropdown);

		assert!(last_child.is_none());
	}

	#[test]
	fn spawn_dropdown_ui_content_with_node_bundle() {
		let mut app = setup();

		let dropdown = app.world_mut().spawn(Dropdown::default()).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui);

		assert_bundle!(NodeBundle, &app, dropdown_ui_content);
	}

	#[test]
	fn spawn_dropdown_item_node_with_node_bundle() {
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

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				items: vec![item],
				..default()
			})
			.id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui).id();
		let item_node = last_child_of!(app, dropdown_ui_content);

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

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				items: vec![item],
				..default()
			})
			.id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui).id();
		let item_node = last_child_of!(app, dropdown_ui_content).id();
		let item_content = last_child_of!(app, item_node);

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

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				items: vec![
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
				],
				layout: Layout::LastColumn(Index(2)),
				..default()
			})
			.id();

		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui);

		assert_eq!(
			Some(&Style {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(3),
				grid_template_rows: RepeatedGridTrack::auto(2),
				..default()
			}),
			dropdown_ui_content.get::<Style>()
		);
	}

	#[test]
	fn set_grid_for_column_limited_size_2() {
		let mut app = setup();

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				items: vec![
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
				],
				layout: Layout::LastColumn(Index(1)),
				..default()
			})
			.id();

		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui);

		assert_eq!(
			Some(&Style {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(2),
				grid_template_rows: RepeatedGridTrack::auto(3),
				..default()
			}),
			dropdown_ui_content.get::<Style>()
		);
	}

	#[test]
	fn set_grid_for_row_limited_size_3() {
		let mut app = setup();

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				items: vec![
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
				],
				layout: Layout::LastRow(Index(2)),
				..default()
			})
			.id();

		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui);

		assert_eq!(
			Some(&Style {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(2),
				grid_template_rows: RepeatedGridTrack::auto(3),
				..default()
			}),
			dropdown_ui_content.get::<Style>()
		);
	}

	#[test]
	fn set_grid_for_row_limited_size_2() {
		let mut app = setup();

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				items: vec![
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
					Box::new(_Item),
				],
				layout: Layout::LastRow(Index(1)),
				..default()
			})
			.id();

		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui);

		assert_eq!(
			Some(&Style {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(3),
				grid_template_rows: RepeatedGridTrack::auto(2),
				..default()
			}),
			dropdown_ui_content.get::<Style>()
		);
	}
}
