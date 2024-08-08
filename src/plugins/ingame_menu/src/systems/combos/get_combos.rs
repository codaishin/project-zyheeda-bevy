use crate::{
	components::skill_descriptor::{DropdownTrigger, SkillDescriptor},
	traits::CombosDescriptor,
};
use bevy::{
	ecs::world::Ref,
	prelude::{Component, Query, With},
};
use common::components::Player;
use skills::{
	items::slot_key::SlotKey,
	skills::Skill,
	traits::{Combo, GetCombosOrdered},
};

pub(crate) fn get_combos<TCombos: Component + GetCombosOrdered>(
	players: Query<Ref<TCombos>, With<Player>>,
) -> CombosDescriptor {
	let Ok(combos) = players.get_single() else {
		return vec![];
	};

	combos.combos_ordered().map(combo_descriptor).collect()
}

fn combo_descriptor(combo: Combo) -> Vec<SkillDescriptor<DropdownTrigger>> {
	combo
		.iter()
		.cloned()
		.map(skill_descriptor)
		.collect::<Vec<_>>()
}

fn skill_descriptor((key_path, skill): (Vec<SlotKey>, &Skill)) -> SkillDescriptor<DropdownTrigger> {
	SkillDescriptor::<DropdownTrigger>::new(skill.clone(), key_path.clone())
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::{Commands, In, IntoSystem, Resource},
		utils::default,
	};
	use common::{
		components::{Player, Side},
		test_tools::utils::SingleThreadedApp,
	};
	use skills::{skills::Skill, traits::Combo};

	#[derive(Debug, PartialEq, Clone)]
	enum _Key {
		Main,
		Off,
	}

	impl From<SlotKey> for _Key {
		fn from(value: SlotKey) -> Self {
			match value {
				SlotKey::Hand(Side::Main) => _Key::Main,
				SlotKey::Hand(Side::Off) => _Key::Off,
			}
		}
	}

	#[derive(Component, Default)]
	struct _Combos(Vec<Vec<(Vec<SlotKey>, Skill)>>);

	impl GetCombosOrdered for _Combos {
		fn combos_ordered(&self) -> impl Iterator<Item = Combo> {
			self.0.iter().map(|combo| {
				combo
					.iter()
					.map(|(key_path, skill)| (key_path.clone(), skill))
					.collect()
			})
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(CombosDescriptor);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			get_combos::<_Combos>.pipe(|combos: In<CombosDescriptor>, mut commands: Commands| {
				commands.insert_resource(_Result(combos.0))
			}),
		);

		app
	}

	#[test]
	fn return_skill_descriptor_arrays() {
		let mut app = setup();
		app.world_mut().spawn((
			Player,
			_Combos(vec![
				vec![
					(
						vec![SlotKey::Hand(Side::Main)],
						Skill {
							name: "a1".to_owned(),
							..default()
						},
					),
					(
						vec![SlotKey::Hand(Side::Off)],
						Skill {
							name: "a2".to_owned(),
							..default()
						},
					),
				],
				vec![
					(
						vec![SlotKey::Hand(Side::Off)],
						Skill {
							name: "b1".to_owned(),
							..default()
						},
					),
					(
						vec![SlotKey::Hand(Side::Main)],
						Skill {
							name: "b2".to_owned(),
							..default()
						},
					),
				],
			]),
		));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(vec![
				vec![
					SkillDescriptor::<DropdownTrigger>::new(
						Skill {
							name: "a1".to_owned(),
							..default()
						},
						vec![SlotKey::Hand(Side::Main)],
					),
					SkillDescriptor::<DropdownTrigger>::new(
						Skill {
							name: "a2".to_owned(),
							..default()
						},
						vec![SlotKey::Hand(Side::Off)],
					)
				],
				vec![
					SkillDescriptor::<DropdownTrigger>::new(
						Skill {
							name: "b1".to_owned(),
							..default()
						},
						vec![SlotKey::Hand(Side::Off)],
					),
					SkillDescriptor::<DropdownTrigger>::new(
						Skill {
							name: "b2".to_owned(),
							..default()
						},
						vec![SlotKey::Hand(Side::Main)],
					)
				]
			]),
			result,
		)
	}

	#[test]
	fn return_unchanged_when_player_component_missing() {
		let mut app = setup();
		app.world_mut().spawn(_Combos(vec![
			vec![
				(
					vec![SlotKey::Hand(Side::Main)],
					Skill {
						name: "a1".to_owned(),
						..default()
					},
				),
				(
					vec![SlotKey::Hand(Side::Off)],
					Skill {
						name: "a2".to_owned(),
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::Hand(Side::Off)],
					Skill {
						name: "b1".to_owned(),
						..default()
					},
				),
				(
					vec![SlotKey::Hand(Side::Main)],
					Skill {
						name: "b2".to_owned(),
						..default()
					},
				),
			],
		]));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(&_Result(vec![]), result)
	}
}
