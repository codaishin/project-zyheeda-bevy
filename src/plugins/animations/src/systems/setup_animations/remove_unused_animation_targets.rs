use crate::components::{animation_lookup::AnimationLookup, setup_animations::SetupAnimations};
use bevy::{animation::AnimationTarget, prelude::*};
use common::{
	traits::{accessors::get::TryApplyOn, wrap_handle::GetHandle},
	zyheeda_commands::ZyheedaCommands,
};

impl SetupAnimations {
	#[allow(clippy::type_complexity)]
	pub(crate) fn remove_unused_animation_targets<TGraph>(
		mut commands: ZyheedaCommands,
		graphs: Res<Assets<AnimationGraph>>,
		players: Query<(Entity, &TGraph), (With<AnimationLookup>, With<Self>)>,
		bones: Query<(Entity, &AnimationTarget)>,
		children: Query<&Children>,
	) where
		TGraph: Component + GetHandle<TAsset = AnimationGraph>,
	{
		for (player, graph) in &players {
			let Some(graph) = graphs.get(graph.get_handle()) else {
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
	use crate::components::animation_lookup::AnimationClips;
	use bevy::animation::{AnimationTarget, AnimationTargetId};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Graph(Handle<AnimationGraph>);

	impl GetHandle for _Graph {
		type TAsset = AnimationGraph;

		fn get_handle(&self) -> &Handle<Self::TAsset> {
			&self.0
		}
	}

	fn setup(handle: &Handle<AnimationGraph>, graph: AnimationGraph) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut graphs = Assets::default();

		_ = graphs.insert(handle, graph);
		app.insert_resource(graphs);
		app.add_systems(
			Update,
			SetupAnimations::remove_unused_animation_targets::<_Graph>,
		);

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
		let player = app
			.world_mut()
			.spawn((
				_Graph(handle),
				SetupAnimations,
				AnimationLookup::<AnimationClips>::default(),
			))
			.id();
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
		let player = app
			.world_mut()
			.spawn((
				_Graph(handle),
				SetupAnimations,
				AnimationLookup::<AnimationClips>::default(),
			))
			.id();
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
		let player = app
			.world_mut()
			.spawn((
				_Graph(handle),
				SetupAnimations,
				AnimationLookup::<AnimationClips>::default(),
			))
			.id();
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
		let player = app
			.world_mut()
			.spawn((
				_Graph(handle),
				SetupAnimations,
				AnimationLookup::<AnimationClips>::default(),
			))
			.id();
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
		let player = app
			.world_mut()
			.spawn((
				_Graph(handle),
				SetupAnimations,
				AnimationLookup::<AnimationClips>::default(),
			))
			.id();
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
	fn do_nothing_when_not_setting_up_animations() {
		let used_targets = [
			AnimationTargetId::from_name(&Name::from("a")),
			AnimationTargetId::from_name(&Name::from("b")),
			AnimationTargetId::from_name(&Name::from("c")),
		];
		let handle = new_handle();
		let mut app = setup(&handle, new_graph(used_targets));
		let player = app
			.world_mut()
			.spawn((_Graph(handle), AnimationLookup::<AnimationClips>::default()))
			.id();
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
			[true, true, true],
			app.world()
				.entity(targets)
				.map(|entity| entity.contains::<AnimationTarget>())
		)
	}

	#[test]
	fn do_nothing_when_no_animation_lookup_present() {
		let used_targets = [
			AnimationTargetId::from_name(&Name::from("a")),
			AnimationTargetId::from_name(&Name::from("b")),
			AnimationTargetId::from_name(&Name::from("c")),
		];
		let handle = new_handle();
		let mut app = setup(&handle, new_graph(used_targets));
		let player = app
			.world_mut()
			.spawn((_Graph(handle), SetupAnimations))
			.id();
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
			[true, true, true],
			app.world()
				.entity(targets)
				.map(|entity| entity.contains::<AnimationTarget>())
		)
	}
}
