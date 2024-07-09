use crate::traits::{CombosDescriptor, UpdateCombos};
use bevy::{
	asset::Handle,
	prelude::{Component, In, Query},
	render::texture::Image,
};
use common::tools::changed::Changed;

pub(crate) fn update_combos<TKey, TComboOverview: Component + UpdateCombos<TKey>>(
	combos: In<Changed<CombosDescriptor<TKey, Handle<Image>>>>,
	mut combo_overviews: Query<&mut TComboOverview>,
) {
	let Changed::Value(combos) = combos.0 else {
		return;
	};
	let Ok(mut combo_overview) = combo_overviews.get_single_mut() else {
		return;
	};

	combo_overview.update_combos(combos);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::SkillDescriptor;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		prelude::{IntoSystem, KeyCode, Resource},
		utils::Uuid,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use mockall::{automock, predicate::eq};

	#[derive(Component, Default, Debug)]
	struct _ComboOverview {
		mock: Mock_ComboOverview,
	}

	#[automock]
	impl UpdateCombos<KeyCode> for _ComboOverview {
		fn update_combos(&mut self, combos: CombosDescriptor<KeyCode, Handle<Image>>) {
			self.mock.update_combos(combos)
		}
	}

	#[derive(Resource)]
	struct _Combos(Changed<CombosDescriptor<KeyCode, Handle<Image>>>);

	fn setup(combos: Changed<CombosDescriptor<KeyCode, Handle<Image>>>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(move || combos.clone()).pipe(update_combos::<KeyCode, _ComboOverview>),
		);

		app
	}

	#[test]
	fn insert_combos_in_combo_list() {
		let combos = vec![
			vec![
				SkillDescriptor {
					name: "a1".to_owned(),
					key: KeyCode::KeyA,
					icon: Some(Handle::Weak(AssetId::Uuid {
						uuid: Uuid::new_v4(),
					})),
				},
				SkillDescriptor {
					name: "a2".to_owned(),
					key: KeyCode::KeyB,
					icon: Some(Handle::Weak(AssetId::Uuid {
						uuid: Uuid::new_v4(),
					})),
				},
			],
			vec![
				SkillDescriptor {
					name: "b1".to_owned(),
					key: KeyCode::KeyC,
					icon: Some(Handle::Weak(AssetId::Uuid {
						uuid: Uuid::new_v4(),
					})),
				},
				SkillDescriptor {
					name: "b2".to_owned(),
					key: KeyCode::KeyD,
					icon: Some(Handle::Weak(AssetId::Uuid {
						uuid: Uuid::new_v4(),
					})),
				},
			],
		];

		let mut app = setup(Changed::Value(combos.clone()));
		let mut combos_overview = _ComboOverview::default();
		combos_overview
			.mock
			.expect_update_combos()
			.times(1)
			.with(eq(combos))
			.return_const(());

		app.world.spawn(combos_overview);

		app.update();
	}

	#[test]
	fn do_nothing_if_combos_unchanged() {
		let mut app = setup(Changed::None);
		let mut combos_overview = _ComboOverview::default();
		combos_overview
			.mock
			.expect_update_combos()
			.never()
			.return_const(());

		app.world.spawn(combos_overview);

		app.update();
	}
}
