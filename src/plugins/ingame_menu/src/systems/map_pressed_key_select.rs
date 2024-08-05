use crate::components::key_select::{KeySelect, ReKeySkill};
use bevy::{prelude::Query, ui::Interaction};

type KeySelectReKey = KeySelect<ReKeySkill>;

pub(crate) fn map_pressed_key_select(
	key_selects: Query<(&KeySelectReKey, &Interaction)>,
) -> Option<KeySelectReKey> {
	let (pressed, ..) = key_selects.iter().find(pressed)?;

	Some(KeySelectReKey {
		extra: ReKeySkill {
			to: pressed.extra.to,
		},
		key_path: pressed.key_path.clone(),
		key_button: pressed.key_button,
	})
}

fn pressed<T>((.., interaction): &(&T, &Interaction)) -> bool {
	interaction == &&Interaction::Pressed
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::{Commands, Entity, In, IntoSystem, Resource},
		ui::Interaction,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use skills::items::slot_key::SlotKey;

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Option<KeySelectReKey>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			map_pressed_key_select.pipe(
				|mapped: In<Option<KeySelectReKey>>, mut commands: Commands| {
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
				extra: ReKeySkill {
					to: SlotKey::Hand(Side::Off),
				},
				key_button: Entity::from_raw(101),
				key_path: vec![
					SlotKey::Hand(Side::Main),
					SlotKey::Hand(Side::Off),
					SlotKey::Hand(Side::Main),
				],
			},
			Interaction::Pressed,
		));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(Some(KeySelect {
				extra: ReKeySkill {
					to: SlotKey::Hand(Side::Off)
				},
				key_button: Entity::from_raw(101),
				key_path: vec![
					SlotKey::Hand(Side::Main),
					SlotKey::Hand(Side::Off),
					SlotKey::Hand(Side::Main)
				]
			})),
			result
		);
	}

	#[test]
	fn return_none_when_hovered() {
		let mut app = setup();
		app.world_mut().spawn((
			KeySelect {
				extra: ReKeySkill {
					to: SlotKey::Hand(Side::Main),
				},
				key_button: Entity::from_raw(101),
				key_path: vec![
					SlotKey::Hand(Side::Main),
					SlotKey::Hand(Side::Off),
					SlotKey::Hand(Side::Main),
				],
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
				extra: ReKeySkill {
					to: SlotKey::Hand(Side::Main),
				},
				key_button: Entity::from_raw(101),
				key_path: vec![
					SlotKey::Hand(Side::Main),
					SlotKey::Hand(Side::Off),
					SlotKey::Hand(Side::Main),
				],
			},
			Interaction::None,
		));

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(&_Result(None), result);
	}
}
