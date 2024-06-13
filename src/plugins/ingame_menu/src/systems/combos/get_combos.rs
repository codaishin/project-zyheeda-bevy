use crate::traits::SkillDescriptor;
use bevy::prelude::{Component, Query, With};
use common::{components::Player, traits::load_asset::Path};
use skills::{
	items::slot_key::SlotKey,
	skills::Skill,
	traits::{Combo, GetCombos},
};

pub(crate) type CombosDescriptor<TKey, TIcon> = Vec<Vec<SkillDescriptor<TKey, TIcon>>>;

pub(crate) fn get_combos<TKey: From<SlotKey> + Clone, TCombos: Component + GetCombos>(
	players: Query<&TCombos, With<Player>>,
) -> CombosDescriptor<TKey, Path> {
	let Ok(combos) = players.get_single() else {
		return vec![];
	};

	combos.combos().iter().map(combo_descriptor).collect()
}

fn combo_descriptor<TKey: From<SlotKey> + Clone>(
	combo: &Combo,
) -> Vec<SkillDescriptor<TKey, Path>> {
	combo.iter().map(skill_descriptor).collect::<Vec<_>>()
}

fn skill_descriptor<TKey: From<SlotKey> + Clone>(
	(key, skill): &(SlotKey, &Skill),
) -> SkillDescriptor<TKey, Path> {
	SkillDescriptor {
		name: skill.name,
		key: TKey::from(*key),
		icon: skill.icon.map(|icon| icon()),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::SkillDescriptor;
	use bevy::{
		app::{App, Update},
		prelude::{Commands, In, IntoSystem, Resource},
		utils::default,
	};
	use common::{
		components::{Player, Side},
		test_tools::utils::SingleThreadedApp,
		traits::load_asset::Path,
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
	struct _Combos(Vec<Vec<(SlotKey, Skill)>>);

	impl GetCombos for _Combos {
		fn combos(&self) -> Vec<Combo> {
			self.0
				.iter()
				.map(|combo| combo.iter().map(|(key, skill)| (*key, skill)).collect())
				.collect()
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Vec<Vec<SkillDescriptor<_Key, Path>>>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			get_combos::<_Key, _Combos>.pipe(
				|combos: In<Vec<Vec<SkillDescriptor<_Key, Path>>>>, mut commands: Commands| {
					commands.insert_resource(_Result(combos.0))
				},
			),
		);

		app
	}

	#[test]
	fn return_skill_descriptor_arrays() {
		let mut app = setup();
		app.world.spawn((
			Player,
			_Combos(vec![
				vec![
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "a1",
							icon: Some(|| Path::from("a/1")),
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Off),
						Skill {
							name: "a2",
							icon: Some(|| Path::from("a/2")),
							..default()
						},
					),
				],
				vec![
					(
						SlotKey::Hand(Side::Off),
						Skill {
							name: "b1",
							icon: Some(|| Path::from("b/1")),
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "b2",
							icon: Some(|| Path::from("b/2")),
							..default()
						},
					),
				],
			]),
		));

		app.update();

		let result = app.world.resource::<_Result>();

		assert_eq!(
			&_Result(vec![
				vec![
					SkillDescriptor {
						name: "a1",
						key: _Key::Main,
						icon: Some(Path::from("a/1")),
					},
					SkillDescriptor {
						name: "a2",
						key: _Key::Off,
						icon: Some(Path::from("a/2")),
					}
				],
				vec![
					SkillDescriptor {
						name: "b1",
						key: _Key::Off,
						icon: Some(Path::from("b/1")),
					},
					SkillDescriptor {
						name: "b2",
						key: _Key::Main,
						icon: Some(Path::from("b/2")),
					}
				]
			]),
			result,
		)
	}

	#[test]
	fn return_empty_when_player_component_missing() {
		let mut app = setup();
		app.world.spawn((_Combos(vec![
			vec![
				(
					SlotKey::Hand(Side::Main),
					Skill {
						name: "a1",
						icon: Some(|| Path::from("a/1")),
						..default()
					},
				),
				(
					SlotKey::Hand(Side::Off),
					Skill {
						name: "a2",
						icon: Some(|| Path::from("a/2")),
						..default()
					},
				),
			],
			vec![
				(
					SlotKey::Hand(Side::Off),
					Skill {
						name: "b1",
						icon: Some(|| Path::from("b/1")),
						..default()
					},
				),
				(
					SlotKey::Hand(Side::Main),
					Skill {
						name: "b2",
						icon: Some(|| Path::from("b/2")),
						..default()
					},
				),
			],
		]),));

		app.update();

		let result = app.world.resource::<_Result>();

		assert_eq!(&_Result(vec![]), result)
	}
}
