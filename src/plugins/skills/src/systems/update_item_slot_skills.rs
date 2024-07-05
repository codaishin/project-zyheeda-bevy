use crate::{
	components::{slots::Slots, Slot},
	items::SkillHandle,
	resources::SkillFolder,
	skills::Skill,
};
use bevy::{
	asset::{Asset, Assets},
	ecs::{system::Res, world::Mut},
	prelude::{DetectChanges, Query},
};
use common::traits::get_handle_from_path::GetHandelFromPath;

pub(crate) fn update_item_slot_skills<TFolder: GetHandelFromPath<Skill> + Asset>(
	skill_folder: Res<SkillFolder<TFolder>>,
	loaded_folders: Res<Assets<TFolder>>,
	mut slots_query: Query<&mut Slots>,
) {
	let Some(folder) = loaded_folders.get(skill_folder.0.clone()) else {
		return;
	};
	let force_update = loaded_folders.is_changed();

	for slots in slots_query.iter_mut().filter(is_changed_or(force_update)) {
		update_slots_skills(slots, folder);
	}
}

fn update_slots_skills<TFolder: GetHandelFromPath<Skill>>(mut slots: Mut<Slots>, folder: &TFolder) {
	for slot in slots.0.values_mut() {
		update_slot_skill(slot, folder);
	}
}

fn update_slot_skill<TFolder: GetHandelFromPath<Skill>>(slot: &mut Slot, folder: &TFolder) {
	let Some(item) = slot.item.as_mut() else {
		return;
	};
	let SkillHandle::Path(path) = &item.skill else {
		return;
	};
	let Some(handle) = folder.handle_from_path(path.clone()) else {
		return;
	};

	item.skill = SkillHandle::Handle(handle);
}

fn is_changed_or(force_update: bool) -> impl FnMut(&Mut<Slots>) -> bool {
	move |slots| slots.is_changed() || force_update
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Mounts, Slot},
		items::{slot_key::SlotKey, Item, SkillHandle},
	};
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		prelude::{default, Entity},
		reflect::TypePath,
		utils::Uuid,
	};
	use common::{
		components::Side,
		test_tools::utils::SingleThreadedApp,
		traits::load_asset::Path,
	};
	use mockall::{automock, predicate::eq};

	#[derive(Default, TypePath, Asset)]
	struct _Folder {
		mock: Mock_Folder,
	}

	#[automock]
	impl GetHandelFromPath<Skill> for _Folder {
		fn handle_from_path(&self, path: Path) -> Option<Handle<Skill>> {
			self.mock.handle_from_path(path)
		}
	}

	fn arbitrary_mounts() -> Mounts<Entity> {
		Mounts {
			hand: Entity::from_raw(100),
			forearm: Entity::from_raw(200),
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<SkillFolder<_Folder>>();
		app.init_resource::<Assets<_Folder>>();
		app.add_systems(Update, update_item_slot_skills::<_Folder>);

		app
	}

	fn set_folder(app: &mut App, folder: _Folder) {
		let folder = app.world.resource_mut::<Assets<_Folder>>().add(folder);
		app.world.insert_resource(SkillFolder(folder));
	}

	#[test]
	fn get_handle_from_item_skill_path() {
		let mut app = setup();

		let mut folder = _Folder::default();
		folder
			.mock
			.expect_handle_from_path()
			.times(1)
			.with(eq(Path::from("my/skill/path")))
			.return_const(None);
		set_folder(&mut app, folder);

		app.world.spawn(Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: arbitrary_mounts(),
					item: Some(Item {
						skill: SkillHandle::Path(Path::from("my/skill/path")),
						..default()
					}),
				},
			)]
			.into(),
		));

		app.update();
	}

	#[test]
	fn set_skill_handle() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});

		let mut app = setup();

		let mut folder = _Folder::default();
		folder
			.mock
			.expect_handle_from_path()
			.return_const(handle.clone());
		set_folder(&mut app, folder);

		let slots = app
			.world
			.spawn(Slots(
				[(
					SlotKey::Hand(Side::Main),
					Slot {
						mounts: arbitrary_mounts(),
						item: Some(Item {
							skill: SkillHandle::Path(Path::from("my/skill/path")),
							..default()
						}),
					},
				)]
				.into(),
			))
			.id();

		app.update();

		let slots = app.world.entity(slots);
		let slot = slots
			.get::<Slots>()
			.unwrap()
			.0
			.get(&SlotKey::Hand(Side::Main))
			.unwrap();
		let item = slot.item.as_ref().unwrap();

		assert_eq!(SkillHandle::Handle(handle), item.skill);
	}

	#[test]
	fn update_only_when_slots_changed() {
		let mut app = setup();

		let mut folder = _Folder::default();
		folder
			.mock
			.expect_handle_from_path()
			.times(2)
			.return_const(None);
		set_folder(&mut app, folder);

		let slots = app
			.world
			.spawn(Slots(
				[(
					SlotKey::Hand(Side::Main),
					Slot {
						mounts: arbitrary_mounts(),
						item: Some(Item {
							skill: SkillHandle::Path(Path::from("my/skill/path")),
							..default()
						}),
					},
				)]
				.into(),
			))
			.id();

		app.update();
		app.update();

		let mut slots = app.world.entity_mut(slots);
		slots.insert(Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: arbitrary_mounts(),
					item: Some(Item {
						skill: SkillHandle::Path(Path::from("my/other/skill/path")),
						..default()
					}),
				},
			)]
			.into(),
		));

		app.update();
	}

	#[test]
	fn also_update_when_skill_folder_changed() {
		let mut app = setup();

		let mut folder = _Folder::default();
		folder
			.mock
			.expect_handle_from_path()
			.times(2)
			.return_const(None);
		set_folder(&mut app, folder);

		app.world.spawn(Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: arbitrary_mounts(),
					item: Some(Item {
						skill: SkillHandle::Path(Path::from("my/skill/path")),
						..default()
					}),
				},
			)]
			.into(),
		));

		app.update();
		app.update();

		app.world
			.resource_mut::<Assets<_Folder>>()
			.add(_Folder::default());

		app.update();
	}
}
