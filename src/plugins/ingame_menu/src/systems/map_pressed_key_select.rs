use crate::components::key_select::{KeySelect, ReKeySkill};
use bevy::{
	prelude::{Query, Res, Resource},
	ui::Interaction,
};
use common::traits::map_value::TryMapBackwards;

type KeySelectReKey<TEquipmentKey> = KeySelect<ReKeySkill<TEquipmentKey>, TEquipmentKey>;

pub(crate) fn map_pressed_key_select<TVisualKey, TEquipmentKey, TMap>(
	key_selects: Query<(&KeySelectReKey<TVisualKey>, &Interaction)>,
	map: Res<TMap>,
) -> Option<KeySelectReKey<TEquipmentKey>>
where
	TVisualKey: Copy + Sync + Send + 'static,
	TMap: Resource + TryMapBackwards<TVisualKey, TEquipmentKey>,
{
	let (pressed, ..) = key_selects.iter().find(pressed)?;
	let to = map.try_map_backwards(pressed.extra.to)?;
	let key_path = pressed
		.key_path
		.iter()
		.filter_map(|key| map.try_map_backwards(*key))
		.collect::<Vec<_>>();

	match key_path.len() == pressed.key_path.len() {
		true => Some(KeySelectReKey {
			extra: ReKeySkill { to },
			key_path,
			key_button: pressed.key_button,
		}),
		false => None,
	}
}

fn pressed<T>((.., interaction): &(&T, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::{Commands, Entity, In, IntoSystem},
		ui::Interaction,
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Clone, Copy)]
	enum _VisualKey {
		A,
		B,
		Unmapped,
	}

	#[derive(Debug, PartialEq)]
	enum _EquipmentKey {
		A,
		B,
	}

	#[derive(Resource, Default)]
	struct _Map;

	impl TryMapBackwards<_VisualKey, _EquipmentKey> for _Map {
		fn try_map_backwards(&self, value: _VisualKey) -> Option<_EquipmentKey> {
			match value {
				_VisualKey::A => Some(_EquipmentKey::A),
				_VisualKey::B => Some(_EquipmentKey::B),
				_VisualKey::Unmapped => None,
			}
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Option<KeySelectReKey<_EquipmentKey>>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Map>();
		app.add_systems(
			Update,
			map_pressed_key_select::<_VisualKey, _EquipmentKey, _Map>.pipe(
				|mapped: In<Option<KeySelectReKey<_EquipmentKey>>>, mut commands: Commands| {
					commands.insert_resource(_Result(mapped.0));
				},
			),
		);

		app
	}

	#[test]
	fn return_pressed_mapped() {
		let mut app = setup();
		app.world_mut().spawn((
			KeySelect {
				extra: ReKeySkill { to: _VisualKey::B },
				key_button: Entity::from_raw(101),
				key_path: vec![_VisualKey::A, _VisualKey::B, _VisualKey::A],
			},
			Interaction::Pressed,
		));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(Some(KeySelect {
				extra: ReKeySkill {
					to: _EquipmentKey::B
				},
				key_button: Entity::from_raw(101),
				key_path: vec![_EquipmentKey::A, _EquipmentKey::B, _EquipmentKey::A]
			})),
			result
		);
	}

	#[test]
	fn return_none_when_re_key_to_unmapped() {
		let mut app = setup();
		app.world_mut().spawn((
			KeySelect {
				extra: ReKeySkill {
					to: _VisualKey::Unmapped,
				},
				key_button: Entity::from_raw(101),
				key_path: vec![_VisualKey::A, _VisualKey::B, _VisualKey::A],
			},
			Interaction::Pressed,
		));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(&_Result(None), result);
	}

	#[test]
	fn return_none_when_key_path_contains_unmapped() {
		let mut app = setup();
		app.world_mut().spawn((
			KeySelect {
				extra: ReKeySkill { to: _VisualKey::A },
				key_button: Entity::from_raw(101),
				key_path: vec![_VisualKey::A, _VisualKey::Unmapped, _VisualKey::A],
			},
			Interaction::Pressed,
		));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(&_Result(None), result);
	}

	#[test]
	fn return_none_when_hovered() {
		let mut app = setup();
		app.world_mut().spawn((
			KeySelect {
				extra: ReKeySkill { to: _VisualKey::A },
				key_button: Entity::from_raw(101),
				key_path: vec![_VisualKey::A, _VisualKey::B, _VisualKey::A],
			},
			Interaction::Hovered,
		));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(&_Result(None), result);
	}

	#[test]
	fn return_none_when_no_interaction() {
		let mut app = setup();
		app.world_mut().spawn((
			KeySelect {
				extra: ReKeySkill { to: _VisualKey::A },
				key_button: Entity::from_raw(101),
				key_path: vec![_VisualKey::A, _VisualKey::B, _VisualKey::A],
			},
			Interaction::None,
		));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(&_Result(None), result);
	}
}
