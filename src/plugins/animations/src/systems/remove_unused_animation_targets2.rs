use bevy::{animation::AnimationTarget, prelude::*};
use common::{
	traits::{accessors::get::TryApplyOn, wrap_handle::UnwrapHandle},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> RemoveUnusedAnimationTargets2 for T where
	T: Component + UnwrapHandle<TAsset = AnimationGraph>
{
}

pub(crate) trait RemoveUnusedAnimationTargets2:
	Component + UnwrapHandle<TAsset = AnimationGraph>
{
	fn remove_unused_animation_targets2(
		mut commands: ZyheedaCommands,
		graphs: Res<Assets<AnimationGraph>>,
		players: Query<(Entity, &AnimationGraphHandle), Added<AnimationGraphHandle>>,
		bones: Query<(Entity, &AnimationTarget)>,
		children: Query<&Children>,
	) {
		for (player, AnimationGraphHandle(handle)) in &players {
			let Some(graph) = graphs.get(handle) else {
				continue;
			};

			for entity in children.iter_descendants(player) {
				let Ok((entity, target)) = bones.get(entity) else {
					continue;
				};

				if target.player != player {
					continue;
				}

				if graph.mask_groups.contains_key(&target.id) {
					continue;
				}

				commands.try_apply_on(&entity, |mut e| {
					e.try_remove::<AnimationTarget>();
				});
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::animation::{AnimationTarget, AnimationTargetId};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Graph(Handle<AnimationGraph>);

	impl UnwrapHandle for _Graph {
		type TAsset = AnimationGraph;

		fn unwrap(&self) -> &Handle<Self::TAsset> {
			&self.0
		}
	}

	fn setup(handle: &Handle<AnimationGraph>, graph: AnimationGraph) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut graphs = Assets::default();

		graphs.insert(handle, graph);
		app.insert_resource(graphs);
		app.add_systems(Update, _Graph::remove_unused_animation_targets2);

		app
	}

	fn new_graph<const N: usize>(targets: [AnimationTargetId; N]) -> AnimationGraph {
		let mut graph = AnimationGraph::new();
		for target in targets {
			graph.mask_groups.insert(target, AnimationMask::default());
		}
		graph
	}

	#[test]
	fn remove_unused_animation_targets() {
		let used_targets = [
			AnimationTargetId::from_name(&Name::from("a")),
			AnimationTargetId::from_name(&Name::from("b")),
			AnimationTargetId::from_name(&Name::from("c")),
		];
		let handle = new_handle();
		let mut app = setup(&handle, new_graph(used_targets));
		let player = app.world_mut().spawn(AnimationGraphHandle(handle)).id();
		let targets = [
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("d")),
					player,
				})
				.insert(ChildOf(player))
				.id(),
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("e")),
					player,
				})
				.insert(ChildOf(player))
				.id(),
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("f")),
					player,
				})
				.insert(ChildOf(player))
				.id(),
		];

		app.update();

		assert_eq!(
			[false, false, false],
			app.world()
				.entity(targets)
				.map(|entity| entity.contains::<AnimationTarget>())
		)
	}

	#[test]
	fn remove_unused_animation_targets_when_deeply_nested() {
		let used_targets = [
			AnimationTargetId::from_name(&Name::from("a")),
			AnimationTargetId::from_name(&Name::from("b")),
			AnimationTargetId::from_name(&Name::from("c")),
		];
		let handle = new_handle();
		let mut app = setup(&handle, new_graph(used_targets));
		let player = app.world_mut().spawn(AnimationGraphHandle(handle)).id();
		let targets = [
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("d")),
					player,
				})
				.insert(ChildOf(player))
				.id(),
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("e")),
					player,
				})
				.id(),
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("f")),
					player,
				})
				.id(),
		];
		app.world_mut()
			.entity_mut(targets[1])
			.insert(ChildOf(targets[0]));
		app.world_mut()
			.entity_mut(targets[2])
			.insert(ChildOf(targets[1]));

		app.update();

		assert_eq!(
			[false, false, false],
			app.world()
				.entity(targets)
				.map(|entity| entity.contains::<AnimationTarget>())
		)
	}

	#[test]
	fn do_not_remove_when_not_child_of_player() {
		let used_targets = [
			AnimationTargetId::from_name(&Name::from("a")),
			AnimationTargetId::from_name(&Name::from("b")),
			AnimationTargetId::from_name(&Name::from("c")),
		];
		let handle = new_handle();
		let mut app = setup(&handle, new_graph(used_targets));
		let player = app.world_mut().spawn(AnimationGraphHandle(handle)).id();
		let targets = [
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("d")),
					player,
				})
				.id(),
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("e")),
					player,
				})
				.id(),
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("f")),
					player,
				})
				.id(),
		];

		app.update();

		assert_eq!(
			[true, true, true],
			app.world()
				.entity(targets)
				.map(|entity| entity.contains::<AnimationTarget>())
		)
	}
	#[test]
	fn do_not_remove_when_not_target_not_linked_to_player() {
		let used_targets = [
			AnimationTargetId::from_name(&Name::from("a")),
			AnimationTargetId::from_name(&Name::from("b")),
			AnimationTargetId::from_name(&Name::from("c")),
		];
		let handle = new_handle();
		let mut app = setup(&handle, new_graph(used_targets));
		let player = app.world_mut().spawn(AnimationGraphHandle(handle)).id();
		let other = app.world_mut().spawn_empty().id();
		let targets = [
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("d")),
					player: other,
				})
				.insert(ChildOf(player))
				.id(),
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("e")),
					player: other,
				})
				.insert(ChildOf(player))
				.id(),
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("f")),
					player: other,
				})
				.insert(ChildOf(player))
				.id(),
		];

		app.update();

		assert_eq!(
			[true, true, true],
			app.world()
				.entity(targets)
				.map(|entity| entity.contains::<AnimationTarget>())
		)
	}

	#[test]
	fn do_not_remove_used_animation_targets() {
		let used_targets = [
			AnimationTargetId::from_name(&Name::from("a")),
			AnimationTargetId::from_name(&Name::from("b")),
			AnimationTargetId::from_name(&Name::from("c")),
		];
		let handle = new_handle();
		let mut app = setup(&handle, new_graph(used_targets));
		let player = app.world_mut().spawn(AnimationGraphHandle(handle)).id();
		let targets =
			used_targets.map(|id| app.world_mut().spawn(AnimationTarget { id, player }).id());

		app.update();

		assert_eq!(
			[true, true, true],
			app.world()
				.entity(targets)
				.map(|entity| entity.contains::<AnimationTarget>())
		)
	}

	#[test]
	fn act_only_once() {
		let used_targets = [
			AnimationTargetId::from_name(&Name::from("a")),
			AnimationTargetId::from_name(&Name::from("b")),
			AnimationTargetId::from_name(&Name::from("c")),
		];
		let handle = new_handle();
		let mut app = setup(&handle, new_graph(used_targets));
		let player = app.world_mut().spawn(AnimationGraphHandle(handle)).id();
		let targets = [
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("d")),
					player,
				})
				.insert(ChildOf(player))
				.id(),
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("e")),
					player,
				})
				.insert(ChildOf(player))
				.id(),
			app.world_mut()
				.spawn(AnimationTarget {
					id: AnimationTargetId::from_name(&Name::from("f")),
					player,
				})
				.insert(ChildOf(player))
				.id(),
		];

		app.update();
		for target in targets {
			app.world_mut().entity_mut(target).insert(AnimationTarget {
				id: AnimationTargetId::from_name(&Name::from("should not be removed")),
				player,
			});
		}
		app.update();

		assert_eq!(
			[true, true, true],
			app.world()
				.entity(targets)
				.map(|entity| entity.contains::<AnimationTarget>())
		)
	}
}
