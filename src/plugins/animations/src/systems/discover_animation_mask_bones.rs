use crate::{components::animation_dispatch::AnimationDispatch, traits::AnimationPlayers};
use bevy::{
	animation::{AnimationTarget, AnimationTargetId},
	prelude::*,
};
use common::traits::{
	animation::{AffectedAnimationBones, GetAnimationDefinitions},
	iteration::IterFinite,
};
use std::iter;

impl<TAgent> DiscoverMaskChains for TAgent
where
	TAgent: GetAnimationDefinitions + Component,
	for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
	for<'a> AffectedAnimationBones: From<&'a TAgent::TAnimationMask>,
{
}

pub(crate) trait DiscoverMaskChains: GetAnimationDefinitions + Component + Sized
where
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AffectedAnimationBones: From<&'a Self::TAnimationMask>,
{
	fn set_animation_mask_bones(
		mut graphs: ResMut<Assets<AnimationGraph>>,
		agents: Query<&AnimationDispatch, With<Self>>,
		players: Query<&AnimationGraphHandle, Added<AnimationGraphHandle>>,
		children: Query<&Children>,
		bones: Query<(&Name, &AnimationTarget)>,
	) {
		let animation_masks = Self::TAnimationMask::iterator().collect::<Vec<_>>();

		for dispatch in &agents {
			for player in dispatch.animation_players() {
				let Ok(AnimationGraphHandle(handle)) = players.get(player) else {
					continue;
				};
				let Some(graph) = graphs.get_mut(handle) else {
					continue;
				};
				let chains = get_mask_bones(player, &children, &bones, &animation_masks);

				update_graph(graph, chains);
			}
		}
	}
}

fn get_mask_bones<TAnimationMask>(
	player: Entity,
	children: &Query<&Children>,
	bones: &Query<(&Name, &AnimationTarget)>,
	animation_masks: &[TAnimationMask],
) -> Vec<(AnimationTargetId, AnimationMask)>
where
	for<'a> AnimationMask: From<&'a TAnimationMask>,
	for<'a> AffectedAnimationBones: From<&'a TAnimationMask>,
{
	let mut r_bones = vec![];
	let get_bone = |child| {
		let (name, target) = bones.get(child).ok()?;
		if target.player != player {
			return None;
		}
		Some((child, name, target))
	};

	for mask in animation_masks {
		let bones = match AffectedAnimationBones::from(mask) {
			AffectedAnimationBones::Leaf { root: from_root } => {
				mask_bones(player, &from_root, children, &get_bone)
					.map(|bones| bones.collect::<Vec<_>>())
			}
			AffectedAnimationBones::SubTree {
				root: from_root,
				until_exclusive: exclude_roots,
			} => mask_bones_with_exclusions(player, from_root, exclude_roots, children, &get_bone),
		};

		let Some(bones) = bones else {
			continue;
		};

		let animation_mask = AnimationMask::from(mask);
		for bone in bones {
			r_bones.push((bone, animation_mask));
		}
	}

	r_bones
}

fn update_graph(graph: &mut AnimationGraph, mask_bones: Vec<(AnimationTargetId, AnimationMask)>) {
	for (target, mask) in mask_bones {
		*graph.mask_groups.entry(target).or_default() |= mask;
	}
}

fn mask_bones_with_exclusions<'a>(
	player: Entity,
	mask_root: Name,
	exclude_root: Vec<Name>,
	children: &Query<'_, '_, &Children>,
	get_bone: &'a impl Fn(Entity) -> Option<(Entity, &'a Name, &'a AnimationTarget)>,
) -> Option<Vec<AnimationTargetId>> {
	let exclude = exclude_root
		.iter()
		.filter_map(|mask_root| mask_bones(player, mask_root, children, get_bone))
		.flatten()
		.collect::<Vec<_>>();

	Some(
		mask_bones(player, &mask_root, children, get_bone)
			.into_iter()
			.flatten()
			.filter(|bone| !exclude.contains(bone))
			.collect::<Vec<_>>(),
	)
}

fn mask_bones<'a>(
	player: Entity,
	mask_root: &Name,
	children: &Query<&Children>,
	get_bone: &'a impl Fn(Entity) -> Option<(Entity, &'a Name, &'a AnimationTarget)>,
) -> Option<impl Iterator<Item = AnimationTargetId>> {
	let (entity, _, target) = root_bone(player, mask_root, children, get_bone)?;
	let children = children
		.iter_descendants(entity)
		.filter_map(get_bone)
		.map(|(.., target)| target.id);

	Some(iter::once(target.id).chain(children))
}

fn root_bone<'a>(
	player: Entity,
	mask_root: &Name,
	children: &Query<'_, '_, &Children>,
	get_bone: &'a impl Fn(Entity) -> Option<(Entity, &'a Name, &'a AnimationTarget)>,
) -> Option<(Entity, &'a Name, &'a AnimationTarget)> {
	match get_bone(player) {
		Some((entity, name, target)) if name == mask_root => Some((entity, name, target)),
		_ => children
			.iter_descendants(player)
			.filter_map(get_bone)
			.find(|(_, name, _)| name == &mask_root),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::animation_dispatch::AnimationDispatch;
	use bevy::{animation::AnimationTargetId, platform::collections::HashMap};
	use common::traits::{
		animation::{AffectedAnimationBones, AnimationPath},
		iteration::{Iter, IterFinite},
	};
	use testing::{SingleThreadedApp, new_handle};

	macro_rules! agent_animation_definitions {
		($masks:expr) => {
			#[derive(Debug, Clone, Copy)]
			struct _Mask {
				id: AnimationMask,
				def: fn() -> AffectedAnimationBones,
			}

			impl PartialEq for _Mask {
				fn eq(&self, other: &Self) -> bool {
					self.id == other.id && std::ptr::fn_addr_eq(self.def, other.def)
				}
			}

			impl From<&_Mask> for AnimationMask {
				fn from(_Mask { id, .. }: &_Mask) -> Self {
					*id
				}
			}

			impl From<&_Mask> for AffectedAnimationBones {
				fn from(_Mask { def, .. }: &_Mask) -> Self {
					def()
				}
			}

			static MASKS: &[_Mask] = $masks;

			impl IterFinite for _Mask {
				fn iterator() -> Iter<Self> {
					Iter(MASKS.get(0).copied())
				}

				fn next(current: &Iter<Self>) -> Option<Self> {
					let Iter(Some(current)) = current else {
						return None;
					};

					let pos = MASKS.iter().position(|v| v == current)?;

					MASKS.get(pos + 1).copied()
				}
			}

			#[derive(Component)]
			struct _Agent;

			impl GetAnimationDefinitions for _Agent {
				type TAnimationMask = _Mask;

				fn animations() -> std::collections::HashMap<AnimationPath, AnimationMask> {
					panic!("SHOULD NOT BE USED HERE")
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
		for<'a> AffectedAnimationBones: From<&'a TAgent::TAnimationMask>,
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
		agent_animation_definitions!(&[_Mask {
			id: 1,
			def: || AffectedAnimationBones::Leaf {
				root: Name::from("root")
			},
		}]);
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
		agent_animation_definitions!(&[
			_Mask {
				id: 1,
				def: || AffectedAnimationBones::Leaf {
					root: Name::from("root")
				},
			},
			_Mask {
				id: 2,
				def: || AffectedAnimationBones::Leaf {
					root: Name::from("root")
				},
			}
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
		agent_animation_definitions!(&[_Mask {
			id: 1,
			def: || AffectedAnimationBones::Leaf {
				root: Name::from("mask root")
			},
		}]);
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
			.insert(ChildOf(root));
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
		agent_animation_definitions!(&[_Mask {
			id: 1,
			def: || AffectedAnimationBones::Leaf {
				root: Name::from("mask root")
			},
		}]);
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
			.insert(ChildOf(root));
		app.world_mut()
			.spawn(bone_components(["root", "not mask root"], root))
			.insert(ChildOf(root));
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
		agent_animation_definitions!(&[_Mask {
			id: 1,
			def: || AffectedAnimationBones::Leaf {
				root: Name::from("mask root")
			},
		}]);
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
			.insert(ChildOf(root))
			.id();
		let child_a = app
			.world_mut()
			.spawn(bone_components(["root", "mask root", "child a"], root))
			.insert(ChildOf(mask_root))
			.id();
		app.world_mut()
			.spawn(bone_components(
				["root", "mask root", "child a", "child b"],
				root,
			))
			.insert(ChildOf(child_a));
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
		agent_animation_definitions!(&[_Mask {
			id: 1,
			def: || AffectedAnimationBones::Leaf {
				root: Name::from("mask root")
			},
		}]);
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
			.insert(ChildOf(root))
			.id();
		app.world_mut()
			.spawn(bone_components(["root", "mask root", "child a"], root))
			.insert(ChildOf(mask_root));
		app.world_mut()
			.spawn(bone_components(["root", "mask root", "child b"], root))
			.insert(ChildOf(mask_root));
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
		agent_animation_definitions!(&[_Mask {
			id: 1,
			def: || AffectedAnimationBones::Leaf {
				root: Name::from("mask root")
			},
		}]);
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
			.insert(ChildOf(root))
			.id();
		app.world_mut()
			.spawn(bone_components(["other"], Entity::from_raw(42)))
			.insert(ChildOf(mask_root));
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
	fn set_exclusion_mask() {
		agent_animation_definitions!(&[_Mask {
			id: 1,
			def: || AffectedAnimationBones::SubTree {
				root: Name::from("root"),
				until_exclusive: vec![Name::from("a"), Name::from("b"),]
			},
		}]);
		let handle = new_handle();
		let mut app = setup::<_Agent>(&handle);
		let root = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		let child = app
			.world_mut()
			.spawn(bone_components(["root", "child"], root))
			.insert(ChildOf(root))
			.id();
		let a = app
			.world_mut()
			.spawn(bone_components(["root", "child", "a"], root))
			.insert(ChildOf(child))
			.id();
		app.world_mut()
			.spawn(bone_components(["root", "child", "a", "a child"], root))
			.insert(ChildOf(a));
		let b = app
			.world_mut()
			.spawn(bone_components(["root", "child", "b"], root))
			.insert(ChildOf(child))
			.id();
		app.world_mut()
			.spawn(bone_components(["root", "child", "b", "b child"], root))
			.insert(ChildOf(b));
		app.world_mut()
			.spawn(bone_components(["root", "child", "c"], root))
			.insert(ChildOf(child));
		app.world_mut()
			.spawn((Name::from("agent"), _Agent, AnimationDispatch::to([root])));

		app.update();

		assert_eq!(
			HashMap::from([
				(AnimationTargetId::from_name(&Name::from("root")), 1),
				(
					AnimationTargetId::from_names([Name::from("root"), Name::from("child")].iter()),
					1
				),
				(
					AnimationTargetId::from_names(
						[Name::from("root"), Name::from("child"), Name::from("c")].iter()
					),
					1
				),
			]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.expect("NO MATCHING GRAPH")
				.mask_groups
		);
	}

	#[test]
	fn act_only_once() {
		agent_animation_definitions!(&[_Mask {
			id: 1,
			def: || AffectedAnimationBones::Leaf {
				root: Name::from("root")
			},
		}]);
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
		agent_animation_definitions!(&[_Mask {
			id: 1,
			def: || AffectedAnimationBones::Leaf {
				root: Name::from("root")
			},
		}]);
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
