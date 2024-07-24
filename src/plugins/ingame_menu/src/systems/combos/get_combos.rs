use crate::traits::{CombosDescriptor, SkillDescriptor};
use bevy::{
	asset::Handle,
	ecs::world::Ref,
	prelude::{Component, Image, Query, With},
};
use common::components::Player;
use skills::{
	items::slot_key::SlotKey,
	skills::Skill,
	traits::{Combo, GetCombos},
};

pub(crate) fn get_combos<TKey: From<SlotKey> + Clone, TCombos: Component + GetCombos>(
	players: Query<Ref<TCombos>, With<Player>>,
) -> CombosDescriptor<TKey, Handle<Image>> {
	let Ok(combos) = players.get_single() else {
		return vec![];
	};

	combos.combos().iter().map(combo_descriptor).collect()
}

fn combo_descriptor<TKey: From<SlotKey> + Clone>(
	combo: &Combo,
) -> Vec<SkillDescriptor<TKey, Handle<Image>>> {
	combo.iter().map(skill_descriptor).collect::<Vec<_>>()
}

fn skill_descriptor<TKey: From<SlotKey> + Clone>(
	(key, skill): &(SlotKey, &Skill),
) -> SkillDescriptor<TKey, Handle<Image>> {
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
		asset::{Asset, AssetId},
		prelude::{Commands, In, IntoSystem, Resource},
		utils::default,
	};
	use common::{
		components::{Player, Side},
		test_tools::utils::SingleThreadedApp,
	};
	use skills::{skills::Skill, traits::Combo};
	use uuid::Uuid;

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
	struct _Result(CombosDescriptor<_Key, Handle<Image>>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			get_combos::<_Key, _Combos>.pipe(
				|combos: In<CombosDescriptor<_Key, Handle<Image>>>, mut commands: Commands| {
					commands.insert_resource(_Result(combos.0))
				},
			),
		);

		app
	}

	fn get_handle<T: Asset>(name: &str) -> Handle<T> {
		match name {
			"a/1" => Handle::Weak(AssetId::Uuid {
				uuid: Uuid::from_u128(0x17afa6e0_f072_47ad_b604_9a29111a59fe),
			}),
			"a/2" => Handle::Weak(AssetId::Uuid {
				uuid: Uuid::from_u128(0x4f16f9ad_c998_4082_bebd_53864cb51e51),
			}),
			"b/1" => Handle::Weak(AssetId::Uuid {
				uuid: Uuid::from_u128(0x60fbff89_cb91_406a_bd78_5124b1bfbbc2),
			}),
			"b/2" => Handle::Weak(AssetId::Uuid {
				uuid: Uuid::from_u128(0xe8c2c4f5_0a4a_4fd0_99eb_a8098ec0b42b),
			}),
			_ => Handle::default(),
		}
	}

	#[test]
	fn return_skill_descriptor_arrays() {
		let mut app = setup();
		app.world_mut().spawn((
			Player,
			_Combos(vec![
				vec![
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "a1".to_owned(),
							icon: Some(get_handle("a/1")),
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Off),
						Skill {
							name: "a2".to_owned(),
							icon: Some(get_handle("a/2")),
							..default()
						},
					),
				],
				vec![
					(
						SlotKey::Hand(Side::Off),
						Skill {
							name: "b1".to_owned(),
							icon: Some(get_handle("b/1")),
							..default()
						},
					),
					(
						SlotKey::Hand(Side::Main),
						Skill {
							name: "b2".to_owned(),
							icon: Some(get_handle("b/2")),
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
						name: "a1".to_owned(),
						key: _Key::Main,
						icon: Some(get_handle("a/1")),
					},
					SkillDescriptor {
						name: "a2".to_owned(),
						key: _Key::Off,
						icon: Some(get_handle("a/2")),
					}
				],
				vec![
					SkillDescriptor {
						name: "b1".to_owned(),
						key: _Key::Off,
						icon: Some(get_handle("b/1")),
					},
					SkillDescriptor {
						name: "b2".to_owned(),
						key: _Key::Main,
						icon: Some(get_handle("b/2")),
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
					SlotKey::Hand(Side::Main),
					Skill {
						name: "a1".to_owned(),
						icon: Some(get_handle("a/1")),
						..default()
					},
				),
				(
					SlotKey::Hand(Side::Off),
					Skill {
						name: "a2".to_owned(),
						icon: Some(get_handle("a/2")),
						..default()
					},
				),
			],
			vec![
				(
					SlotKey::Hand(Side::Off),
					Skill {
						name: "b1".to_owned(),
						icon: Some(get_handle("b/1")),
						..default()
					},
				),
				(
					SlotKey::Hand(Side::Main),
					Skill {
						name: "b2".to_owned(),
						icon: Some(get_handle("b/2")),
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
