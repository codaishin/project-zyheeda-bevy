use crate::{
	folder_asset_loader::LoadResult,
	resources::SkillFolder,
	skills::Skill,
	states::SkillAssets,
};
use bevy::{
	asset::{AssetEvent, Assets, LoadedFolder},
	prelude::{EventReader, Local, NextState, Res, ResMut, State},
};

pub(crate) fn set_skill_assets_to_loaded(
	current_state: Res<State<SkillAssets>>,
	mut next_state: ResMut<NextState<SkillAssets>>,
	mut folder_events: EventReader<AssetEvent<LoadedFolder>>,
	skill_folder: Res<SkillFolder>,
	skill_results: Res<Assets<LoadResult<Skill>>>,
	mut folder_loaded: Local<bool>,
) {
	let folder_events = consume_events(&mut folder_events);

	if current_state.get() == &SkillAssets::Loaded {
		return;
	}

	if folder_events.into_iter().any(is_loaded(skill_folder)) {
		*folder_loaded = true;
	}

	if !*folder_loaded {
		return;
	}

	if !skill_results.is_empty() {
		return;
	}

	next_state.set(SkillAssets::Loaded);
}

fn consume_events<'a>(
	folder_events: &'a mut EventReader<AssetEvent<LoadedFolder>>,
) -> Vec<&'a AssetEvent<LoadedFolder>> {
	folder_events.read().collect()
}

fn is_loaded(skill_folder: Res<SkillFolder>) -> impl Fn(&AssetEvent<LoadedFolder>) -> bool + '_ {
	move |event| {
		let AssetEvent::LoadedWithDependencies { id } = event else {
			return false;
		};

		id == &skill_folder.0.id()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		folder_asset_loader::LoadResult,
		resources::SkillFolder,
		skills::Skill,
		states::SkillAssets,
	};
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetEvent, AssetId, Assets, Handle, LoadedFolder},
		prelude::{AppExtStates, State},
		state::app::StatesPlugin,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use uuid::Uuid;

	fn new_handle<TAsset: Asset>() -> Handle<TAsset> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	fn setup(skill_folder: Handle<LoadedFolder>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_plugins(StatesPlugin);
		app.insert_state(SkillAssets::Loading);
		app.insert_resource(SkillFolder(skill_folder));
		app.add_event::<AssetEvent<LoadedFolder>>();
		app.init_resource::<Assets<LoadResult<Skill>>>();

		app.add_systems(Update, set_skill_assets_to_loaded);

		app
	}

	#[test]
	fn set_to_loaded_when_load_results_empty_and_skill_folder_loaded() {
		let skill_folder = new_handle();
		let mut app = setup(skill_folder.clone());

		app.world_mut()
			.send_event(AssetEvent::LoadedWithDependencies {
				id: skill_folder.id(),
			});
		app.update();
		app.update();

		assert_eq!(
			&SkillAssets::Loaded,
			app.world().resource::<State<SkillAssets>>().get()
		);
	}

	#[test]
	fn do_not_set_to_loaded_when_skill_folder_not_loaded() {
		let skill_folder = new_handle();
		let mut app = setup(skill_folder.clone());

		app.update();
		app.update();

		assert_eq!(
			&SkillAssets::Loading,
			app.world().resource::<State<SkillAssets>>().get()
		);
	}

	#[test]
	fn do_not_set_to_loaded_when_unrelated_folder_loaded() {
		let skill_folder = new_handle();
		let unrelated_folder = new_handle::<LoadedFolder>();
		let mut app = setup(skill_folder.clone());

		app.world_mut()
			.send_event(AssetEvent::LoadedWithDependencies {
				id: unrelated_folder.id(),
			});
		app.update();
		app.update();

		assert_eq!(
			&SkillAssets::Loading,
			app.world().resource::<State<SkillAssets>>().get()
		);
	}

	#[test]
	fn do_not_set_to_loaded_when_load_results_not_empty_and_skill_folder_loaded() {
		let skill_folder = new_handle();
		let mut app = setup(skill_folder.clone());

		app.world_mut()
			.send_event(AssetEvent::LoadedWithDependencies {
				id: skill_folder.id(),
			});
		app.world_mut()
			.resource_mut::<Assets<LoadResult<Skill>>>()
			.add(LoadResult::Ok(Skill::default()));
		app.update();
		app.update();

		assert_eq!(
			&SkillAssets::Loading,
			app.world().resource::<State<SkillAssets>>().get()
		);
	}

	#[test]
	fn set_to_loaded_when_load_results_empty_after_skill_folder_loaded() {
		let skill_folder = new_handle();
		let mut app = setup(skill_folder.clone());

		app.world_mut()
			.send_event(AssetEvent::LoadedWithDependencies {
				id: skill_folder.id(),
			});
		app.world_mut()
			.resource_mut::<Assets<LoadResult<Skill>>>()
			.add(LoadResult::Ok(Skill::default()));
		app.update();
		app.update();

		app.world_mut()
			.insert_resource(Assets::<LoadResult<Skill>>::default());
		app.update();
		app.update();

		assert_eq!(
			&SkillAssets::Loaded,
			app.world().resource::<State<SkillAssets>>().get()
		);
	}

	#[test]
	fn do_not_set_to_loaded_multiple_times() {
		let skill_folder = new_handle();
		let mut app = setup(skill_folder.clone());

		app.world_mut()
			.send_event(AssetEvent::LoadedWithDependencies {
				id: skill_folder.id(),
			});
		app.update();
		app.update();
		app.update();

		assert!(match app.world().resource::<NextState<SkillAssets>>() {
			NextState::Unchanged => true,
			NextState::Pending(_) => false,
		});
	}
}
