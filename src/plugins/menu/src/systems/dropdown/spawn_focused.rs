use crate::{
	components::{
		GlobalZIndexTop,
		dropdown::{Dropdown, DropdownUI},
	},
	tools::Layout,
	traits::{GetLayout, GetRootNode, insert_ui_content::InsertUiContent},
};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{
	tools::Focus,
	traits::{handles_localization::Localize, thread_safe::ThreadSafe},
};

pub(crate) fn dropdown_spawn_focused<TLocalization, TItem>(
	focus: In<Focus>,
	mut commands: Commands,
	localization: Res<TLocalization>,
	dropdowns: Query<(Entity, &Dropdown<TItem>)>,
) where
	TLocalization: Localize + Resource,
	TItem: InsertUiContent + Sync + Send + 'static,
	Dropdown<TItem>: GetRootNode + GetLayout,
{
	let Focus::New(new_focus) = focus.0 else {
		return;
	};

	for (source, dropdown) in &dropdowns {
		if !new_focus.contains(&source) {
			continue;
		}
		let Ok(mut entity) = commands.get_entity(source) else {
			continue;
		};

		entity.with_children(|entity_node| {
			entity_node
				.spawn((
					GlobalZIndexTop,
					DropdownUI::<TItem>::new(source),
					dropdown.root_node(),
				))
				.with_children(|container_node| {
					container_node
						.spawn(get_node(dropdown))
						.with_children(|dropdown_node| {
							spawn_items(localization.as_ref(), dropdown_node, dropdown)
						});
				});
		});
	}
}

fn get_node<TItem>(dropdown: &Dropdown<TItem>) -> Node
where
	Dropdown<TItem>: GetLayout,
{
	match &dropdown.layout() {
		Layout::LastColumn(max_index) => {
			let (limit, auto) = repetitions(dropdown.items.len(), max_index.0);
			Node {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(limit),
				grid_template_rows: RepeatedGridTrack::auto(auto),
				..default()
			}
		}
		Layout::LastRow(max_index) => {
			let (limit, auto) = repetitions(dropdown.items.len(), max_index.0);
			Node {
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

fn spawn_items<TLocalization, TItem>(
	localization: &TLocalization,
	dropdown_node: &mut RelatedSpawnerCommands<ChildOf>,
	dropdown: &Dropdown<TItem>,
) where
	TItem: InsertUiContent,
	TLocalization: Localize + ThreadSafe,
{
	for item in &dropdown.items {
		dropdown_node
			.spawn(Node::default())
			.with_children(|item_node| item.insert_ui_content(localization, item_node));
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	use crate::{
		components::GlobalZIndexTop,
		tools::Layout,
		traits::insert_ui_content::InsertUiContent,
	};
	use common::{
		tools::Index,
		traits::handles_localization::{LocalizationResult, Token, localized::Localized},
	};
	use testing::SingleThreadedApp;

	#[derive(Resource, Default, Debug, PartialEq, Clone, Copy)]
	struct _Localization;

	impl Localize for _Localization {
		fn localize(&self, token: &Token) -> LocalizationResult {
			let token = &**token;

			LocalizationResult::Ok(Localized::from(format!("Token: {token}")))
		}
	}

	macro_rules! impl_item {
		($item:ident) => {
			#[derive(Debug, PartialEq)]
			struct $item;

			impl InsertUiContent for $item {
				fn insert_ui_content<TLocalization>(
					&self,
					_: &TLocalization,
					_: &mut RelatedSpawnerCommands<ChildOf>,
				) {
				}
			}
		};
	}

	macro_rules! impl_dropdown {
		($item:ident) => {
			impl_item! {$item}

			impl GetRootNode for Dropdown<$item> {
				fn root_node(&self) -> Node {
					Node::default()
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

			impl GetRootNode for Dropdown<$item> {
				fn root_node(&self) -> Node {
					Node::default()
				}
			}

			impl GetLayout for Dropdown<$item> {
				fn layout(&self) -> Layout {
					$layout
				}
			}
		};
	}

	macro_rules! impl_dropdown_with_node {
		($item:ident, $node:expr) => {
			impl_item! {$item}

			impl GetRootNode for Dropdown<$item> {
				fn root_node(&self) -> Node {
					$node
				}
			}

			impl GetLayout for Dropdown<$item> {
				fn layout(&self) -> Layout {
					Layout::default()
				}
			}
		};
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ItemContent(Localized);

	impl From<&str> for _ItemContent {
		fn from(value: &str) -> Self {
			Self(Localized::from(value))
		}
	}

	struct _Item;

	impl InsertUiContent for _Item {
		fn insert_ui_content<TLocalization>(
			&self,
			localization: &TLocalization,
			parent: &mut RelatedSpawnerCommands<ChildOf>,
		) where
			TLocalization: Localize,
		{
			parent.spawn(_ItemContent(
				localization
					.localize(&Token::from("Content"))
					.or_string(|| "???"),
			));
		}
	}

	impl GetRootNode for Dropdown<_Item> {
		fn root_node(&self) -> Node {
			Node::default()
		}
	}

	impl GetLayout for Dropdown<_Item> {
		fn layout(&self) -> Layout {
			Layout::SINGLE_COLUMN
		}
	}

	#[derive(Resource, Default)]
	struct _In(Focus);

	fn setup<TItem>() -> App
	where
		TItem: InsertUiContent + Send + Sync + 'static,
		Dropdown<TItem>: GetRootNode + GetLayout,
	{
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_In>();
		app.init_resource::<_Localization>();
		app.add_systems(
			Update,
			(|focus: Res<_In>| focus.0.clone())
				.pipe(dropdown_spawn_focused::<_Localization, TItem>),
		);

		app
	}

	macro_rules! try_last_child_of {
		($app:expr, $entity:expr) => {
			$app.world()
				.iter_entities()
				.filter_map(|e| {
					if e.get::<ChildOf>()?.parent() == $entity {
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

		assert!(dropdown_ui.contains::<Node>());
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
		impl_dropdown_with_node!(
			_Item,
			Node {
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
			Some(&Node {
				top: Val::Px(404.),
				..default()
			}),
			dropdown_ui.get::<Node>(),
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
	fn spawn_dropdown_ui_content_with_node() {
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();

		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui);

		assert!(dropdown_ui_content.contains::<Node>());
	}

	#[test]
	fn spawn_dropdown_item_container_node() {
		let mut app = setup::<_Item>();
		let item = _Item;

		let dropdown = app.world_mut().spawn(Dropdown { items: vec![item] }).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui).id();
		let item_node = last_child_of!(app, dropdown_ui_content);

		assert_eq!(Some(&Node::default()), item_node.get::<Node>());
	}

	#[test]
	fn instantiate_dropdown_item_content() {
		let mut app = setup::<_Item>();
		let item = _Item;

		let dropdown = app.world_mut().spawn(Dropdown { items: vec![item] }).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown).id();
		let dropdown_ui_content = last_child_of!(app, dropdown_ui).id();
		let item_node = last_child_of!(app, dropdown_ui_content).id();
		let item_content = last_child_of!(app, item_node);

		assert_eq!(
			Some(&_ItemContent::from("Token: Content")),
			item_content.get::<_ItemContent>(),
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
			Some(&Node {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(3),
				grid_template_rows: RepeatedGridTrack::auto(2),
				..default()
			}),
			dropdown_ui_content.get::<Node>()
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
			Some(&Node {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(2),
				grid_template_rows: RepeatedGridTrack::auto(3),
				..default()
			}),
			dropdown_ui_content.get::<Node>()
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
			Some(&Node {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(2),
				grid_template_rows: RepeatedGridTrack::auto(3),
				..default()
			}),
			dropdown_ui_content.get::<Node>()
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
			Some(&Node {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(3),
				grid_template_rows: RepeatedGridTrack::auto(2),
				..default()
			}),
			dropdown_ui_content.get::<Node>()
		);
	}

	#[test]
	fn spawn_dropdown_ui_with_global_z_index_top() {
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();

		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		app.world_mut()
			.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_ui = last_child_of!(app, dropdown);

		assert_eq!(Some(&GlobalZIndexTop), dropdown_ui.get::<GlobalZIndexTop>());
	}
}
