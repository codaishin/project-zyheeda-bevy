use crate::{
	components::dropdown::Dropdown,
	tools::Layout,
	traits::{GetLayout, RootStyle, UI},
};
use bevy::{
	hierarchy::{BuildChildren, ChildBuilder},
	prelude::{Commands, Component, Entity, In, Query},
	ui::{node_bundles::NodeBundle, Display, RepeatedGridTrack, Style, ZIndex},
	utils::default,
};
use common::tools::Focus;
use std::{fmt::Debug, marker::PhantomData};

#[derive(Component)]
pub(crate) struct DropdownUI<TItem> {
	phantom_data: PhantomData<TItem>,
	pub(crate) source: Entity,
}

impl<TItem> DropdownUI<TItem> {
	pub(crate) fn new(source: Entity) -> Self {
		Self {
			source,
			phantom_data: PhantomData,
		}
	}
}

impl<TItem> Debug for DropdownUI<TItem> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("DropdownUI")
			.field("phantom_data", &self.phantom_data)
			.field("source", &self.source)
			.finish()
	}
}

impl<TItem> PartialEq for DropdownUI<TItem> {
	fn eq(&self, other: &Self) -> bool {
		self.source == other.source
	}
}

pub(crate) fn dropdown_spawn_focused<TItem>(
	focus: In<Focus>,
	mut commands: Commands,
	dropdowns: Query<(Entity, &Dropdown<TItem>)>,
) where
	TItem: UI + Sync + Send + 'static,
	Dropdown<TItem>: RootStyle + GetLayout,
{
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
					DropdownUI::<TItem>::new(source),
					NodeBundle {
						style: dropdown.root_style(),
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

fn get_style<TItem>(dropdown: &Dropdown<TItem>) -> Style
where
	Dropdown<TItem>: GetLayout,
{
	match &dropdown.layout() {
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

fn spawn_items<TItem: UI>(dropdown_node: &mut ChildBuilder, dropdown: &Dropdown<TItem>) {
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

	macro_rules! impl_item {
		($item:ident) => {
			struct $item;

			impl GetNode for $item {
				fn node(&self) -> NodeBundle {
					NodeBundle::default()
				}
			}

			impl InstantiateContentOn for $item {
				fn instantiate_content_on(&self, _: &mut ChildBuilder) {}
			}
		};
	}

	macro_rules! impl_dropdown {
		($item:ident) => {
			impl_item! {$item}

			impl RootStyle for Dropdown<$item> {
				fn root_style(&self) -> Style {
					Style::default()
				}
			}

			impl GetLayout for Dropdown<$item> {
				fn layout(&self) -> Layout {
					Layout::default()
				}
			}
		};
	}

	macro_rules! impl_dropdown_with_layout {
		($item:ident, $layout:expr) => {
			impl_item! {$item}

			impl RootStyle for Dropdown<$item> {
				fn root_style(&self) -> Style {
					Style::default()
				}
			}

			impl GetLayout for Dropdown<$item> {
				fn layout(&self) -> Layout {
					$layout
				}
			}
		};
	}

	macro_rules! impl_dropdown_with_style {
		($item:ident, $style:expr) => {
			impl_item! {$item}

			impl RootStyle for Dropdown<$item> {
				fn root_style(&self) -> Style {
					$style
				}
			}

			impl GetLayout for Dropdown<$item> {
				fn layout(&self) -> Layout {
					Layout::default()
				}
			}
		};
	}

	mock! {
		_Item {}
		impl GetNode for _Item {
			fn node(&self) -> NodeBundle;
		}
		impl InstantiateContentOn for _Item {
			fn instantiate_content_on<'a>(&self, parent: &mut ChildBuilder<'a>);
		}
	}

	impl RootStyle for Dropdown<Mock_Item> {
		fn root_style(&self) -> Style {
			Style::default()
		}
	}

	impl GetLayout for Dropdown<Mock_Item> {
		fn layout(&self) -> Layout {
			Layout::SINGLE_COLUMN
		}
	}

	#[derive(Resource, Default)]
	struct _In(Focus);

	fn setup<TItem>() -> App
	where
		TItem: UI + Send + Sync + 'static,
		Dropdown<TItem>: RootStyle + GetLayout,
	{
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_In>();
		app.add_systems(
			Update,
			(|focus: Res<_In>| focus.0.clone()).pipe(dropdown_spawn_focused::<TItem>),
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
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();

		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown);

		assert_bundle!(NodeBundle, &app, dropdown_ui);
	}

	#[test]
	fn spawn_dropdown_ui_with_dropdown_ui_marker() {
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();

		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown);

		assert_eq!(
			Some(&DropdownUI::<_Item>::new(dropdown)),
			dropdown_ui.get::<DropdownUI<_Item>>()
		);
	}

	#[test]
	fn spawn_dropdown_ui_with_dropdown_style() {
		impl_dropdown_with_style!(
			_Item,
			Style {
				top: Val::Px(404.),
				..default()
			}
		);

		let mut app = setup::<_Item>();

		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
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
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();

		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
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
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();

		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		app.world_mut().insert_resource(_In(Focus::New(vec![])));

		app.update();

		let last_child = try_last_child_of!(app, dropdown);

		assert!(last_child.is_none());
	}

	#[test]
	fn spawn_dropdown_ui_content_with_node_bundle() {
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();

		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui);

		assert_bundle!(NodeBundle, &app, dropdown_ui_content);
	}

	#[test]
	fn spawn_dropdown_item_node_with_node_bundle() {
		let mut app = setup::<Mock_Item>();
		let mut item = Mock_Item::default();
		item.expect_node().return_const(NodeBundle {
			style: Style {
				top: Val::Px(42.),
				..default()
			},
			..default()
		});
		item.expect_instantiate_content_on().return_const(());

		let dropdown = app.world_mut().spawn(Dropdown { items: vec![item] }).id();
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

		let mut app = setup::<Mock_Item>();
		let mut item = Mock_Item::default();
		item.expect_node().return_const(NodeBundle::default());
		item.expect_instantiate_content_on().returning(|item_node| {
			item_node.spawn(_Content("My Content"));
		});

		let dropdown = app.world_mut().spawn(Dropdown { items: vec![item] }).id();
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

	#[test]
	fn set_grid_for_column_limited_size_3() {
		impl_dropdown_with_layout!(_Item, Layout::LastColumn(Index(2)));

		let mut app = setup::<_Item>();

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				items: vec![_Item, _Item, _Item, _Item, _Item],
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
		impl_dropdown_with_layout!(_Item, Layout::LastColumn(Index(1)));

		let mut app = setup::<_Item>();

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				items: vec![_Item, _Item, _Item, _Item, _Item],
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
		impl_dropdown_with_layout!(_Item, Layout::LastRow(Index(2)));

		let mut app = setup::<_Item>();

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				items: vec![_Item, _Item, _Item, _Item, _Item],
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
		impl_dropdown_with_layout!(_Item, Layout::LastRow(Index(1)));

		let mut app = setup::<_Item>();

		let dropdown = app
			.world_mut()
			.spawn(Dropdown {
				items: vec![_Item, _Item, _Item, _Item, _Item],
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
