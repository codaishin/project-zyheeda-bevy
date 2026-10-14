use crate::components::icon::{IconStack, IconStackItem};
use bevy::prelude::*;

impl IconStack {
	const OVERLAP_PERCENT: usize = 30;

	pub(crate) fn format_root(stacks: Query<&mut Node, Added<Self>>) {
		for mut node in stacks {
			node.flex_direction = FlexDirection::Column;
		}
	}

	pub(crate) fn format_items(
		stack_items: Query<(&mut Node, &IconStackItem), Added<IconStackItem>>,
		stacks: Query<&Self>,
	) {
		for (mut node, IconStackItem { of, index }) in stack_items {
			let Ok(stack) = stacks.get(*of) else {
				continue;
			};

			let count = stack.len();
			let dimensions = (100 + count.saturating_sub(1) * Self::OVERLAP_PERCENT) / count;

			node.width = percent(dimensions);
			node.height = percent(dimensions);

			if count < 2 {
				continue;
			}

			node.margin.left = percent(index * (dimensions - Self::OVERLAP_PERCENT));
			node.margin.bottom = percent(-(Self::OVERLAP_PERCENT as isize));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::icon::IconStackItem;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				IconStack::format_root,
				IconStack::format_items,
				IsChanged::<Node>::detect,
			)
				.chain(),
		);

		app
	}

	mod root {
		use super::*;

		#[test]
		fn set_to_flex_direction_column() {
			let mut app = setup();
			let stack = app.world_mut().spawn(IconStack::default()).id();

			app.update();

			assert_eq!(
				Some(&Node {
					flex_direction: FlexDirection::Column,
					..default()
				}),
				app.world().entity(stack).get::<Node>(),
			);
		}

		#[test]
		fn act_only_once() {
			let mut app = setup();
			let stack = app.world_mut().spawn(IconStack::default()).id();

			app.update();
			app.update();

			assert_eq!(
				Some(&IsChanged::FALSE),
				app.world().entity(stack).get::<IsChanged<Node>>(),
			);
		}
	}

	mod items {
		use super::*;

		#[test]
		fn single_item() {
			let mut app = setup();
			let stack = app.world_mut().spawn_empty().id();
			let item = app.world_mut().spawn(IconStackItem::of(stack)).id();

			app.update();

			assert_eq!(
				Some(&Node {
					width: percent(100),
					height: percent(100),
					margin: UiRect {
						bottom: percent(0),
						..default()
					},
					..default()
				}),
				app.world().entity(item).get::<Node>(),
			);
		}

		#[test]
		fn two_items() {
			let mut app = setup();
			let stack = app.world_mut().spawn_empty().id();
			let items = [
				app.world_mut()
					.spawn(IconStackItem::of(stack).with_index(0))
					.id(),
				app.world_mut()
					.spawn(IconStackItem::of(stack).with_index(1))
					.id(),
			];

			app.update();

			assert_eq!(
				[
					Some(&Node {
						width: percent(65),
						height: percent(65),
						margin: UiRect {
							bottom: percent(-30),
							..default()
						},
						..default()
					}),
					Some(&Node {
						width: percent(65),
						height: percent(65),
						margin: UiRect {
							bottom: percent(-30),
							left: percent(35),
							..default()
						},
						..default()
					}),
				],
				app.world().entity(items).map(|e| e.get::<Node>())
			);
		}

		#[test]
		fn three_items() {
			let mut app = setup();
			let stack = app.world_mut().spawn_empty().id();
			let items = [
				app.world_mut()
					.spawn(IconStackItem::of(stack).with_index(0))
					.id(),
				app.world_mut()
					.spawn(IconStackItem::of(stack).with_index(1))
					.id(),
				app.world_mut()
					.spawn(IconStackItem::of(stack).with_index(2))
					.id(),
			];

			app.update();

			assert_eq!(
				[
					Some(&Node {
						width: percent(53),
						height: percent(53),
						margin: UiRect {
							bottom: percent(-30),
							..default()
						},
						..default()
					}),
					Some(&Node {
						width: percent(53),
						height: percent(53),
						margin: UiRect {
							bottom: percent(-30),
							left: percent(23),
							..default()
						},
						..default()
					}),
					Some(&Node {
						width: percent(53),
						height: percent(53),
						margin: UiRect {
							bottom: percent(-30),
							left: percent(46),
							..default()
						},
						..default()
					}),
				],
				app.world().entity(items).map(|e| e.get::<Node>()),
			);
		}

		#[test]
		fn act_only_once() {
			let mut app = setup();
			let stack = app.world_mut().spawn_empty().id();
			let item = app.world_mut().spawn(IconStackItem::of(stack)).id();

			app.update();
			app.update();

			assert_eq!(
				Some(&IsChanged::FALSE),
				app.world().entity(item).get::<IsChanged<Node>>(),
			);
		}
	}
}
