use crate::traits::{CombosDescriptor, UpdateCombos};
use bevy::prelude::{Component, In, Query};

pub(crate) fn update_combos_view<TKey, TComboOverview: Component + UpdateCombos<TKey>>(
	combos: In<CombosDescriptor<TKey>>,
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
	use crate::tools::SkillDescriptor;
	use bevy::{
		app::{App, Update},
		prelude::{default, IntoSystem, KeyCode, Resource},
	};
	use common::test_tools::utils::SingleThreadedApp;
	use mockall::{automock, predicate::eq};
	use skills::skills::Skill;

	#[derive(Component, Default, Debug)]
	struct _ComboOverview {
		mock: Mock_ComboOverview,
	}

	#[automock]
	impl UpdateCombos<KeyCode> for _ComboOverview {
		fn update_combos(&mut self, combos: CombosDescriptor<KeyCode>) {
			self.mock.update_combos(combos)
		}
	}

	#[derive(Resource)]
	struct _Combos(CombosDescriptor<KeyCode>);

	fn setup(combos: CombosDescriptor<KeyCode>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(move || combos.clone()).pipe(update_combos_view::<KeyCode, _ComboOverview>),
		);

		app
	}

	#[test]
	fn insert_combos_in_combo_list() {
		let combos = vec![
			vec![
				SkillDescriptor {
					key_path: vec![KeyCode::KeyA],
					skill: Skill {
						name: "a1".to_owned(),
						..default()
					},
				},
				SkillDescriptor {
					key_path: vec![KeyCode::KeyA],
					skill: Skill {
						name: "a2".to_owned(),
						..default()
					},
				},
			],
			vec![
				SkillDescriptor {
					key_path: vec![KeyCode::KeyA],
					skill: Skill {
						name: "b1".to_owned(),
						..default()
					},
				},
				SkillDescriptor {
					key_path: vec![KeyCode::KeyA],
					skill: Skill {
						name: "b2".to_owned(),
						..default()
					},
				},
			],
		];

		let mut app = setup(combos.clone());
		let mut combos_overview = _ComboOverview::default();
		combos_overview
			.mock
			.expect_update_combos()
			.times(1)
			.with(eq(combos))
			.return_const(());

		app.world_mut().spawn(combos_overview);

		app.update();
	}
}
