use crate::traits::{CombosDescriptor, SkillDescriptor};
use bevy::{
	ecs::{system::In, world::Ref},
	prelude::{Component, DetectChanges, Query, With},
};
use common::{components::Player, tools::changed::Changed, traits::load_asset::Path};
use skills::{
	items::slot_key::SlotKey,
	skills::Skill,
	traits::{Combo, GetCombos},
};

pub(crate) fn get_combos<TKey: From<SlotKey> + Clone, TCombos: Component + GetCombos>(
	changed_override: In<bool>,
	players: Query<Ref<TCombos>, With<Player>>,
) -> Changed<CombosDescriptor<TKey, Path>> {
	let Ok(combos) = players.get_single() else {
		return Changed::None;
	};
	if !combos.is_changed() && !changed_override.0 {
		return Changed::None;
	}

	Changed::Value(combos.combos().iter().map(combo_descriptor).collect())
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
		name: skill.name.clone(),
		key: TKey::from(*key),
		icon: skill.icon.clone(),
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
	struct _Result(Changed<CombosDescriptor<_Key, Path>>);

	fn setup(changed_override: bool) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(move || changed_override)
				.pipe(get_combos::<_Key, _Combos>)
				.pipe(
					|combos: In<Changed<CombosDescriptor<_Key, Path>>>, mut commands: Commands| {
						commands.insert_resource(_Result(combos.0))
					},
				),
		);

		app
	}

	#[test]
	fn return_skill_descriptor_arrays() {
		let mut app = setup(false);
		app.world_mut().spawn((
			Player,
			_Combos(vec![
				vec![
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "a1".to_owned(),
							icon: Some(Path::from("a/1")),
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Off),
						Skill {
							name: "a2".to_owned(),
							icon: Some(Path::from("a/2")),
							..default()
						},
					),
				],
				vec![
					(
						SlotKey::Hand(Side::Off),
						Skill {
							name: "b1".to_owned(),
							icon: Some(Path::from("b/1")),
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "b2".to_owned(),
							icon: Some(Path::from("b/2")),
							..default()
						},
					),
				],
			]),
		));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(Changed::Value(vec![
				vec![
					SkillDescriptor {
						name: "a1".to_owned(),
						key: _Key::Main,
						icon: Some(Path::from("a/1")),
					},
					SkillDescriptor {
						name: "a2".to_owned(),
						key: _Key::Off,
						icon: Some(Path::from("a/2")),
					}
				],
				vec![
					SkillDescriptor {
						name: "b1".to_owned(),
						key: _Key::Off,
						icon: Some(Path::from("b/1")),
					},
					SkillDescriptor {
						name: "b2".to_owned(),
						key: _Key::Main,
						icon: Some(Path::from("b/2")),
					}
				]
			])),
			result,
		)
	}

	#[test]
	fn return_unchanged_when_player_component_missing() {
		let mut app = setup(false);
		app.world_mut().spawn(_Combos(vec![
			vec![
				(
					SlotKey::Hand(Side::Main),
					Skill {
						name: "a1".to_owned(),
						icon: Some(Path::from("a/1")),
						..default()
					},
				),
				(
					SlotKey::Hand(Side::Off),
					Skill {
						name: "a2".to_owned(),
						icon: Some(Path::from("a/2")),
						..default()
					},
				),
			],
			vec![
				(
					SlotKey::Hand(Side::Off),
					Skill {
						name: "b1".to_owned(),
						icon: Some(Path::from("b/1")),
						..default()
					},
				),
				(
					SlotKey::Hand(Side::Main),
					Skill {
						name: "b2".to_owned(),
						icon: Some(Path::from("b/2")),
						..default()
					},
				),
			],
		]));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(&_Result(Changed::None), result)
	}

	#[test]
	fn return_unchanged_when_combo_unchanged() {
		let mut app = setup(false);
		app.world_mut().spawn((
			Player,
			_Combos(vec![
				vec![
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "a1".to_owned(),
							icon: Some(Path::from("a/1")),
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Off),
						Skill {
							name: "a2".to_owned(),
							icon: Some(Path::from("a/2")),
							..default()
						},
					),
				],
				vec![
					(
						SlotKey::Hand(Side::Off),
						Skill {
							name: "b1".to_owned(),
							icon: Some(Path::from("b/1")),
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "b2".to_owned(),
							icon: Some(Path::from("b/2")),
							..default()
						},
					),
				],
			]),
		));

		app.update();
		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(&_Result(Changed::None), result)
	}

	#[test]
	fn return_changed_when_combo_unchanged_but_changed_override_true() {
		let mut app = setup(true);
		app.world_mut().spawn((
			Player,
			_Combos(vec![
				vec![
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "a1".to_owned(),
							icon: Some(Path::from("a/1")),
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Off),
						Skill {
							name: "a2".to_owned(),
							icon: Some(Path::from("a/2")),
							..default()
						},
					),
				],
				vec![
					(
						SlotKey::Hand(Side::Off),
						Skill {
							name: "b1".to_owned(),
							icon: Some(Path::from("b/1")),
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "b2".to_owned(),
							icon: Some(Path::from("b/2")),
							..default()
						},
					),
				],
			]),
		));

		app.update();
		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(Changed::Value(vec![
				vec![
					SkillDescriptor {
						name: "a1".to_owned(),
						key: _Key::Main,
						icon: Some(Path::from("a/1")),
					},
					SkillDescriptor {
						name: "a2".to_owned(),
						key: _Key::Off,
						icon: Some(Path::from("a/2")),
					}
				],
				vec![
					SkillDescriptor {
						name: "b1".to_owned(),
						key: _Key::Off,
						icon: Some(Path::from("b/1")),
					},
					SkillDescriptor {
						name: "b2".to_owned(),
						key: _Key::Main,
						icon: Some(Path::from("b/2")),
					}
				]
			])),
			result,
		);
	}
}
