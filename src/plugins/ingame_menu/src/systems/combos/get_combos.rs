use crate::{tools::SkillDescriptor, traits::CombosDescriptor};
use bevy::{
	ecs::world::Ref,
	prelude::{Component, Query, With},
};
use common::components::Player;
use skills::{
	items::slot_key::SlotKey,
	skills::Skill,
	traits::{Combo, GetCombos},
};

pub(crate) fn get_combos<TKey: From<SlotKey> + Clone, TCombos: Component + GetCombos>(
	players: Query<Ref<TCombos>, With<Player>>,
) -> CombosDescriptor<TKey> {
	let Ok(combos) = players.get_single() else {
		return vec![];
	};

	combos.combos().iter().map(combo_descriptor).collect()
}

fn combo_descriptor<TKey: From<SlotKey> + Clone>(combo: &Combo) -> Vec<SkillDescriptor<TKey>> {
	combo
		.iter()
		.cloned()
		.map(skill_descriptor)
		.collect::<Vec<_>>()
}

fn skill_descriptor<TKey: From<SlotKey> + Clone>(
	(key_path, skill): (Vec<SlotKey>, &Skill),
) -> SkillDescriptor<TKey> {
	SkillDescriptor {
		skill: skill.clone(),
		key_path: key_path.iter().cloned().map(TKey::from).collect(),
	}
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

	impl GetCombos for _Combos {
		fn combos(&self) -> Vec<Combo> {
			self.0
				.iter()
				.map(|combo| {
					combo
						.iter()
						.map(|(key_path, skill)| (key_path.clone(), skill))
						.collect()
				})
				.collect()
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(CombosDescriptor<_Key>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			get_combos::<_Key, _Combos>.pipe(
				|combos: In<CombosDescriptor<_Key>>, mut commands: Commands| {
					commands.insert_resource(_Result(combos.0))
				},
			),
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
					SkillDescriptor {
						key_path: vec![_Key::Main],
						skill: Skill {
							name: "a1".to_owned(),
							..default()
						},
					},
					SkillDescriptor {
						key_path: vec![_Key::Off],
						skill: Skill {
							name: "a2".to_owned(),
							..default()
						}
					}
				],
				vec![
					SkillDescriptor {
						key_path: vec![_Key::Off],
						skill: Skill {
							name: "b1".to_owned(),
							..default()
						}
					},
					SkillDescriptor {
						key_path: vec![_Key::Main],
						skill: Skill {
							name: "b2".to_owned(),
							..default()
						}
					}
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
