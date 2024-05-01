use crate::components::{slots::Slots, Slot, SlotBones, SlotKey};
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
		let add_slot = |slot_info: (SlotKey, &'static str)| {
			let (key, bone_name) = slot_info;
			match find_bone(agent, bone_name, &children, &bones) {
				Some(bone) => {
					slots.0.insert(
						key,
						Slot {
							entity: new_slot_on(bone, &mut commands),
							item: None,
						},
					);
					None
				}
				None => Some((key, bone_name)),
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

fn find_bone(
	agent: Entity,
	bone_name: &str,
	children: &Query<&Children>,
	names: &Query<&Name>,
) -> Option<Entity> {
	children
		.iter_descendants(agent)
		.filter_map(|descendant| {
			names
				.get(descendant)
				.ok()
				.map(|name| match bone_name == name.as_str() {
					true => Some(descendant),
					false => None,
				})
		})
		.flatten()
		.next()
}

fn new_slot_on(parent: Entity, commands: &mut Commands) -> Entity {
	let slot = commands.spawn(SceneBundle::default()).id();
	commands.entity(parent).push_children(&[slot]);
	slot
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

	#[derive(PartialEq, Debug)]
	struct MockBehavior;

	#[test]
	fn add_slot_as_child_of_bone() {
		let mut app = App::new();
		let bone = app
			.world
			.spawn((Name::new("bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		app.world
			.spawn((
				Slots::new(),
				SlotBones([(SlotKey::Hand(Side::Off), "bone")].into()),
			))
			.push_children(&[bone]);
		app.add_systems(Update, add_item_slots);

		app.update();

		let bone = app.world.entity(bone);
		let bone_children_count = bone.get::<Children>().map(|c| c.iter().len());

		assert_eq!(Some(1), bone_children_count);
	}

	#[test]
	fn bone_child_has_scene() {
		let mut app = App::new();
		let bone = app
			.world
			.spawn((Name::new("bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		app.world
			.spawn((
				Slots::new(),
				SlotBones([(SlotKey::Hand(Side::Off), "bone")].into()),
			))
			.push_children(&[bone]);
		app.add_systems(Update, add_item_slots);

		app.update();

		let bone = app.world.entity(bone);
		let slot = *bone.get::<Children>().and_then(|c| c.first()).unwrap();
		let slot = app.world.entity(slot);

		assert!(slot.contains::<Handle<Scene>>());
	}

	#[test]
	fn bone_child_has_rotation_zero() {
		let mut app = App::new();
		let rotation = Quat::from_axis_angle(Vec3::ONE, 1.);
		let bone = app
			.world
			.spawn((Name::new("bone"), Transform::from_rotation(rotation)))
			.id();
		app.world
			.spawn((
				Slots::new(),
				SlotBones([(SlotKey::Hand(Side::Off), "bone")].into()),
			))
			.push_children(&[bone]);
		app.add_systems(Update, add_item_slots);

		app.update();

		let bone = app.world.entity(bone);
		let slot = *bone.get::<Children>().and_then(|c| c.first()).unwrap();
		let slot_transform = app.world.entity(slot).get::<Transform>().unwrap();

		assert_eq!(Quat::IDENTITY, slot_transform.rotation);
	}

	#[test]
	fn bone_child_has_slot_with_correct_key_and_entity() {
		let mut app = App::new();
		let bone = app
			.world
			.spawn((Name::new("bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let root = app
			.world
			.spawn((
				Slots::new(),
				SlotBones([(SlotKey::Hand(Side::Off), "bone")].into()),
			))
			.push_children(&[bone])
			.id();
		app.add_systems(Update, add_item_slots);

		app.update();

		let bone = app.world.entity(bone);
		let slot = *bone.get::<Children>().and_then(|c| c.first()).unwrap();
		let slots = app.world.entity(root).get::<Slots>().unwrap();

		assert_eq!(
			HashMap::from([(
				SlotKey::Hand(Side::Off),
				Slot {
					entity: slot,
					item: None,
				}
			)]),
			slots.0
		);
	}

	#[test]
	fn root_has_slot_infos_removed() {
		let mut app = App::new();
		let bone = app
			.world
			.spawn((Name::new("bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let root = app
			.world
			.spawn((
				Slots::new(),
				SlotBones([(SlotKey::Hand(Side::Off), "bone")].into()),
			))
			.push_children(&[bone])
			.id();
		app.add_systems(Update, add_item_slots);

		app.update();

		let root = app.world.entity(root);

		assert!(!root.contains::<SlotBones>());
	}

	#[test]
	fn partly_remove_slot_infos_when_not_all_matched() {
		let mut app = App::new();
		let bone = app
			.world
			.spawn((Name::new("bone"), Transform::from_xyz(0., 0., 0.)))
			.id();
		let root = app
			.world
			.spawn((
				Slots::new(),
				SlotBones(
					[
						(SlotKey::Hand(Side::Off), "bone"),
						(SlotKey::Hand(Side::Main), "bone2"),
					]
					.into(),
				),
			))
			.push_children(&[bone])
			.id();
		app.add_systems(Update, add_item_slots);

		app.update();

		let slot_infos = app.world.entity(root).get::<SlotBones>();

		assert_eq!(
			Some(&SlotBones([(SlotKey::Hand(Side::Main), "bone2")].into())),
			slot_infos
		);
	}
}
