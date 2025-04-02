use crate::{components::animation_dispatch::AnimationDispatch, traits::AnimationPlayers};
use bevy::{
	animation::{AnimationTarget, AnimationTargetId},
	prelude::*,
};
use common::traits::animation::{AnimationMaskRoot, GetAnimationDefinitions};

impl<TAgent> DiscoverMaskChains for TAgent
where
	TAgent: GetAnimationDefinitions + Component,
	for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
	for<'a> AnimationMaskRoot: From<&'a TAgent::TAnimationMask>,
{
}

pub(crate) trait DiscoverMaskChains: GetAnimationDefinitions + Component + Sized
where
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AnimationMaskRoot: From<&'a Self::TAnimationMask>,
{
	fn set_animation_mask_bones(
		mut graphs: ResMut<Assets<AnimationGraph>>,
		agents: Query<&AnimationDispatch, With<Self>>,
		roots: Query<&AnimationGraphHandle, Added<AnimationGraphHandle>>,
		children: Query<&Children>,
		bones: Query<(&Name, &AnimationTarget)>,
	) {
		let animation_masks = Self::animation_definitions()
			.into_iter()
			.filter_map(|(animation_mask, ..)| animation_mask)
			.collect::<Vec<_>>();

		for dispatch in &agents {
			for root in dispatch.animation_players() {
				let Ok(AnimationGraphHandle(handle)) = roots.get(root) else {
					continue;
				};
				let Some(graph) = graphs.get_mut(handle) else {
					continue;
				};
				let chains = get_mask_bones(root, &children, &bones, &animation_masks);

				update_graph(graph, chains);
			}
		}
	}
}

fn get_mask_bones<TAnimationMask>(
	root: Entity,
	children: &Query<&Children>,
	bones: &Query<(&Name, &AnimationTarget)>,
	animation_masks: &[TAnimationMask],
) -> Vec<(AnimationTargetId, AnimationMask)>
where
	for<'a> AnimationMask: From<&'a TAnimationMask>,
	for<'a> AnimationMaskRoot: From<&'a TAnimationMask>,
{
	let mut r_bones = vec![];
	let get_bone = |child| {
		let (name, target) = bones.get(child).ok()?;
		if target.player != root {
			return None;
		}
		Some((child, name, target))
	};

	for mask in animation_masks {
		let AnimationMaskRoot(mask_root) = AnimationMaskRoot::from(mask);
		let animation_mask = AnimationMask::from(mask);

		let mask_root = match bones.get(root).ok() {
			Some((name, target)) if name == &mask_root => Some((root, name, target)),
			_ => children
				.iter_descendants(root)
				.filter_map(get_bone)
				.find(|(_, name, _)| name == &&mask_root),
		};
		let Some((entity, _, target)) = mask_root else {
			continue;
		};

		r_bones.push((target.id, animation_mask));
		for (.., target) in children.iter_descendants(entity).filter_map(get_bone) {
			r_bones.push((target.id, animation_mask));
		}
	}

	r_bones
}

fn update_graph(graph: &mut AnimationGraph, mask_bones: Vec<(AnimationTargetId, AnimationMask)>) {
	for (target, mask) in mask_bones {
		*graph.mask_groups.entry(target).or_default() |= mask;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::animation_dispatch::AnimationDispatch;
	use bevy::{animation::AnimationTargetId, utils::HashMap};
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::{animation::AnimationMaskRoot, load_asset::Path},
	};

	struct _Mask {
		id: AnimationMask,
		root: AnimationMaskRoot,
	}

	impl From<&_Mask> for AnimationMask {
		fn from(_Mask { id, .. }: &_Mask) -> Self {
			*id
		}
	}

	impl From<&_Mask> for AnimationMaskRoot {
		fn from(_Mask { root, .. }: &_Mask) -> Self {
			root.clone()
		}
	}

	macro_rules! agent_animation_definitions {
		($definitions:expr) => {
			#[derive(Component)]
			struct _Agent;

			impl GetAnimationDefinitions for _Agent {
				type TAnimationMask = _Mask;

				fn animation_definitions() -> Vec<(Option<Self::TAnimationMask>, Path)> {
					$definitions
				}
			}
		};
	}

	fn bone_components<const N: usize>(
		bone_chain: [&str; N],
		player: Entity,
	) -> (Name, AnimationTarget) {
		let names = bone_chain.map(Name::from);

		match names.as_slice() {
			[] => panic!("AT LEAST ONE BONE NAME REQUIRED"),
			[bones @ .., last] => (
				last.clone(),
				AnimationTarget {
					player,
					id: AnimationTargetId::from_names(bones.iter().chain([last])),
				},
			),
		}
	}

	fn setup<TAgent>(handle: &Handle<AnimationGraph>) -> App
	where
		TAgent: Component + GetAnimationDefinitions,
		for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
		for<'a> AnimationMaskRoot: From<&'a TAgent::TAnimationMask>,
	{
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		assets.insert(handle, AnimationGraph::new());
		app.insert_resource(assets);
		app.add_systems(Update, TAgent::set_animation_mask_bones);

		app
	}

	#[test]
	fn set_when_root() {
		agent_animation_definitions!(vec![(
			Some(_Mask {
				id: 1,
				root: AnimationMaskRoot(Name::from("root")),
			}),
			Path::from(""),
		)]);
		let handle = new_handle();
		let mut app = setup::<_Agent>(&handle);
		let root = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		app.world_mut()
			.spawn((Name::from("agent"), _Agent, AnimationDispatch::to([root])));

		app.update();

		assert_eq!(
			HashMap::from([(AnimationTargetId::from_name(&Name::from("root")), 1)]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.expect("NO MATCHING GRAPH")
				.mask_groups
		);
	}

	#[test]
	fn set_when_root_multiple_masks() {
		agent_animation_definitions!(vec![
			(
				Some(_Mask {
					id: 1,
					root: AnimationMaskRoot(Name::from("root")),
				}),
				Path::from(""),
			),
			(
				Some(_Mask {
					id: 2,
					root: AnimationMaskRoot(Name::from("root")),
				}),
				Path::from(""),
			)
		]);
		let handle = new_handle();
		let mut app = setup::<_Agent>(&handle);
		let root = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		app.world_mut()
			.spawn((Name::from("agent"), _Agent, AnimationDispatch::to([root])));

		app.update();

		assert_eq!(
			HashMap::from([(AnimationTargetId::from_name(&Name::from("root")), 3)]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.expect("NO MATCHING GRAPH")
				.mask_groups
		);
	}

	#[test]
	fn set_chain() {
		agent_animation_definitions!(vec![(
			Some(_Mask {
				id: 1,
				root: AnimationMaskRoot(Name::from("mask root")),
			}),
			Path::from(""),
		)]);
		let handle = new_handle();
		let mut app = setup::<_Agent>(&handle);
		let root = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		app.world_mut()
			.spawn(bone_components(["root", "mask root"], root))
			.set_parent(root);
		app.world_mut()
			.spawn((Name::from("agent"), _Agent, AnimationDispatch::to([root])));

		app.update();

		assert_eq!(
			HashMap::from([(
				AnimationTargetId::from_names([Name::from("root"), Name::from("mask root")].iter()),
				1
			)]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.expect("NO MATCHING GRAPH")
				.mask_groups
		);
	}

	#[test]
	fn ignore_path_not_containing_mask_root() {
		agent_animation_definitions!(vec![(
			Some(_Mask {
				id: 1,
				root: AnimationMaskRoot(Name::from("mask root")),
			}),
			Path::from(""),
		)]);
		let handle = new_handle();
		let mut app = setup::<_Agent>(&handle);
		let root = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		app.world_mut()
			.spawn(bone_components(["root", "mask root"], root))
			.set_parent(root);
		app.world_mut()
			.spawn(bone_components(["root", "not mask root"], root))
			.set_parent(root);
		app.world_mut()
			.spawn((Name::from("agent"), _Agent, AnimationDispatch::to([root])));

		app.update();

		assert_eq!(
			HashMap::from([(
				AnimationTargetId::from_names([Name::from("root"), Name::from("mask root")].iter()),
				1
			)]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.expect("NO MATCHING GRAPH")
				.mask_groups
		);
	}

	#[test]
	fn add_multiple_names_below_mask_root_when_single_chain() {
		agent_animation_definitions!(vec![(
			Some(_Mask {
				id: 1,
				root: AnimationMaskRoot(Name::from("mask root")),
			}),
			Path::from(""),
		)]);
		let handle = new_handle();
		let mut app = setup::<_Agent>(&handle);
		let root = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		let mask_root = app
			.world_mut()
			.spawn(bone_components(["root", "mask root"], root))
			.set_parent(root)
			.id();
		let child_a = app
			.world_mut()
			.spawn(bone_components(["root", "mask root", "child a"], root))
			.set_parent(mask_root)
			.id();
		app.world_mut()
			.spawn(bone_components(
				["root", "mask root", "child a", "child b"],
				root,
			))
			.set_parent(child_a);
		app.world_mut()
			.spawn((Name::from("agent"), _Agent, AnimationDispatch::to([root])));

		app.update();

		assert_eq!(
			HashMap::from([
				(
					AnimationTargetId::from_names(
						[Name::from("root"), Name::from("mask root")].iter()
					),
					1
				),
				(
					AnimationTargetId::from_names(
						[
							Name::from("root"),
							Name::from("mask root"),
							Name::from("child a"),
						]
						.iter()
					),
					1
				),
				(
					AnimationTargetId::from_names(
						[
							Name::from("root"),
							Name::from("mask root"),
							Name::from("child a"),
							Name::from("child b"),
						]
						.iter()
					),
					1
				)
			]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.expect("NO MATCHING GRAPH")
				.mask_groups
		);
	}

	#[test]
	fn add_names_below_mask_root_when_not_single_chain() {
		agent_animation_definitions!(vec![(
			Some(_Mask {
				id: 1,
				root: AnimationMaskRoot(Name::from("mask root")),
			}),
			Path::from(""),
		)]);
		let handle = new_handle();
		let mut app = setup::<_Agent>(&handle);
		let root = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		let mask_root = app
			.world_mut()
			.spawn(bone_components(["root", "mask root"], root))
			.set_parent(root)
			.id();
		app.world_mut()
			.spawn(bone_components(["root", "mask root", "child a"], root))
			.set_parent(mask_root);
		app.world_mut()
			.spawn(bone_components(["root", "mask root", "child b"], root))
			.set_parent(mask_root);
		app.world_mut()
			.spawn((Name::from("agent"), _Agent, AnimationDispatch::to([root])));

		app.update();

		assert_eq!(
			HashMap::from([
				(
					AnimationTargetId::from_names(
						[Name::from("root"), Name::from("mask root")].iter()
					),
					1
				),
				(
					AnimationTargetId::from_names(
						[
							Name::from("root"),
							Name::from("mask root"),
							Name::from("child a"),
						]
						.iter()
					),
					1
				),
				(
					AnimationTargetId::from_names(
						[
							Name::from("root"),
							Name::from("mask root"),
							Name::from("child b"),
						]
						.iter()
					),
					1
				)
			]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.expect("NO MATCHING GRAPH")
				.mask_groups
		);
	}

	#[test]
	fn ignore_targets_not_belonging_to_root() {
		agent_animation_definitions!(vec![(
			Some(_Mask {
				id: 1,
				root: AnimationMaskRoot(Name::from("mask root")),
			}),
			Path::from(""),
		)]);
		let handle = new_handle();
		let mut app = setup::<_Agent>(&handle);
		let root = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		let mask_root = app
			.world_mut()
			.spawn(bone_components(["root", "mask root"], root))
			.set_parent(root)
			.id();
		app.world_mut()
			.spawn(bone_components(["other"], Entity::from_raw(42)))
			.set_parent(mask_root);
		app.world_mut()
			.spawn((Name::from("agent"), _Agent, AnimationDispatch::to([root])));

		app.update();

		assert_eq!(
			HashMap::from([(
				AnimationTargetId::from_names([Name::from("root"), Name::from("mask root")].iter()),
				1
			)]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.expect("NO MATCHING GRAPH")
				.mask_groups
		);
	}

	#[test]
	fn act_only_once() {
		agent_animation_definitions!(vec![(
			Some(_Mask {
				id: 1,
				root: AnimationMaskRoot(Name::from("root")),
			}),
			Path::from(""),
		)]);
		let handle = new_handle();
		let mut app = setup::<_Agent>(&handle);
		let root = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		app.world_mut()
			.spawn((Name::from("agent"), _Agent, AnimationDispatch::to([root])));

		app.update();
		app.world_mut()
			.resource_mut::<Assets<AnimationGraph>>()
			.get_mut(&handle)
			.expect("NO MATCHING GRAPH")
			.mask_groups
			.clear();
		app.update();

		assert_eq!(
			HashMap::from([]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.expect("NO MATCHING GRAPH")
				.mask_groups
		);
	}

	#[test]
	fn do_not_act_when_agent_missing_on_dispatch() {
		agent_animation_definitions!(vec![(
			Some(_Mask {
				id: 1,
				root: AnimationMaskRoot(Name::from("root")),
			}),
			Path::from(""),
		)]);
		let handle = new_handle();
		let mut app = setup::<_Agent>(&handle);
		let root = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		app.world_mut()
			.spawn((Name::from("agent"), AnimationDispatch::to([root])));

		app.update();

		assert_eq!(
			HashMap::from([]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.expect("NO MATCHING GRAPH")
				.mask_groups
		);
	}
}
