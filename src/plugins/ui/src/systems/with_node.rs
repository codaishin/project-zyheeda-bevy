use crate::components::with_node::WithNode;
use bevy::prelude::*;
use common::prelude::*;

impl WithNode {
	pub(crate) fn system(
		mut commands: ZyheedaCommands,
		nodes: Query<(Entity, &Self, Option<&mut Node>)>,
	) {
		for (entity, Self(mod_func), node) in nodes {
			let Some(mut entity) = commands.get_mut(&entity) else {
				continue;
			};

			match node {
				Some(mut node) => {
					mod_func(&mut node);
				}
				None => {
					let mut node = Node::default();

					mod_func(&mut node);

					entity.try_insert(node);
				}
			}

			entity.try_remove::<WithNode>();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, WithNode::system);

		app
	}

	#[test]
	fn modify() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Node {
					width: px(10),
					height: px(10),
					..default()
				},
				WithNode::modify_or_insert(|n| n.height = px(100)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Node {
				width: px(10),
				height: px(100),
				..default()
			}),
			app.world().entity(entity).get::<Node>()
		);
	}

	#[test]
	fn insert() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(WithNode::modify_or_insert(|n| n.height = px(100)))
			.id();

		app.update();

		assert_eq!(
			Some(&Node {
				height: px(100),
				..default()
			}),
			app.world().entity(entity).get::<Node>()
		);
	}

	#[test]
	fn remove_component() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Node {
					width: px(10),
					height: px(10),
					..default()
				},
				WithNode::modify_or_insert(|n| n.height = px(100)),
			))
			.id();

		app.update();

		assert!(!app.world().entity(entity).contains::<WithNode>());
	}
}
