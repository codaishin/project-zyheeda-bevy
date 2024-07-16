use crate::{
	components::{BoneName, Mounts, Slot, SlotContent, SlotDefinition, SlotsDefinition},
	items::slot_key::SlotKey,
};
use bevy::{
	prelude::{BuildChildren, Children, Commands, Entity, HierarchyQueryExt, Name, Query},
	scene::SceneBundle,
};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};
use std::collections::HashMap;

struct FailedToAdd(SlotDefinition);
struct Remaining(HashMap<SlotKey, SlotContent>);

pub(crate) fn init_slots(
	mut commands: Commands,
	mut agent: Query<(Entity, &mut SlotsDefinition)>,
	children: Query<&Children>,
	bones: Query<&Name>,
) {
	for (agent, mut slots) in &mut agent {
		let slots = slots.as_mut();
		let try_add_slot = |(key, (mounts, item))| {
			let Some((hand, forearm)) = find_bones(agent, &mounts, &children, &bones) else {
				return Err(FailedToAdd((key, (mounts, item))));
			};

			slots.slot_buffer.0.insert(
				key,
				Slot {
					mounts: new_handles_on(hand, forearm, &mut commands),
					item,
				},
			);

			Ok(())
		};

		let remaining = try_to_init_slots(&mut slots.definitions, try_add_slot);

		if remaining.0.is_empty() {
			commands.try_remove_from::<SlotsDefinition>(agent);
			commands.try_insert_on(agent, slots.slot_buffer.clone());
		} else {
			slots.definitions = remaining.0;
		}
	}
}

fn try_to_init_slots(
	slot_definition: &mut HashMap<SlotKey, SlotContent>,
	try_add_slot: impl FnMut(SlotDefinition) -> Result<(), FailedToAdd>,
) -> Remaining {
	Remaining(
		slot_definition
			.clone()
			.into_iter()
			.map(try_add_slot)
			.filter_map(not_added)
			.collect(),
	)
}

fn not_added(result: Result<(), FailedToAdd>) -> Option<SlotDefinition> {
	if let Err(FailedToAdd(info)) = result {
		Some(info)
	} else {
		None
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
	use crate::{
		components::slots::Slots,
		items::{slot_key::SlotKey, Item},
	};

	use super::*;
	use bevy::{
		prelude::{App, BuildWorldChildren, Handle, Name, Quat, Transform, Update, Vec3},
		scene::Scene,
		utils::default,
	};
	use common::{components::Side, traits::load_asset::Path};
	use std::collections::HashMap;

	#[test]
	fn add_slot_as_child_of_bone() {
		let mut app = App::new();
		let hand_bone = app
			.world_mut()
			.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let forearm_bone = app
			.world_mut()
			.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		app.world_mut()
			.spawn(SlotsDefinition::new([(
				SlotKey::Hand(Side::Off),
				(
					Mounts {
						hand: "hand bone",
						forearm: "forearm bone",
					},
					None,
				),
			)]))
			.push_children(&[hand_bone, forearm_bone]);
		app.add_systems(Update, init_slots);

		app.update();

		let hand_bone = app.world().entity(hand_bone);
		let hand_bone_children_count = hand_bone.get::<Children>().map(|c| c.iter().len());
		let forearm_bone = app.world().entity(forearm_bone);
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
			.world_mut()
			.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let forearm_bone = app
			.world_mut()
			.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		app.world_mut()
			.spawn(SlotsDefinition::new([(
				SlotKey::Hand(Side::Off),
				(
					Mounts {
						hand: "hand bone",
						forearm: "forearm bone",
					},
					None,
				),
			)]))
			.push_children(&[hand_bone, forearm_bone]);
		app.add_systems(Update, init_slots);

		app.update();

		let hand_bone = app.world().entity(hand_bone);
		let hand_slot = *hand_bone.get::<Children>().and_then(|c| c.first()).unwrap();
		let hand_slot = app.world().entity(hand_slot);
		let forearm_bone = app.world().entity(forearm_bone);
		let forearm_slot = *forearm_bone
			.get::<Children>()
			.and_then(|c| c.first())
			.unwrap();
		let forearm_slot = app.world().entity(forearm_slot);

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
			.world_mut()
			.spawn((Name::new("hand bone"), Transform::from_rotation(rotation)))
			.id();
		let forearm_bone = app
			.world_mut()
			.spawn((
				Name::new("forearm bone"),
				Transform::from_rotation(rotation),
			))
			.id();
		app.world_mut()
			.spawn(SlotsDefinition::new([(
				SlotKey::Hand(Side::Off),
				(
					Mounts {
						hand: "hand bone",
						forearm: "forearm bone",
					},
					None,
				),
			)]))
			.push_children(&[hand_bone, forearm_bone]);
		app.add_systems(Update, init_slots);

		app.update();

		let hand_bone = app.world().entity(hand_bone);
		let hand_slot = *hand_bone.get::<Children>().and_then(|c| c.first()).unwrap();
		let hand_slot_transform = app.world().entity(hand_slot).get::<Transform>().unwrap();
		let forearm_bone = app.world().entity(forearm_bone);
		let forearm_slot = *forearm_bone
			.get::<Children>()
			.and_then(|c| c.first())
			.unwrap();
		let forearm_slot_transform = app.world().entity(forearm_slot).get::<Transform>().unwrap();

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
			.world_mut()
			.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let forearm_bone = app
			.world_mut()
			.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let root = app
			.world_mut()
			.spawn(SlotsDefinition::new([(
				SlotKey::Hand(Side::Off),
				(
					Mounts {
						hand: "hand bone",
						forearm: "forearm bone",
					},
					None,
				),
			)]))
			.push_children(&[hand_bone, forearm_bone])
			.id();
		app.add_systems(Update, init_slots);

		app.update();

		let hand_bone = app.world().entity(hand_bone);
		let hand_slot = *hand_bone.get::<Children>().and_then(|c| c.first()).unwrap();
		let forearm_bone = app.world().entity(forearm_bone);
		let forearm_slot = *forearm_bone
			.get::<Children>()
			.and_then(|c| c.first())
			.unwrap();
		let slots = app.world().entity(root).get::<Slots<Path>>().unwrap();

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
			app.world_mut()
				.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
			app.world_mut()
				.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
		];
		let root = app
			.world_mut()
			.spawn(SlotsDefinition::new([(
				SlotKey::Hand(Side::Off),
				(
					Mounts {
						hand: "hand bone",
						forearm: "forearm bone",
					},
					None,
				),
			)]))
			.push_children(&bones)
			.id();
		app.add_systems(Update, init_slots);

		app.update();

		let root = app.world().entity(root);

		assert!(!root.contains::<SlotsDefinition>());
	}

	#[test]
	fn do_not_remove_mismatched_slot_bones() {
		let mut app = App::new();
		let bones = [
			app.world_mut()
				.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
			app.world_mut()
				.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
		];
		let root = app
			.world_mut()
			.spawn(SlotsDefinition::new([
				(
					SlotKey::Hand(Side::Off),
					(
						Mounts {
							hand: "hand bone",
							forearm: "forearm bone",
						},
						None,
					),
				),
				(
					SlotKey::Hand(Side::Off),
					(
						Mounts {
							hand: "hand bone2",
							forearm: "forearm bone2",
						},
						None,
					),
				),
			]))
			.push_children(&bones)
			.id();
		app.add_systems(Update, init_slots);

		app.update();

		let slot_infos = app.world().entity(root).get::<SlotsDefinition>();

		assert_eq!(
			Some(
				&[(
					SlotKey::Hand(Side::Off),
					(
						Mounts {
							hand: "hand bone2",
							forearm: "forearm bone2",
						},
						None
					)
				)]
				.into()
			),
			slot_infos.map(|s| &s.definitions)
		);
	}

	#[test]
	fn do_not_remove_partly_mismatched_slot_bones() {
		let mut app = App::new();
		let bones = [
			app.world_mut()
				.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
			app.world_mut()
				.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
				.id(),
			app.world_mut()
				.spawn((Name::new("hand bone2"), Transform::from_xyz(0., 0., 0.)))
				.id(),
		];
		let root = app
			.world_mut()
			.spawn(SlotsDefinition::new([
				(
					SlotKey::Hand(Side::Off),
					(
						Mounts {
							hand: "hand bone",
							forearm: "forearm bone",
						},
						None,
					),
				),
				(
					SlotKey::Hand(Side::Off),
					(
						Mounts {
							hand: "hand bone2",
							forearm: "forearm bone2",
						},
						None,
					),
				),
			]))
			.push_children(&bones)
			.id();
		app.add_systems(Update, init_slots);

		app.update();

		let slot_infos = app.world().entity(root).get::<SlotsDefinition>();

		assert_eq!(
			Some(
				&[(
					SlotKey::Hand(Side::Off),
					(
						Mounts {
							hand: "hand bone2",
							forearm: "forearm bone2",
						},
						None
					)
				)]
				.into()
			),
			slot_infos.map(|s| &s.definitions)
		);
	}

	#[test]
	fn assign_item() {
		let mut app = App::new();
		let hand_bone = app
			.world_mut()
			.spawn((Name::new("hand bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let forearm_bone = app
			.world_mut()
			.spawn((Name::new("forearm bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let root = app
			.world_mut()
			.spawn(SlotsDefinition::new([(
				SlotKey::Hand(Side::Off),
				(
					Mounts {
						hand: "hand bone",
						forearm: "forearm bone",
					},
					Some(Item {
						name: "my item",
						..default()
					}),
				),
			)]))
			.push_children(&[hand_bone, forearm_bone])
			.id();
		app.add_systems(Update, init_slots);

		app.update();

		let hand_bone = app.world().entity(hand_bone);
		let hand_slot = *hand_bone.get::<Children>().and_then(|c| c.first()).unwrap();
		let forearm_bone = app.world().entity(forearm_bone);
		let forearm_slot = *forearm_bone
			.get::<Children>()
			.and_then(|c| c.first())
			.unwrap();
		let slots = app.world().entity(root).get::<Slots<Path>>().unwrap();

		assert_eq!(
			HashMap::from([(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: Mounts {
						hand: hand_slot,
						forearm: forearm_slot
					},
					item: Some(Item {
						name: "my item",
						..default()
					}),
				}
			)]),
			slots.0
		);
	}
}
