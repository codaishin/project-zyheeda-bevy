use crate::components::animation_lookup::{AnimationLookup2, AnimationLookupData};
use bevy::{
	animation::{AnimationTarget, AnimationTargetId},
	prelude::*,
};
use common::traits::{
	animation::BoneName,
	iter_descendants_conditional::IterDescendantsConditional,
};
use std::iter;

impl AnimationLookup2 {
	pub(crate) fn init_animation_bone_groups(
		mut graphs: ResMut<Assets<AnimationGraph>>,
		lookups: Query<(Entity, &Self, &AnimationGraphHandle)>,
		bones: Query<(&Name, &AnimationTarget)>,
		children: Query<&Children>,
	) {
		for (entity, lookup, AnimationGraphHandle(handle)) in &lookups {
			let Some(graph) = graphs.get_mut(handle) else {
				continue;
			};
			let chains =
				all_animation_bone_chains(entity, &children, &bones, lookup.animations.values());

			update_graph(graph, chains);
		}
	}
}

fn all_animation_bone_chains<'a>(
	entity: Entity,
	children: &Query<&Children>,
	animation_targets: &Query<(&Name, &AnimationTarget)>,
	animation_masks: impl Iterator<Item = &'a AnimationLookupData>,
) -> Vec<(AnimationTargetId, AnimationMask)> {
	let mut bones = vec![];
	let get_bone = |child| {
		let (name, target) = animation_targets.get(child).ok()?;
		if target.player != entity {
			return None;
		}
		Some((child, name, target))
	};

	for data in animation_masks {
		let animation_bones = animation_bone_chains(
			entity,
			&data.bones.from_root,
			&data.bones.until_exclusive,
			children,
			&get_bone,
		);

		let Some(animation_bones) = animation_bones else {
			continue;
		};

		for mask_bone in animation_bones {
			bones.push((mask_bone, data.mask));
		}
	}

	bones
}

fn update_graph(graph: &mut AnimationGraph, mask_bones: Vec<(AnimationTargetId, AnimationMask)>) {
	for (target, mask) in mask_bones {
		*graph.mask_groups.entry(target).or_default() |= mask;
	}
}

fn animation_bone_chains<'a>(
	player: Entity,
	mask_root: &BoneName,
	until_excluded: &[BoneName],
	children: &Query<'_, '_, &Children>,
	get_bone: &'a impl Fn(Entity) -> Option<(Entity, &'a Name, &'a AnimationTarget)>,
) -> Option<impl Iterator<Item = AnimationTargetId>> {
	let not_excluded = |e| {
		let Some((_, name, _)) = get_bone(e) else {
			return false;
		};
		!until_excluded.contains(&BoneName::from(name))
	};
	let (entity, _, target) = root_bone(player, mask_root, children, get_bone)?;
	let children = children
		.iter_descendants_conditional(entity, not_excluded)
		.filter_map(get_bone)
		.map(|(.., target)| target.id);

	Some(iter::once(target.id).chain(children))
}

fn root_bone<'a>(
	player: Entity,
	mask_root: &BoneName,
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
	use crate::components::animation_lookup::AnimationClips;
	use bevy::{animation::AnimationTargetId, platform::collections::HashMap as BevyHashMap};
	use common::traits::animation::{AffectedAnimationBones2, AnimationKey};
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, new_handle};

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

	fn setup(handle: &Handle<AnimationGraph>) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		assets.insert(handle, AnimationGraph::new());
		app.insert_resource(assets);
		app.add_systems(Update, AnimationLookup2::init_animation_bone_groups);

		app
	}

	#[test]
	fn set_when_root() {
		let handle = new_handle();
		let mut app = setup(&handle);
		let root = app
			.world_mut()
			.spawn((
				AnimationLookup2 {
					animations: HashMap::from([(
						AnimationKey::Run,
						AnimationLookupData::<AnimationClips> {
							mask: 1,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from("root"),
								..default()
							},
							..default()
						},
					)]),
				},
				AnimationGraphHandle(handle.clone()),
			))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));

		app.update();

		assert_eq!(
			BevyHashMap::from([(AnimationTargetId::from_name(&Name::from("root")), 1)]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.unwrap()
				.mask_groups
		);
	}

	#[test]
	fn set_when_root_multiple_masks() {
		let handle = new_handle();
		let mut app = setup(&handle);
		let root = app
			.world_mut()
			.spawn((
				AnimationLookup2 {
					animations: HashMap::from([
						(
							AnimationKey::Run,
							AnimationLookupData::<AnimationClips> {
								mask: 1,
								bones: AffectedAnimationBones2 {
									from_root: BoneName::from("root"),
									..default()
								},
								..default()
							},
						),
						(
							AnimationKey::Walk,
							AnimationLookupData::<AnimationClips> {
								mask: 2,
								bones: AffectedAnimationBones2 {
									from_root: BoneName::from("root"),
									..default()
								},
								..default()
							},
						),
					]),
				},
				AnimationGraphHandle(handle.clone()),
			))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));

		app.update();

		assert_eq!(
			BevyHashMap::from([(AnimationTargetId::from_name(&Name::from("root")), 3)]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.unwrap()
				.mask_groups
		);
	}

	#[test]
	fn set_chain() {
		let handle = new_handle();
		let mut app = setup(&handle);
		let root = app
			.world_mut()
			.spawn((
				AnimationLookup2 {
					animations: HashMap::from([(
						AnimationKey::Run,
						AnimationLookupData::<AnimationClips> {
							mask: 1,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from("mask root"),
								..default()
							},
							..default()
						},
					)]),
				},
				AnimationGraphHandle(handle.clone()),
			))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));
		app.world_mut()
			.spawn(bone_components(["root", "mask root"], root))
			.insert(ChildOf(root));

		app.update();

		assert_eq!(
			BevyHashMap::from([(
				AnimationTargetId::from_names([Name::from("root"), Name::from("mask root")].iter()),
				1
			)]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.unwrap()
				.mask_groups
		);
	}

	#[test]
	fn ignore_path_not_containing_mask_root() {
		let handle = new_handle();
		let mut app = setup(&handle);
		let root = app
			.world_mut()
			.spawn((
				AnimationLookup2 {
					animations: HashMap::from([(
						AnimationKey::Run,
						AnimationLookupData::<AnimationClips> {
							mask: 1,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from("mask root"),
								..default()
							},
							..default()
						},
					)]),
				},
				AnimationGraphHandle(handle.clone()),
			))
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

		app.update();

		assert_eq!(
			BevyHashMap::from([(
				AnimationTargetId::from_names([Name::from("root"), Name::from("mask root")].iter()),
				1
			)]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.unwrap()
				.mask_groups
		);
	}

	#[test]
	fn add_multiple_names_below_mask_root_when_single_chain() {
		let handle = new_handle();
		let mut app = setup(&handle);
		let root = app
			.world_mut()
			.spawn((
				AnimationLookup2 {
					animations: HashMap::from([(
						AnimationKey::Run,
						AnimationLookupData::<AnimationClips> {
							mask: 1,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from("mask root"),
								..default()
							},
							..default()
						},
					)]),
				},
				AnimationGraphHandle(handle.clone()),
			))
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

		app.update();

		assert_eq!(
			BevyHashMap::from([
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
				.unwrap()
				.mask_groups
		);
	}

	#[test]
	fn add_names_below_mask_root_when_not_single_chain() {
		let handle = new_handle();
		let mut app = setup(&handle);
		let root = app
			.world_mut()
			.spawn((
				AnimationLookup2 {
					animations: HashMap::from([(
						AnimationKey::Run,
						AnimationLookupData::<AnimationClips> {
							mask: 1,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from("mask root"),
								..default()
							},
							..default()
						},
					)]),
				},
				AnimationGraphHandle(handle.clone()),
			))
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

		app.update();

		assert_eq!(
			BevyHashMap::from([
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
				.unwrap()
				.mask_groups
		);
	}

	#[test]
	fn ignore_targets_not_belonging_to_root() {
		let handle = new_handle();
		let mut app = setup(&handle);
		let root = app
			.world_mut()
			.spawn((
				AnimationLookup2 {
					animations: HashMap::from([(
						AnimationKey::Run,
						AnimationLookupData::<AnimationClips> {
							mask: 1,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from("mask root"),
								..default()
							},
							..default()
						},
					)]),
				},
				AnimationGraphHandle(handle.clone()),
			))
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

		app.update();

		assert_eq!(
			BevyHashMap::from([(
				AnimationTargetId::from_names([Name::from("root"), Name::from("mask root")].iter()),
				1
			)]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.unwrap()
				.mask_groups
		);
	}

	#[test]
	fn set_exclusion_mask() {
		let handle = new_handle();
		let mut app = setup(&handle);
		let root = app
			.world_mut()
			.spawn((
				AnimationLookup2 {
					animations: HashMap::from([(
						AnimationKey::Run,
						AnimationLookupData::<AnimationClips> {
							mask: 1,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from("root"),
								until_exclusive: vec![BoneName::from("a"), BoneName::from("b")],
							},
							..default()
						},
					)]),
				},
				AnimationGraphHandle(handle.clone()),
			))
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

		app.update();

		assert_eq!(
			BevyHashMap::from([
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
				.unwrap()
				.mask_groups
		);
	}

	#[test]
	fn act_only_once() {
		let handle = new_handle();
		let mut app = setup(&handle);
		let root = app
			.world_mut()
			.spawn((
				AnimationLookup2 {
					animations: HashMap::from([(
						AnimationKey::Run,
						AnimationLookupData::<AnimationClips> {
							mask: 1,
							bones: AffectedAnimationBones2 {
								from_root: BoneName::from(" root"),
								..default()
							},
							..default()
						},
					)]),
				},
				AnimationGraphHandle(handle.clone()),
			))
			.id();
		app.world_mut()
			.entity_mut(root)
			.insert(bone_components(["root"], root));

		app.update();
		app.world_mut()
			.resource_mut::<Assets<AnimationGraph>>()
			.get_mut(&handle)
			.unwrap()
			.mask_groups
			.clear();
		app.update();

		assert_eq!(
			BevyHashMap::from([]),
			app.world()
				.resource::<Assets<AnimationGraph>>()
				.get(&handle)
				.unwrap()
				.mask_groups
		);
	}
}
