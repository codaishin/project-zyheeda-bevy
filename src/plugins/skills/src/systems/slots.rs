use crate::{
	components::{slots::Slots, BoneName, Mounts, Slot, SlotBones},
	items::SlotKey,
};
use bevy::{
	prelude::{BuildChildren, Children, Commands, Entity, HierarchyQueryExt, Name, Query},
	scene::SceneBundle,
};
use common::traits::try_remove_from::TryRemoveFrom;

pub(crate) fn add_item_slots(
	mut commands: Commands,
	mut agent: Query<(Entity, &mut Slots, &mut SlotBones)>,
	children: Query<&Children>,
	bones: Query<&Name>,
) {
	for (agent, mut slots, mut slot_infos) in &mut agent {
		let add_slot = |slot_info: (SlotKey, Mounts<&'static BoneName>)| {
			let (key, mounts) = slot_info;
			match find_bones(agent, &mounts, &children, &bones) {
				Some((hand, forearm)) => {
					slots.0.insert(
						key,
						Slot {
							mounts: new_handles_on(hand, forearm, &mut commands),
							item: None,
						},
					);
					None
				}
				None => Some((key, mounts)),
			}
		};

		slot_infos.0 = slot_infos
			.0
			.clone()
			.into_iter()
			.filter_map(add_slot)
			.collect();
		if slot_infos.0.is_empty() {
			commands.try_remove_from::<SlotBones>(agent);
		}
	}
}

fn find_bones(
	agent: Entity,
	mounts: &Mounts<&'static BoneName>,
	children: &Query<&Children>,
	names: &Query<&Name>,
) -> Option<(Entity, Entity)> {
	let has_name = |mount_name| {
		move |entity| {
			names
				.get(entity)
				.ok()
				.map(|name| match mount_name == name.as_str() {
					true => Some(entity),
					false => None,
				})
		}
	};
	let hand = children
		.iter_descendants(agent)
		.filter_map(has_name(mounts.hand))
		.flatten()
		.next()?;
	let forearm = children
		.iter_descendants(agent)
		.filter_map(has_name(mounts.forearm))
		.flatten()
		.next()?;

	Some((hand, forearm))
}

fn new_handles_on(hand: Entity, forearm: Entity, commands: &mut Commands) -> Mounts<Entity> {
	let hand_scene = commands.spawn(SceneBundle::default()).id();
	let forearm_scene = commands.spawn(SceneBundle::default()).id();
	commands.entity(hand).push_children(&[hand_scene]);
	commands.entity(forearm).push_children(&[forearm_scene]);
	Mounts {
		hand: hand_scene,
		forearm: forearm_scene,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		prelude::{App, BuildWorldChildren, Handle, Name, Quat, Transform, Update, Vec3},
		scene::Scene,
	};
	use common::components::Side;
	use std::collections::HashMap;

	#[test]
	fn add_slot_as_child_of_bone() {
		let mut app = App::new();
		let hand_bone = app
			.world
			.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let forearm_bone = app
			.world
			.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		app.world
			.spawn((
				Slots::new(),
				SlotBones(
					[(
						SlotKey::Hand(Side::Off),
						Mounts {
							hand: "hand bone",
							forearm: "forearm bone",
						},
					)]
					.into(),
				),
			))
			.push_children(&[hand_bone, forearm_bone]);
		app.add_systems(Update, add_item_slots);

		app.update();

		let hand_bone = app.world.entity(hand_bone);
		let hand_bone_children_count = hand_bone.get::<Children>().map(|c| c.iter().len());
		let forearm_bone = app.world.entity(forearm_bone);
		let forearm_bone_children_count = forearm_bone.get::<Children>().map(|c| c.iter().len());

		assert_eq!(
			(Some(1), Some(1)),
			(hand_bone_children_count, forearm_bone_children_count)
		);
	}

	#[test]
	fn bone_child_has_scene() {
		let mut app = App::new();
		let hand_bone = app
			.world
			.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let forearm_bone = app
			.world
			.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		app.world
			.spawn((
				Slots::new(),
				SlotBones(
					[(
						SlotKey::Hand(Side::Off),
						Mounts {
							hand: "hand bone",
							forearm: "forearm bone",
						},
					)]
					.into(),
				),
			))
			.push_children(&[hand_bone, forearm_bone]);
		app.add_systems(Update, add_item_slots);

		app.update();

		let hand_bone = app.world.entity(hand_bone);
		let hand_slot = *hand_bone.get::<Children>().and_then(|c| c.first()).unwrap();
		let hand_slot = app.world.entity(hand_slot);
		let forearm_bone = app.world.entity(forearm_bone);
		let forearm_slot = *forearm_bone
			.get::<Children>()
			.and_then(|c| c.first())
			.unwrap();
		let forearm_slot = app.world.entity(forearm_slot);

		assert_eq!(
			(true, true),
			(
				hand_slot.contains::<Handle<Scene>>(),
				forearm_slot.contains::<Handle<Scene>>()
			)
		);
	}

	#[test]
	fn bone_child_has_rotation_zero() {
		let mut app = App::new();
		let rotation = Quat::from_axis_angle(Vec3::ONE, 1.);
		let hand_bone = app
			.world
			.spawn((Name::new("hand bone"), Transform::from_rotation(rotation)))
			.id();
		let forearm_bone = app
			.world
			.spawn((
				Name::new("forearm bone"),
				Transform::from_rotation(rotation),
			))
			.id();
		app.world
			.spawn((
				Slots::new(),
				SlotBones(
					[(
						SlotKey::Hand(Side::Off),
						Mounts {
							hand: "hand bone",
							forearm: "forearm bone",
						},
					)]
					.into(),
				),
			))
			.push_children(&[hand_bone, forearm_bone]);
		app.add_systems(Update, add_item_slots);

		app.update();

		let hand_bone = app.world.entity(hand_bone);
		let hand_slot = *hand_bone.get::<Children>().and_then(|c| c.first()).unwrap();
		let hand_slot_transform = app.world.entity(hand_slot).get::<Transform>().unwrap();
		let forearm_bone = app.world.entity(forearm_bone);
		let forearm_slot = *forearm_bone
			.get::<Children>()
			.and_then(|c| c.first())
			.unwrap();
		let forearm_slot_transform = app.world.entity(forearm_slot).get::<Transform>().unwrap();

		assert_eq!(
			(Quat::IDENTITY, Quat::IDENTITY),
			(
				hand_slot_transform.rotation,
				forearm_slot_transform.rotation
			)
		);
	}

	#[test]
	fn bone_child_has_slot_with_correct_key_and_entity() {
		let mut app = App::new();
		let hand_bone = app
			.world
			.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let forearm_bone = app
			.world
			.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let root = app
			.world
			.spawn((
				Slots::new(),
				SlotBones(
					[(
						SlotKey::Hand(Side::Off),
						Mounts {
							hand: "hand bone",
							forearm: "forearm bone",
						},
					)]
					.into(),
				),
			))
			.push_children(&[hand_bone, forearm_bone])
			.id();
		app.add_systems(Update, add_item_slots);

		app.update();

		let hand_bone = app.world.entity(hand_bone);
		let hand_slot = *hand_bone.get::<Children>().and_then(|c| c.first()).unwrap();
		let forearm_bone = app.world.entity(forearm_bone);
		let forearm_slot = *forearm_bone
			.get::<Children>()
			.and_then(|c| c.first())
			.unwrap();
		let slots = app.world.entity(root).get::<Slots>().unwrap();

		assert_eq!(
			HashMap::from([(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: Mounts {
						hand: hand_slot,
						forearm: forearm_slot
					},
					item: None,
				}
			)]),
			slots.0
		);
	}

	#[test]
	fn root_has_slot_infos_removed() {
		let mut app = App::new();
		let bones = [
			app.world
				.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
			app.world
				.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
		];
		let root = app
			.world
			.spawn((
				Slots::new(),
				SlotBones(
					[(
						SlotKey::Hand(Side::Off),
						Mounts {
							hand: "hand bone",
							forearm: "forearm bone",
						},
					)]
					.into(),
				),
			))
			.push_children(&bones)
			.id();
		app.add_systems(Update, add_item_slots);

		app.update();

		let root = app.world.entity(root);

		assert!(!root.contains::<SlotBones>());
	}

	#[test]
	fn do_not_remove_mismatched_slot_bones() {
		let mut app = App::new();
		let bones = [
			app.world
				.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
			app.world
				.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
		];
		let root = app
			.world
			.spawn((
				Slots::new(),
				SlotBones(
					[
						(
							SlotKey::Hand(Side::Off),
							Mounts {
								hand: "hand bone",
								forearm: "forearm bone",
							},
						),
						(
							SlotKey::Hand(Side::Off),
							Mounts {
								hand: "hand bone2",
								forearm: "forearm bone2",
							},
						),
					]
					.into(),
				),
			))
			.push_children(&bones)
			.id();
		app.add_systems(Update, add_item_slots);

		app.update();

		let slot_infos = app.world.entity(root).get::<SlotBones>();

		assert_eq!(
			Some(&SlotBones(
				[(
					SlotKey::Hand(Side::Off),
					Mounts {
						hand: "hand bone2",
						forearm: "forearm bone2",
					},
				),]
				.into()
			)),
			slot_infos
		);
	}

	#[test]
	fn do_not_remove_partly_mismatched_slot_bones() {
		let mut app = App::new();
		let bones = [
			app.world
				.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
			app.world
				.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
			app.world
				.spawn((Name::new("hand bone2"), Transform::from_xyz(0., 0., 0.)))
				.id(),
		];
		let root = app
			.world
			.spawn((
				Slots::new(),
				SlotBones(
					[
						(
							SlotKey::Hand(Side::Off),
							Mounts {
								hand: "hand bone",
								forearm: "forearm bone",
							},
						),
						(
							SlotKey::Hand(Side::Off),
							Mounts {
								hand: "hand bone2",
								forearm: "forearm bone2",
							},
						),
					]
					.into(),
				),
			))
			.push_children(&bones)
			.id();
		app.add_systems(Update, add_item_slots);

		app.update();

		let slot_infos = app.world.entity(root).get::<SlotBones>();

		assert_eq!(
			Some(&SlotBones(
				[(
					SlotKey::Hand(Side::Off),
					Mounts {
						hand: "hand bone2",
						forearm: "forearm bone2",
					},
				),]
				.into()
			)),
			slot_infos
		);
	}
}
