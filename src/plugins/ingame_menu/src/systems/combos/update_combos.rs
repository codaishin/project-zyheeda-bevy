use crate::traits::{CombosDescriptor, UpdateCombos};
use bevy::{
	asset::Handle,
	prelude::{Component, In, Query},
	render::texture::Image,
};

pub(crate) fn update_combos<TKey, TComboOverview: Component + UpdateCombos<TKey>>(
	combos: In<CombosDescriptor<TKey, Handle<Image>>>,
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
	use crate::traits::SkillDescriptor;
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Handle},
		prelude::{IntoSystem, KeyCode, Resource},
	};
	use common::test_tools::utils::SingleThreadedApp;
	use mockall::{automock, predicate::eq};
	use uuid::Uuid;

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
	struct _Combos(CombosDescriptor<KeyCode, Handle<Image>>);

	fn setup(combos: CombosDescriptor<KeyCode, Handle<Image>>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(move || combos.clone()).pipe(update_combos::<KeyCode, _ComboOverview>),
		);

		app
	}

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	#[test]
	fn insert_combos_in_combo_list() {
		let combos = vec![
			vec![
				SkillDescriptor {
					name: "a1".to_owned(),
					key: KeyCode::KeyA,
					icon: Some(new_handle()),
				},
				SkillDescriptor {
					name: "a2".to_owned(),
					key: KeyCode::KeyB,
					icon: Some(new_handle()),
				},
			],
			vec![
				SkillDescriptor {
					name: "b1".to_owned(),
					key: KeyCode::KeyC,
					icon: Some(new_handle()),
				},
				SkillDescriptor {
					name: "b2".to_owned(),
					key: KeyCode::KeyD,
					icon: Some(new_handle()),
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
