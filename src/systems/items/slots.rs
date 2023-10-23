use std::borrow::Cow;

use crate::components::{SlotInfos, SlotKey, Slots};
use bevy::{
	prelude::{
		BuildChildren,
		Children,
		Commands,
		Entity,
		HierarchyQueryExt,
		Name,
		Query,
		Transform,
	},
	scene::SceneBundle,
	utils::default,
};

fn find_bone(
	agent: Entity,
	bone_name: &str,
	children: &Query<&Children>,
	names: &Query<(&Name, &Transform)>,
) -> Option<(Entity, Transform)> {
	children
		.iter_descendants(agent)
		.filter_map(|descendant| {
			names.get(descendant).ok().map(|(name, transform)| {
				if bone_name == name.as_str() {
					Some((descendant, *transform))
				} else {
					None
				}
			})
		})
		.flatten()
		.next()
}

fn new_slot_on(parent: (Entity, Transform), commands: &mut Commands) -> Entity {
	let (parent_entity, parent_transform) = parent;
	let slot = commands
		.spawn(SceneBundle {
			transform: Transform::from_rotation(parent_transform.rotation),
			..default()
		})
		.id();
	commands.entity(parent_entity).push_children(&[slot]);
	slot
}

pub fn add_slots(
	mut commands: Commands,
	mut agent: Query<(Entity, &mut Slots, &mut SlotInfos)>,
	children: Query<&Children>,
	bones: Query<(&Name, &Transform)>,
) {
	for (agent, mut slots, mut slot_infos) in &mut agent {
		let add_slot = |slot_info: (SlotKey, Cow<'static, str>)| {
			let (key, bone_name) = slot_info;
			match find_bone(agent, &bone_name, &children, &bones) {
				Some(bone) => {
					let entity = new_slot_on(bone, &mut commands);
					slots.0.insert(key, entity);
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
			commands.entity(agent).remove::<SlotInfos>();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Side;
	use bevy::{
		prelude::{App, BuildWorldChildren, Handle, Name, Quat, Transform, Update, Vec3},
		scene::Scene,
		utils::HashMap,
	};

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
				SlotInfos::new([(SlotKey::Hand(Side::Left), "bone")]),
			))
			.push_children(&[bone]);
		app.add_systems(Update, add_slots);

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
				SlotInfos::new([(SlotKey::Hand(Side::Left), "bone")]),
			))
			.push_children(&[bone]);
		app.add_systems(Update, add_slots);

		app.update();

		let bone = app.world.entity(bone);
		let slot = *bone.get::<Children>().and_then(|c| c.get(0)).unwrap();
		let slot = app.world.entity(slot);

		assert!(slot.contains::<Handle<Scene>>());
	}

	#[test]
	fn bone_child_has_same_rotation_as_its_parent() {
		let mut app = App::new();
		let rotation = Quat::from_axis_angle(Vec3::ONE, 1.);
		let bone = app
			.world
			.spawn((Name::new("bone"), Transform::from_rotation(rotation)))
			.id();
		app.world
			.spawn((
				Slots::new(),
				SlotInfos::new([(SlotKey::Hand(Side::Left), "bone")]),
			))
			.push_children(&[bone]);
		app.add_systems(Update, add_slots);

		app.update();

		let bone = app.world.entity(bone);
		let slot = *bone.get::<Children>().and_then(|c| c.get(0)).unwrap();
		let slot_transform = app.world.entity(slot).get::<Transform>().unwrap();

		assert_eq!(rotation, slot_transform.rotation);
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
				SlotInfos::new([(SlotKey::Hand(Side::Left), "bone")]),
			))
			.push_children(&[bone])
			.id();
		app.add_systems(Update, add_slots);

		app.update();

		let bone = app.world.entity(bone);
		let slot = *bone.get::<Children>().and_then(|c| c.get(0)).unwrap();
		let slots = app.world.entity(root).get::<Slots>().unwrap();

		assert_eq!(HashMap::from([(SlotKey::Hand(Side::Left), slot)]), slots.0);
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
				SlotInfos::new([(SlotKey::Hand(Side::Left), "bone")]),
			))
			.push_children(&[bone])
			.id();
		app.add_systems(Update, add_slots);

		app.update();

		let root = app.world.entity(root);

		assert!(!root.contains::<SlotInfos>());
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
				SlotInfos::new([
					(SlotKey::Hand(Side::Left), "bone"),
					(SlotKey::Hand(Side::Right), "bone2"),
				]),
			))
			.push_children(&[bone])
			.id();
		app.add_systems(Update, add_slots);

		app.update();

		let slot_infos = app.world.entity(root).get::<SlotInfos>();

		assert_eq!(
			Some(&SlotInfos::new([(SlotKey::Hand(Side::Right), "bone2")])),
			slot_infos
		);
	}
}
