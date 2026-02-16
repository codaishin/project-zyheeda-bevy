use crate::{
	assets::agent_config::{AgentConfigAsset, AgentModel, Loadout},
	components::{
		agent::{AgentTransformDirty, ApplyAgentConfig},
		agent_config::AgentConfig,
	},
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	components::asset_model::AssetModel,
	tools::{action_key::slot::SlotKey, inventory_key::InventoryKey},
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		handles_animations::{Animations, RegisterAnimations},
		handles_loadout::{
			LoadoutKey,
			insert_default_loadout::{InsertDefaultLoadout, NotLoadedOut},
			register_loadout_bones::{NoBonesRegistered, RegisterLoadoutBones},
		},
		handles_physics::PhysicalDefaultAttributes,
		handles_skill_physics::{RegisterDefinition, SkillSpawnPoints},
		loadout::ItemName,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::{iter::Enumerate, slice::Iter};

impl ApplyAgentConfig {
	pub(crate) fn system<TLoadout, TSkills, TAnimations, TAttributes>(
		mut loadout_param: StaticSystemParam<TLoadout>,
		mut skills_param: StaticSystemParam<TSkills>,
		mut animations_param: StaticSystemParam<TAnimations>,
		mut commands: ZyheedaCommands,
		agents: Query<
			(
				Entity,
				&AgentConfig,
				&mut Transform,
				Option<&AgentTransformDirty>,
			),
			With<Self>,
		>,
		configs: Res<Assets<AgentConfigAsset>>,
	) where
		TLoadout: SystemParam
			+ for<'c> GetContextMut<NotLoadedOut, TContext<'c>: InsertDefaultLoadout>
			+ for<'c> GetContextMut<NoBonesRegistered, TContext<'c>: RegisterLoadoutBones>,
		TSkills:
			SystemParam + for<'c> GetContextMut<SkillSpawnPoints, TContext<'c>: RegisterDefinition>,
		TAnimations:
			SystemParam + for<'c> GetContextMut<Animations, TContext<'c>: RegisterAnimations>,
		TAttributes: Component + From<PhysicalDefaultAttributes>,
	{
		for (entity, AgentConfig { config_handle }, mut transform, transform_dirty) in agents {
			let Some(config) = configs.get(config_handle) else {
				continue;
			};

			let no_loadout = NotLoadedOut { entity };
			if let Some(mut ctx) = TLoadout::get_context_mut(&mut loadout_param, no_loadout) {
				ctx.insert_default_loadout(&config.loadout);
			};

			let no_loadout_bones = NoBonesRegistered { entity };
			if let Some(mut ctx) = TLoadout::get_context_mut(&mut loadout_param, no_loadout_bones) {
				ctx.register_loadout_bones(
					config.bones.forearm_slots.clone(),
					config.bones.hand_slots.clone(),
					config.bones.essence_slots.clone(),
				);
			};

			let skill_spawn_points = SkillSpawnPoints { entity };
			if let Some(mut ctx) = TSkills::get_context_mut(&mut skills_param, skill_spawn_points) {
				ctx.register_definition(config.bones.spawners.clone());
			};

			let animations = Animations { entity };
			if let Some(mut ctx) = TAnimations::get_context_mut(&mut animations_param, animations) {
				ctx.register_animations(&config.animations, &config.animation_mask_groups);
			}

			if transform_dirty.is_some() {
				transform.translation += config.ground_offset;
			}

			commands.try_apply_on(&entity, |mut e| {
				match &config.agent_model {
					AgentModel::Asset(path) => {
						e.try_insert(AssetModel::path(path));
					}
					AgentModel::Procedural(func) => {
						func(&mut e);
					}
				};
				e.try_insert(TAttributes::from(config.attributes));
				e.try_remove::<(Self, AgentTransformDirty)>();
			});
		}
	}
}

pub struct LoadoutIterator<'a> {
	inventory: Enumerate<Iter<'a, Option<ItemName>>>,
	slots: Iter<'a, (SlotKey, Option<ItemName>)>,
}

impl LoadoutIterator<'_> {
	fn next_inventory_item(&mut self) -> Option<(LoadoutKey, Option<ItemName>)> {
		self.inventory
			.next()
			.map(|(key, item)| (LoadoutKey::from(InventoryKey(key)), item.clone()))
	}

	fn next_slot_item(&mut self) -> Option<(LoadoutKey, Option<ItemName>)> {
		self.slots
			.next()
			.map(|(key, item)| (LoadoutKey::from(*key), item.clone()))
	}
}

impl Iterator for LoadoutIterator<'_> {
	type Item = (LoadoutKey, Option<ItemName>);

	fn next(&mut self) -> Option<Self::Item> {
		self.next_inventory_item().or_else(|| self.next_slot_item())
	}
}

impl<'a> IntoIterator for &'a Loadout {
	type Item = (LoadoutKey, Option<ItemName>);
	type IntoIter = LoadoutIterator<'a>;

	fn into_iter(self) -> LoadoutIterator<'a> {
		LoadoutIterator {
			inventory: self.inventory.iter().enumerate(),
			slots: self.slots.iter(),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		assets::agent_config::{Bones, Loadout},
		components::agent::AgentTransformDirty,
	};
	use common::{
		attributes::{effect_target::EffectTarget, health::Health},
		bit_mask_index,
		tools::{
			action_key::slot::SlotKey,
			bone_name::BoneName,
			inventory_key::InventoryKey,
			path::Path,
		},
		traits::{
			handles_animations::{
				AffectedAnimationBones,
				Animation,
				AnimationKey,
				AnimationMaskBits,
				AnimationPath,
				PlayMode,
			},
			handles_skill_physics::SkillSpawner,
		},
		zyheeda_commands::ZyheedaEntityCommands,
	};
	use macros::{NestedMocks, simple_mock};
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{IsChanged, Mock, NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Loadout {
		loadout: Vec<(LoadoutKey, Option<ItemName>)>,
		bones: Mock_Bones,
	}

	impl Default for _Loadout {
		fn default() -> Self {
			Self {
				loadout: vec![],
				bones: Mock_Bones::new_mock(|mock| {
					mock.expect_register_loadout_bones().return_const(());
				}),
			}
		}
	}

	impl InsertDefaultLoadout for _Loadout {
		fn insert_default_loadout<TItems>(&mut self, items: TItems)
		where
			TItems: IntoIterator<Item = (LoadoutKey, Option<ItemName>)>,
		{
			self.loadout = items.into_iter().collect()
		}
	}

	simple_mock! {
		_Bones {}
		impl RegisterLoadoutBones for _Bones {
			fn register_loadout_bones(
				&mut self,
				forearms: HashMap<BoneName, SlotKey>,
				hands: HashMap<BoneName, SlotKey>,
				essences: HashMap<BoneName, SlotKey>,
			) {
				self.register_loadout_bones(forearms, hands, essences);
			}
		}
	}

	#[automock]
	impl RegisterLoadoutBones for _Loadout {
		fn register_loadout_bones(
			&mut self,
			forearms: HashMap<BoneName, SlotKey>,
			hands: HashMap<BoneName, SlotKey>,
			essences: HashMap<BoneName, SlotKey>,
		) {
			self.bones.register_loadout_bones(forearms, hands, essences);
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Skills {
		mock: Mock_Skills,
	}

	impl Default for _Skills {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_register_definition().return_const(());
			})
		}
	}

	#[automock]
	impl RegisterDefinition for _Skills {
		fn register_definition(&mut self, definition: HashMap<BoneName, SkillSpawner>) {
			self.mock.register_definition(definition);
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Animations {
		mock: Mock_Animations,
	}

	impl Default for _Animations {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_register_animations().return_const(());
			})
		}
	}

	#[automock]
	impl RegisterAnimations for _Animations {
		fn register_animations(
			&mut self,
			animations: &HashMap<AnimationKey, Animation>,
			animation_mask_groups: &HashMap<AnimationMaskBits, AffectedAnimationBones>,
		) {
			self.mock
				.register_animations(animations, animation_mask_groups);
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Attributes(PhysicalDefaultAttributes);

	impl From<PhysicalDefaultAttributes> for _Attributes {
		fn from(attributes: PhysicalDefaultAttributes) -> Self {
			Self(attributes)
		}
	}

	fn setup<const N: usize>(configs: [(&Handle<AgentConfigAsset>, AgentConfigAsset); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut config_assets = Assets::default();

		for (id, config) in configs {
			_ = config_assets.insert(id, config);
		}

		app.insert_resource(config_assets);
		app.add_systems(
			Update,
			(
				ApplyAgentConfig::system::<
					Query<&mut _Loadout>,
					Query<&mut _Skills>,
					Query<&mut _Animations>,
					_Attributes,
				>,
				IsChanged::<_Loadout>::detect,
				IsChanged::<_Skills>::detect,
				IsChanged::<_Animations>::detect,
				IsChanged::<AssetModel>::detect,
				IsChanged::<_Attributes>::detect,
				IsChanged::<Transform>::detect,
			)
				.chain(),
		);

		app
	}

	mod default_loadout {
		use super::*;

		#[test]
		fn insert_default_loadout() {
			let config_handle = new_handle();
			let config = AgentConfigAsset {
				loadout: Loadout {
					inventory: vec![Some(ItemName::from("inventory.item"))],
					slots: vec![(SlotKey(42), Some(ItemName::from("slot.item")))],
				},
				..default()
			};
			let mut app = setup([(&config_handle, config)]);
			let entity = app
				.world_mut()
				.spawn((
					ApplyAgentConfig,
					Transform::default(),
					AgentConfig { config_handle },
					_Loadout::default(),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&vec![
					(
						LoadoutKey::Inventory(InventoryKey(0)),
						Some(ItemName::from("inventory.item"))
					),
					(
						LoadoutKey::Slot(SlotKey(42)),
						Some(ItemName::from("slot.item"))
					),
				]),
				app.world()
					.entity(entity)
					.get::<_Loadout>()
					.map(|l| &l.loadout),
			);
		}
	}

	mod loadout_bones {
		use super::*;

		#[test]
		fn register_bones() {
			let config_handle = new_handle();
			let config = AgentConfigAsset {
				bones: Bones {
					spawners: HashMap::from([]),
					forearm_slots: HashMap::from([(BoneName::from("a"), SlotKey(0))]),
					hand_slots: HashMap::from([(BoneName::from("b"), SlotKey(1))]),
					essence_slots: HashMap::from([(BoneName::from("c"), SlotKey(2))]),
				},
				..default()
			};
			let mut app = setup([(&config_handle, config)]);
			app.world_mut().spawn((
				ApplyAgentConfig,
				Transform::default(),
				AgentConfig { config_handle },
				_Loadout {
					bones: Mock_Bones::new_mock(|mock| {
						mock.expect_register_loadout_bones()
							.times(1)
							.with(
								eq(HashMap::from([(BoneName::from("a"), SlotKey(0))])),
								eq(HashMap::from([(BoneName::from("b"), SlotKey(1))])),
								eq(HashMap::from([(BoneName::from("c"), SlotKey(2))])),
							)
							.return_const(());
					}),
					..default()
				},
			));

			app.update();
		}
	}

	mod skill_spawn_points {
		use super::*;

		#[test]
		fn insert_spawners_definition() {
			let config_handle = new_handle();
			let asset = AgentConfigAsset {
				bones: Bones {
					spawners: HashMap::from([
						(BoneName::from("a"), SkillSpawner::Neutral),
						(BoneName::from("b"), SkillSpawner::Slot(SlotKey(42))),
					]),
					..default()
				},
				..default()
			};
			let mut app = setup([(&config_handle, asset)]);
			app.world_mut().spawn((
				ApplyAgentConfig,
				Transform::default(),
				AgentConfig { config_handle },
				_Skills::new().with_mock(|mock| {
					mock.expect_register_definition()
						.once()
						.with(eq(HashMap::from([
							(BoneName::from("a"), SkillSpawner::Neutral),
							(BoneName::from("b"), SkillSpawner::Slot(SlotKey(42))),
						])))
						.return_const(());
				}),
			));

			app.update();
		}
	}

	mod animations {
		use super::*;

		#[test]
		fn set_animations() {
			let animations = HashMap::from([(
				AnimationKey::Run,
				Animation {
					path: AnimationPath::Single(Path::from("my/path")),
					play_mode: PlayMode::Replay,
					mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(42)),
				},
			)]);
			let animation_mask_groups = HashMap::from([(
				AnimationMaskBits::zero().with_set(bit_mask_index!(4)),
				AffectedAnimationBones {
					from_root: BoneName::from("root"),
					..default()
				},
			)]);
			let config_handle = new_handle();
			let asset = AgentConfigAsset {
				animations: animations.clone(),
				animation_mask_groups: animation_mask_groups.clone(),
				..default()
			};
			let mut app = setup([(&config_handle, asset)]);
			app.world_mut().spawn((
				ApplyAgentConfig,
				Transform::default(),
				AgentConfig { config_handle },
				_Animations::new().with_mock(move |mock| {
					mock.expect_register_animations()
						.times(1)
						.with(eq(animations.clone()), eq(animation_mask_groups.clone()))
						.return_const(());
				}),
			));

			app.update();
		}
	}

	mod model {
		use super::*;

		#[test]
		fn insert_asset_model() {
			let config_handle = new_handle();
			let config = AgentConfigAsset {
				agent_model: AgentModel::from("my/path"),
				..default()
			};
			let mut app = setup([(&config_handle, config)]);
			let entity = app
				.world_mut()
				.spawn((
					ApplyAgentConfig,
					Transform::default(),
					AgentConfig { config_handle },
				))
				.id();

			app.update();

			assert_eq!(
				Some(&AssetModel::from("my/path")),
				app.world().entity(entity).get::<AssetModel>()
			);
		}

		#[derive(Component, Debug, PartialEq)]
		struct _Model;

		impl _Model {
			fn insert(e: &mut ZyheedaEntityCommands) {
				e.try_insert(Self);
			}
		}

		#[test]
		fn insert_procedural_model() {
			let config_handle = new_handle();
			let config = AgentConfigAsset {
				agent_model: AgentModel::Procedural(_Model::insert),
				..default()
			};
			let mut app = setup([(&config_handle, config)]);
			let entity = app
				.world_mut()
				.spawn((
					ApplyAgentConfig,
					Transform::default(),
					AgentConfig { config_handle },
				))
				.id();

			app.update();

			assert_eq!(Some(&_Model), app.world().entity(entity).get::<_Model>());
		}
	}
	mod apply_ground_offset {
		use super::*;

		#[test]
		fn update_transform() {
			let config_handle = new_handle();
			let config = AgentConfigAsset {
				ground_offset: Vec3::new(5., 6., 7.),
				..default()
			};
			let mut app = setup([(&config_handle, config)]);
			let entity = app
				.world_mut()
				.spawn((
					ApplyAgentConfig,
					AgentTransformDirty,
					Transform::from_xyz(1., 2., 3.),
					AgentConfig { config_handle },
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Transform::from_xyz(6., 8., 10.)),
				app.world().entity(entity).get::<Transform>()
			);
		}

		#[test]
		fn do_not_update_transform_when_agent_transform_not_dirty() {
			let config_handle = new_handle();
			let config = AgentConfigAsset {
				ground_offset: Vec3::new(5., 6., 7.),
				..default()
			};
			let mut app = setup([(&config_handle, config)]);
			let entity = app
				.world_mut()
				.spawn((
					ApplyAgentConfig,
					Transform::from_xyz(1., 2., 3.),
					AgentConfig { config_handle },
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Transform::from_xyz(1., 2., 3.)),
				app.world().entity(entity).get::<Transform>()
			);
		}

		#[test]
		fn remove_transform_dirty_marker() {
			let config_handle = new_handle();
			let config = AgentConfigAsset {
				ground_offset: Vec3::new(5., 6., 7.),
				..default()
			};
			let mut app = setup([(&config_handle, config)]);
			let entity = app
				.world_mut()
				.spawn((
					ApplyAgentConfig,
					AgentTransformDirty,
					Transform::from_xyz(1., 2., 3.),
					AgentConfig { config_handle },
				))
				.id();

			app.update();

			assert_eq!(
				None,
				app.world().entity(entity).get::<AgentTransformDirty>()
			);
		}
	}

	mod attributes {
		use super::*;

		#[test]
		fn insert_attributes() {
			let config_handle = new_handle();
			let attributes = PhysicalDefaultAttributes {
				health: Health::new(100.),
				force_interaction: EffectTarget::Immune,
				gravity_interaction: EffectTarget::Affected,
			};
			let mut app = setup([(
				&config_handle,
				AgentConfigAsset {
					attributes,
					..default()
				},
			)]);
			let entity = app
				.world_mut()
				.spawn((
					ApplyAgentConfig,
					Transform::default(),
					AgentConfig { config_handle },
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Attributes(attributes)),
				app.world().entity(entity).get::<_Attributes>(),
			);
		}
	}

	#[test]
	fn act_only_once() {
		let config_handle = new_handle();
		let mut app = setup([(
			&config_handle,
			AgentConfigAsset {
				agent_model: AgentModel::from("my/path"),
				..default()
			},
		)]);
		let entity = app
			.world_mut()
			.spawn((
				ApplyAgentConfig,
				Transform::default(),
				AgentTransformDirty,
				AgentConfig { config_handle },
				_Loadout::default(),
				_Skills::default(),
				_Animations::default(),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			(
				Some(&IsChanged::FALSE),
				Some(&IsChanged::FALSE),
				Some(&IsChanged::FALSE),
				Some(&IsChanged::FALSE),
				Some(&IsChanged::FALSE),
				Some(&IsChanged::FALSE),
			),
			(
				app.world().entity(entity).get::<IsChanged<_Loadout>>(),
				app.world().entity(entity).get::<IsChanged<_Skills>>(),
				app.world().entity(entity).get::<IsChanged<_Animations>>(),
				app.world().entity(entity).get::<IsChanged<AssetModel>>(),
				app.world().entity(entity).get::<IsChanged<_Attributes>>(),
				app.world().entity(entity).get::<IsChanged<Transform>>(),
			),
		);
	}

	#[test]
	fn act_if_config_inserted_later() {
		let config_handle = new_handle();
		let mut app = setup([]);
		let entity = app
			.world_mut()
			.spawn((
				ApplyAgentConfig,
				AgentTransformDirty,
				AgentConfig {
					config_handle: config_handle.clone(),
				},
				_Loadout::default(),
				_Skills::default(),
				_Animations::default(),
			))
			.id();

		app.update();
		let mut configs = app.world_mut().resource_mut::<Assets<AgentConfigAsset>>();
		_ = configs.insert(
			&config_handle,
			AgentConfigAsset {
				agent_model: AgentModel::from("my/path"),
				..default()
			},
		);
		app.update();

		assert_eq!(
			(
				Some(&IsChanged::TRUE),
				Some(&IsChanged::TRUE),
				Some(&IsChanged::TRUE),
				Some(&IsChanged::TRUE),
				Some(&IsChanged::TRUE),
				Some(&IsChanged::TRUE),
			),
			(
				app.world().entity(entity).get::<IsChanged<_Loadout>>(),
				app.world().entity(entity).get::<IsChanged<_Skills>>(),
				app.world().entity(entity).get::<IsChanged<_Animations>>(),
				app.world().entity(entity).get::<IsChanged<AssetModel>>(),
				app.world().entity(entity).get::<IsChanged<_Attributes>>(),
				app.world().entity(entity).get::<IsChanged<Transform>>(),
			),
		);
	}
}
