use crate::{
	components::bone_definitions::BoneDefinitions,
	system_parameters::loadout::LoadoutPrep,
};
use bevy::prelude::*;
use common::{
	tools::{action_key::slot::SlotKey, bone_name::BoneName},
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_loadout::register_loadout_bones::{NoBonesRegistered, RegisterLoadoutBones},
		load_asset::LoadAsset,
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use std::collections::HashMap;

impl<TAssetServer> GetContextMut<NoBonesRegistered> for LoadoutPrep<'_, '_, TAssetServer>
where
	TAssetServer: Resource + LoadAsset,
{
	type TContext<'ctx> = PrepareLoadoutBones<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut LoadoutPrep<TAssetServer>,
		NoBonesRegistered { entity }: NoBonesRegistered,
	) -> Option<Self::TContext<'ctx>> {
		if param.bone_definitions.contains(entity) {
			return None;
		}

		let entity = param.commands.get_mut(&entity)?;

		Some(PrepareLoadoutBones { entity })
	}
}

pub struct PrepareLoadoutBones<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}

impl RegisterLoadoutBones for PrepareLoadoutBones<'_> {
	fn register_loadout_bones(
		&mut self,
		forearms: HashMap<BoneName, SlotKey>,
		hands: HashMap<BoneName, SlotKey>,
		essences: HashMap<BoneName, SlotKey>,
	) {
		self.entity.try_insert(BoneDefinitions {
			forearms,
			hands,
			essences,
		});
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::bone_definitions::BoneDefinitions;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::load_asset::mock::MockAssetServer;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<MockAssetServer>();

		app
	}

	#[test]
	fn insert_bone_definition() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: LoadoutPrep<MockAssetServer>| {
				let key = NoBonesRegistered { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut p, key).unwrap();
				ctx.register_loadout_bones(
					HashMap::from([(BoneName::from("a"), SlotKey(0))]),
					HashMap::from([(BoneName::from("b"), SlotKey(1))]),
					HashMap::from([(BoneName::from("c"), SlotKey(2))]),
				);
			})?;

		assert_eq!(
			Some(&BoneDefinitions {
				forearms: HashMap::from([(BoneName::from("a"), SlotKey(0))]),
				hands: HashMap::from([(BoneName::from("b"), SlotKey(1))]),
				essences: HashMap::from([(BoneName::from("c"), SlotKey(2))]),
			}),
			app.world().entity(entity).get::<BoneDefinitions>()
		);
		Ok(())
	}

	#[test]
	fn context_is_none_when_bones_registered() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(BoneDefinitions::default()).id();

		let ctx_is_none =
			app.world_mut()
				.run_system_once(move |mut p: LoadoutPrep<MockAssetServer>| {
					LoadoutPrep::get_context_mut(&mut p, NoBonesRegistered { entity }).is_none()
				})?;

		assert!(ctx_is_none);
		Ok(())
	}
}
