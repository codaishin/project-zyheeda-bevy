use crate::{
	components::{combos::Combos, slots::Slots},
	item::Item,
	skills::Skill,
};
use bevy::prelude::*;
use common::{
	tools::{
		change::Change,
		item_type::{CompatibleItems, ItemType},
		keys::slot::{Combo, SlotKey},
	},
	traits::{
		handles_combo_menu::{GetComboAbleSkills, GetCombosOrdered, NextKeys},
		iteration::IterFinite,
	},
};
use std::collections::{HashMap, HashSet, hash_map::Entry};

#[derive(Debug, PartialEq)]
pub struct ComboDescriptor {
	pub compatible_skills: HashMap<SlotKey, Vec<Skill>>,
	pub combos: Vec<Combo<Skill>>,
}

impl ComboDescriptor {
	#[allow(clippy::type_complexity)]
	pub(crate) fn describe_combos_for<TPlayer>(
		slots: Query<(Ref<Slots>, Ref<Combos>), With<TPlayer>>,
		items: Res<Assets<Item>>,
		skills: Res<Assets<Skill>>,
	) -> Change<ComboDescriptor>
	where
		TPlayer: Component,
	{
		let Ok((slots, combos)) = slots.get_single() else {
			return Change::None;
		};

		if !slots.is_changed() && !combos.is_changed() {
			return Change::None;
		}

		let combos = combos.combos_ordered();
		let slot_layout = slot_layout(&slots, &items, &skills);
		let mut compatible_skills = HashMap::<SlotKey, Vec<Skill>>::default();

		for (.., skill) in skills.iter() {
			let CompatibleItems(compatible_items) = &skill.compatible_items;

			for slot_key in SlotKey::iterator() {
				let Some((item_type, ..)) = slot_layout.get(&slot_key) else {
					continue;
				};
				if !compatible_items.contains(item_type) {
					continue;
				}

				match compatible_skills.entry(slot_key) {
					Entry::Occupied(mut entry) => {
						if entry.get().contains(skill) {
							continue;
						}
						entry.get_mut().push(skill.clone());
					}
					Entry::Vacant(entry) => {
						entry.insert(vec![skill.clone()]);
					}
				}
			}
		}

		Change::Some(ComboDescriptor {
			compatible_skills,
			combos,
		})
	}
}

fn slot_layout<'a>(
	slots: &'a Slots,
	items: &'a Assets<Item>,
	skills: &'a Assets<Skill>,
) -> HashMap<SlotKey, (ItemType, Option<&'a Skill>)> {
	slots
		.0
		.iter()
		.filter_map(|(key, handle)| {
			let handle = handle.as_ref()?;
			let item = items.get(handle)?;
			let skill = item.skill.as_ref().and_then(|handle| skills.get(handle));
			Some((*key, (item.item_type, skill)))
		})
		.collect::<HashMap<_, _>>()
}

impl GetComboAbleSkills<Skill> for ComboDescriptor {
	fn get_combo_able_skills(&self, key: &SlotKey) -> Vec<Skill> {
		self.compatible_skills.get(key).cloned().unwrap_or_default()
	}
}

impl NextKeys for ComboDescriptor {
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey> {
		let mut next_keys = HashSet::default();
		let target_len = combo_keys.len();

		for combo in self.combos.iter() {
			for (current_combo_keys, ..) in combo.iter().filter(matching_length(target_len + 1)) {
				if combo_keys != &current_combo_keys[..target_len] {
					continue;
				}

				next_keys.insert(current_combo_keys[target_len]);
			}
		}

		next_keys
	}
}

impl GetCombosOrdered<Skill> for ComboDescriptor {
	fn combos_ordered(&self) -> Vec<Combo<Skill>> {
		self.combos.clone()
	}
}

fn matching_length(target_len: usize) -> impl Fn(&&(Vec<SlotKey>, Skill)) -> bool {
	move |(combo_keys, ..)| combo_keys.len() == target_len
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::combo_node::ComboNode;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::{item_type::CompatibleItems, keys::slot::Side},
		traits::handles_localization::Token,
	};

	#[derive(Component)]
	struct _Player;

	fn setup<const N_SKILLS: usize, const N_SLOTS: usize>(
		skills: [Skill; N_SKILLS],
		slot_layout: [(SlotKey, ItemType); N_SLOTS],
		combos: Combos,
	) -> (App, Entity) {
		let mut app = App::new().single_threaded(Update);
		let mut item_assets = Assets::<Item>::default();
		let mut skill_assets = Assets::<Skill>::default();
		let mut slots = Slots::default();

		for skill in skills {
			skill_assets.add(skill);
		}

		for (key, item_type) in slot_layout {
			slots.0.insert(
				key,
				Some(item_assets.add(Item {
					item_type,
					..default()
				})),
			);
		}

		let player = app.world_mut().spawn((_Player, slots, combos)).id();
		app.insert_resource(item_assets);
		app.insert_resource(skill_assets);

		(app, player)
	}

	#[test]
	fn get_combo_able_unique_skills() -> Result<(), RunSystemError> {
		let (mut app, ..) = setup(
			[
				Skill {
					token: Token::from("compatible skill a"),
					compatible_items: CompatibleItems::from([ItemType::Pistol]),
					..default()
				},
				Skill {
					token: Token::from("compatible skill a"),
					compatible_items: CompatibleItems::from([ItemType::Pistol]),
					..default()
				},
				Skill {
					token: Token::from("compatible skill b"),
					compatible_items: CompatibleItems::from([ItemType::Pistol]),
					..default()
				},
				Skill {
					token: Token::from("incompatible skill"),
					compatible_items: CompatibleItems::from([ItemType::ForceEssence]),
					..default()
				},
			],
			[(SlotKey::TopHand(Side::Left), ItemType::Pistol)],
			Combos::default(),
		);

		let equipment = app
			.world_mut()
			.run_system_once(ComboDescriptor::describe_combos_for::<_Player>)?
			.expect("could not produce equipment descriptor");

		assert_eq!(
			vec![
				Skill {
					token: Token::from("compatible skill a"),
					compatible_items: CompatibleItems::from([ItemType::Pistol]),
					..default()
				},
				Skill {
					token: Token::from("compatible skill b"),
					compatible_items: CompatibleItems::from([ItemType::Pistol]),
					..default()
				},
			],
			equipment.get_combo_able_skills(&SlotKey::TopHand(Side::Left))
		);
		Ok(())
	}

	#[test]
	fn get_next() -> Result<(), RunSystemError> {
		let (mut app, ..) = setup(
			[],
			[],
			Combos::new(ComboNode::new([(
				SlotKey::BottomHand(Side::Right),
				(
					Skill::default(),
					ComboNode::new([(
						SlotKey::TopHand(Side::Left),
						(
							Skill::default(),
							ComboNode::new([
								(
									SlotKey::BottomHand(Side::Left),
									(Skill::default(), ComboNode::default()),
								),
								(
									SlotKey::BottomHand(Side::Right),
									(Skill::default(), ComboNode::default()),
								),
							]),
						),
					)]),
				),
			)])),
		);

		let equipment = app
			.world_mut()
			.run_system_once(ComboDescriptor::describe_combos_for::<_Player>)?
			.expect("could not produce equipment descriptor");

		assert_eq!(
			HashSet::from([
				SlotKey::BottomHand(Side::Left),
				SlotKey::BottomHand(Side::Right)
			]),
			equipment.next_keys(&[
				SlotKey::BottomHand(Side::Right),
				SlotKey::TopHand(Side::Left)
			])
		);
		Ok(())
	}

	#[test]
	fn get_combos() -> Result<(), RunSystemError> {
		let (mut app, ..) = setup(
			[],
			[],
			Combos::new(ComboNode::new([(
				SlotKey::BottomHand(Side::Right),
				(
					Skill {
						token: Token::from("a"),
						..default()
					},
					ComboNode::new([
						(
							SlotKey::BottomHand(Side::Left),
							(
								Skill {
									token: Token::from("b"),
									..default()
								},
								ComboNode::default(),
							),
						),
						(
							SlotKey::BottomHand(Side::Right),
							(
								Skill {
									token: Token::from("c"),
									..default()
								},
								ComboNode::default(),
							),
						),
					]),
				),
			)])),
		);

		let equipment = app
			.world_mut()
			.run_system_once(ComboDescriptor::describe_combos_for::<_Player>)?
			.expect("could not produce equipment descriptor");

		assert_eq!(
			vec![
				vec![
					(
						vec![SlotKey::BottomHand(Side::Right)],
						Skill {
							token: Token::from("a"),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
						],
						Skill {
							token: Token::from("b"),
							..default()
						}
					)
				],
				vec![
					(
						vec![SlotKey::BottomHand(Side::Right)],
						Skill {
							token: Token::from("a"),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						Skill {
							token: Token::from("c"),
							..default()
						}
					)
				],
			],
			equipment.combos_ordered()
		);
		Ok(())
	}

	#[derive(Resource, Default, Debug, PartialEq)]
	struct _Res(Change<ComboDescriptor>);

	impl _Res {
		fn update(In(e): In<Change<ComboDescriptor>>, mut r: ResMut<_Res>) {
			*r = _Res(e);
		}
	}

	#[test]
	fn return_none_when_neither_slots_nor_combos_changed() {
		let (mut app, ..) = setup([], [], Combos::default());
		app.init_resource::<_Res>();
		app.add_systems(
			Update,
			ComboDescriptor::describe_combos_for::<_Player>.pipe(_Res::update),
		);

		app.update();
		app.update();

		assert_eq!(&_Res(Change::None), app.world().resource::<_Res>());
	}

	#[test]
	fn return_some_when_slots_changed() {
		let (mut app, player) = setup([], [], Combos::default());
		app.init_resource::<_Res>();
		app.add_systems(
			Update,
			ComboDescriptor::describe_combos_for::<_Player>.pipe(_Res::update),
		);

		app.update();
		app.world_mut()
			.entity_mut(player)
			.get_mut::<Slots>()
			.as_deref_mut();
		app.update();

		assert!(app.world().resource::<_Res>().0.is_some());
	}

	#[test]
	fn return_some_when_combos_changed() {
		let (mut app, player) = setup([], [], Combos::default());
		app.init_resource::<_Res>();
		app.add_systems(
			Update,
			ComboDescriptor::describe_combos_for::<_Player>.pipe(_Res::update),
		);

		app.update();
		app.world_mut()
			.entity_mut(player)
			.get_mut::<Combos>()
			.as_deref_mut();
		app.update();

		assert!(app.world().resource::<_Res>().0.is_some());
	}
}
