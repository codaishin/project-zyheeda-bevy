use crate::traits::{CombosDescriptor, UpdateCombos};
use bevy::prelude::{Component, In, Query};

pub(crate) fn update_combos_view<TComboOverview: Component + UpdateCombos>(
	combos: In<CombosDescriptor>,
	mut combo_overviews: Query<&mut TComboOverview>,
) {
	let combos = combos.0;

	let Ok(mut combo_overview) = combo_overviews.get_single_mut() else {
		return;
	};

	combo_overview.update_combos(combos);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::skill_descriptor::SkillDescriptor;
	use bevy::{
		app::{App, Update},
		prelude::{default, IntoSystem, Resource},
	};
	use common::{
		components::Side,
		test_tools::utils::SingleThreadedApp,
		traits::nested_mock::NestedMock,
	};
	use macros::NestedMock;
	use mockall::{automock, predicate::eq};
	use skills::{items::slot_key::SlotKey, skills::Skill};

	#[derive(Component, NestedMock, Debug)]
	struct _ComboOverview {
		mock: Mock_ComboOverview,
	}

	#[automock]
	impl UpdateCombos for _ComboOverview {
		fn update_combos(&mut self, combos: CombosDescriptor) {
			self.mock.update_combos(combos)
		}
	}

	#[derive(Resource)]
	struct _Combos(CombosDescriptor);

	fn setup(combos: CombosDescriptor) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(move || combos.clone()).pipe(update_combos_view::<_ComboOverview>),
		);

		app
	}

	fn combos() -> CombosDescriptor {
		vec![
			vec![
				SkillDescriptor::new_dropdown_item(
					Skill {
						name: "a1".to_owned(),
						..default()
					},
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
				),
				SkillDescriptor::new_dropdown_item(
					Skill {
						name: "a2".to_owned(),
						..default()
					},
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
				),
			],
			vec![
				SkillDescriptor::new_dropdown_item(
					Skill {
						name: "b1".to_owned(),
						..default()
					},
					vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
				),
				SkillDescriptor::new_dropdown_item(
					Skill {
						name: "b2".to_owned(),
						..default()
					},
					vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Off)],
				),
			],
		]
	}

	#[test]
	fn insert_combos_in_combo_list() {
		let mut app = setup(combos());
		app.world_mut().spawn(_ComboOverview::new_mock(|mock| {
			mock.expect_update_combos()
				.times(1)
				.with(eq(combos()))
				.return_const(());
		}));

		app.update();
	}
}
