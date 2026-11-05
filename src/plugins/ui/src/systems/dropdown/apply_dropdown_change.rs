use crate::{
	components::dropdown::{Dropdown, DropdownActive},
	resources::dropdown_change::DropdownChanged,
	tools::Layout,
	traits::{GetContainerNode, GetLayout, insert_ui_content::InsertUiContent},
};
use bevy::{
	ecs::{
		relationship::RelatedSpawnerCommands,
		system::{StaticSystemParam, SystemParam},
	},
	prelude::*,
};
use common::prelude::*;
use std::ops::Deref;

impl DropdownChanged {
	pub(crate) fn apply<TLocalization, TItem>(
		changed: Res<DropdownChanged>,
		mut commands: ZyheedaCommands,
		localization: StaticSystemParam<TLocalization>,
		dropdowns: Query<(&Dropdown<TItem>, Ref<DropdownActive>)>,
	) where
		TLocalization: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TItem: InsertUiContent + Sync + Send + 'static,
		Dropdown<TItem>: GetContainerNode + GetLayout,
	{
		if !*changed {
			return;
		}

		for (dropdown, active) in &dropdowns {
			match Op::of(active) {
				Op::Retain => continue,
				Op::Init => {
					commands.try_apply_on(&active.container(), |mut e| {
						e.try_insert(dropdown.get_container_node()).with_children(
							|container_node| {
								container_node.spawn(layout(dropdown)).with_children(
									|dropdown_node| {
										spawn_items(localization.deref(), dropdown_node, dropdown)
									},
								);
							},
						);
					});
				}
				Op::Remove => {
					commands.try_apply_on(&active.container(), |e| e.try_despawn());
				}
			}
		}
	}
}

enum Op {
	Retain,
	Init,
	Remove,
}

impl Op {
	fn of(active: Ref<DropdownActive>) -> Self {
		if active.is_added() {
			return Self::Init;
		}

		if active.is_changed() {
			return Self::Retain;
		}

		Self::Remove
	}
}

fn layout<TItem>(dropdown: &Dropdown<TItem>) -> Node
where
	Dropdown<TItem>: GetLayout,
{
	match &dropdown.get_layout() {
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
	TLocalization: Localize,
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
		components::dropdown::{DropdownActive, DropdownContainerOf},
		tools::Layout,
		traits::insert_ui_content::InsertUiContent,
	};
	use testing::{SingleThreadedApp, assert_related_count};

	#[derive(Resource, Default, Debug, PartialEq, Clone, Copy)]
	struct _Localization;

	impl Localize for _Localization {
		fn localize(&self, token: &Token) -> LocalizationResult {
			LocalizationResult::Ok(Localized::from(format!("Localized({token})")))
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

			impl GetContainerNode for Dropdown<$item> {
				fn get_container_node(&self) -> Node {
					Node::default()
				}
			}

			impl GetLayout for Dropdown<$item> {
				fn get_layout(&self) -> Layout {
					Layout::default()
				}
			}
		};
	}

	macro_rules! impl_dropdown_with_layout {
		($item:ident, $layout:expr) => {
			impl_item! {$item}

			impl GetContainerNode for Dropdown<$item> {
				fn get_container_node(&self) -> Node {
					Node::default()
				}
			}

			impl GetLayout for Dropdown<$item> {
				fn get_layout(&self) -> Layout {
					$layout
				}
			}
		};
	}

	macro_rules! impl_dropdown_with_node {
		($item:ident, $node:expr) => {
			impl_item! {$item}

			impl GetContainerNode for Dropdown<$item> {
				fn get_container_node(&self) -> Node {
					$node
				}
			}

			impl GetLayout for Dropdown<$item> {
				fn get_layout(&self) -> Layout {
					Layout::default()
				}
			}
		};
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ItemContent(Localized);

	impl<T> From<T> for _ItemContent
	where
		T: Into<Localized>,
	{
		fn from(value: T) -> Self {
			Self(value.into())
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

	impl GetContainerNode for Dropdown<_Item> {
		fn get_container_node(&self) -> Node {
			Node::default()
		}
	}

	impl GetLayout for Dropdown<_Item> {
		fn get_layout(&self) -> Layout {
			Layout::SINGLE_COLUMN
		}
	}

	fn setup<TItem>() -> App
	where
		TItem: InsertUiContent + Send + Sync + 'static,
		Dropdown<TItem>: GetContainerNode + GetLayout,
	{
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<DropdownChanged>();
		app.init_resource::<_Localization>();
		app.add_systems(Update, DropdownChanged::apply::<Res<_Localization>, TItem>);

		app
	}

	#[test]
	fn add_root_style() {
		impl_dropdown_with_node!(
			_Item,
			Node {
				top: Val::Px(404.),
				..default()
			}
		);

		let mut app = setup::<_Item>();
		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		let container = app.world_mut().spawn(DropdownContainerOf(dropdown)).id();
		app.insert_resource(DropdownChanged(true));

		app.update();

		assert_eq!(
			Some(&Node {
				top: Val::Px(404.),
				..default()
			}),
			app.world().entity(container).get::<Node>()
		);
	}

	#[test]
	fn spawn_layout_style() {
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();
		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		let container = app.world_mut().spawn(DropdownContainerOf(dropdown)).id();
		app.insert_resource(DropdownChanged(true));

		app.update();

		let [layout] = assert_related_count!(ChildOf, 1, app, container);
		assert!(layout.contains::<Node>());
	}

	#[test]
	fn spawn_dropdown_item_container_node() {
		let mut app = setup::<_Item>();
		let dropdown = app.world_mut().spawn(Dropdown { items: vec![_Item] }).id();
		let container = app.world_mut().spawn(DropdownContainerOf(dropdown)).id();
		app.insert_resource(DropdownChanged(true));

		app.update();

		let [layout] = assert_related_count!(ChildOf, 1, app, container, |e| Some(e.id()));
		let [item] = assert_related_count!(ChildOf, 1, app, layout);
		assert_eq!(Some(&Node::default()), item.get::<Node>());
	}

	#[test]
	fn instantiate_dropdown_item_content() {
		let mut app = setup::<_Item>();
		let dropdown = app.world_mut().spawn(Dropdown { items: vec![_Item] }).id();
		let container = app.world_mut().spawn(DropdownContainerOf(dropdown)).id();
		app.insert_resource(DropdownChanged(true));

		app.update();

		let [layout] = assert_related_count!(ChildOf, 1, app, container, |e| Some(e.id()));
		let [item] = assert_related_count!(ChildOf, 1, app, layout, |e| Some(e.id()));
		let [content] = assert_related_count!(ChildOf, 1, app, item);
		assert_eq!(
			Some(&_ItemContent::from(format!(
				"Localized({})",
				Token::from("Content")
			))),
			content.get::<_ItemContent>(),
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
		let container = app.world_mut().spawn(DropdownContainerOf(dropdown)).id();
		app.insert_resource(DropdownChanged(true));

		app.update();

		let [layout] = assert_related_count!(ChildOf, 1, app, container);
		assert_eq!(
			Some(&Node {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(3),
				grid_template_rows: RepeatedGridTrack::auto(2),
				..default()
			}),
			layout.get::<Node>()
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
		let container = app.world_mut().spawn(DropdownContainerOf(dropdown)).id();
		app.insert_resource(DropdownChanged(true));

		app.update();

		let [layout] = assert_related_count!(ChildOf, 1, app, container);
		assert_eq!(
			Some(&Node {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(2),
				grid_template_rows: RepeatedGridTrack::auto(3),
				..default()
			}),
			layout.get::<Node>()
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
		let container = app.world_mut().spawn(DropdownContainerOf(dropdown)).id();
		app.insert_resource(DropdownChanged(true));

		app.update();

		let [layout] = assert_related_count!(ChildOf, 1, app, container);

		assert_eq!(
			Some(&Node {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(2),
				grid_template_rows: RepeatedGridTrack::auto(3),
				..default()
			}),
			layout.get::<Node>()
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
		let container = app.world_mut().spawn(DropdownContainerOf(dropdown)).id();
		app.insert_resource(DropdownChanged(true));

		app.update();

		let [layout] = assert_related_count!(ChildOf, 1, app, container);

		assert_eq!(
			Some(&Node {
				display: Display::Grid,
				grid_template_columns: RepeatedGridTrack::auto(3),
				grid_template_rows: RepeatedGridTrack::auto(2),
				..default()
			}),
			layout.get::<Node>()
		);
	}

	#[test]
	fn do_nothing_if_changed_false() {
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();
		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		let container = app.world_mut().spawn(DropdownContainerOf(dropdown)).id();
		app.insert_resource(DropdownChanged(false));

		app.update();

		assert_eq!(None, app.world().entity(container).get::<Node>());
	}

	#[test]
	fn act_only_once() {
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();
		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		let container = app.world_mut().spawn(DropdownContainerOf(dropdown)).id();
		app.insert_resource(DropdownChanged(true));

		app.update();
		app.insert_resource(DropdownChanged(true));
		app.world_mut().entity_mut(container).remove::<Node>();
		// refresh active
		app.world_mut()
			.entity_mut(dropdown)
			.get_mut::<DropdownActive>()
			.as_deref_mut();
		app.update();

		assert!(!app.world().entity(container).contains::<Node>());
	}

	#[test]
	fn despawn_non_refreshed() {
		impl_dropdown!(_Item);

		let mut app = setup::<_Item>();
		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();
		app.world_mut().spawn(DropdownContainerOf(dropdown));
		app.insert_resource(DropdownChanged(true));

		app.update();
		app.insert_resource(DropdownChanged(true));
		app.update();

		assert_eq!(None, app.world().entity(dropdown).get::<DropdownActive>());
	}
}
