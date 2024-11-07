pub mod components;
pub mod definitions;
pub mod inventory_key;
pub mod item;
pub mod resources;
pub mod skills;
pub mod slot_key;
pub mod systems;
pub mod traits;

mod behaviors;
mod bundles;

use animations::{animation::Animation, components::animation_dispatch::AnimationDispatch};
use bevy::{
	color::palettes::{
		css::LIGHT_CYAN,
		tailwind::{CYAN_100, CYAN_200},
	},
	prelude::*,
};
use bundles::{ComboBundle, Loadout};
use common::{
	components::{AssetModel, Collection, Side, Swap},
	resources::key_map::KeyMap,
	states::MouseContext,
	systems::{log::log_many, track_components::TrackComponentInSelfAndChildren},
	traits::try_insert_on::TryInsertOn,
};
use components::{
	combos::Combos,
	combos_time_out::CombosTimeOut,
	inventory::Inventory,
	queue::Queue,
	renderer::{EssenceRender, ModelRender, Renderer},
	skill_executer::SkillExecuter,
	skill_spawners::SkillSpawners,
	slots::Slots,
};
use definitions::{
	item_slots::{ForearmSlots, HandSlots},
	sub_models::SubModels,
};
use inventory_key::InventoryKey;
use item::{item_type::SkillItemType, SkillItem, SkillItemContent};
use items::RegisterItemView;
use loading::traits::register_custom_folder_assets::RegisterCustomFolderAssets;
use macros::skill_asset;
use player::components::player::Player;
use shaders::materials::essence_material::EssenceMaterial;
use skills::{skill_data::SkillData, QueuedSkill, RunSkillBehavior, Skill};
use slot_key::SlotKey;
use std::time::Duration;
use systems::{
	advance_active_skill::advance_active_skill,
	enqueue::enqueue,
	equip::equip_item,
	execute::ExecuteSkills,
	flush::flush,
	get_inputs::get_inputs,
	mouse_context::{
		advance::{advance_just_released_mouse_context, advance_just_triggered_mouse_context},
		release::release_triggered_mouse_context,
		trigger_primed::trigger_primed_mouse_context,
	},
	update_skill_combos::update_skill_combos,
	visualize_slot_items::visualize_slot_items,
};

pub struct SkillsPlugin<TState> {
	pub play: TState,
}

impl<TState> SkillsPlugin<TState>
where
	TState: States + Copy,
{
	fn skill_load(&self, app: &mut App) {
		app.register_custom_folder_assets::<Skill, SkillData>();
	}

	fn skill_slot_load(&self, app: &mut App) {
		app.add_systems(
			PreUpdate,
			SkillSpawners::track_in_self_and_children::<Name>().system(),
		)
		.add_systems(Update, Self::set_player_items)
		.register_item_view_for::<Player, HandSlots<Player>>()
		.register_item_view_for::<Player, ForearmSlots<Player>>()
		.register_item_view_for::<Player, SubModels<Player>>()
		.add_systems(
			Update,
			(
				visualize_slot_items::<HandSlots<Player>>,
				visualize_slot_items::<ForearmSlots<Player>>,
				visualize_slot_items::<SubModels<Player>>,
			),
		)
		.add_systems(
			Update,
			(
				equip_item::<Inventory, InventoryKey, Collection<Swap<InventoryKey, SlotKey>>>
					.pipe(log_many),
				equip_item::<Inventory, InventoryKey, Collection<Swap<SlotKey, InventoryKey>>>
					.pipe(log_many),
			),
		);
	}

	fn skill_execution(&self, app: &mut App) {
		app.init_resource::<KeyMap<SlotKey, KeyCode>>()
			.add_systems(
				Update,
				(
					get_inputs::<
						KeyMap<SlotKey, KeyCode>,
						ButtonInput<KeyCode>,
						State<MouseContext<KeyCode>>,
					>
						.pipe(enqueue::<Slots, Queue, QueuedSkill>),
					update_skill_combos::<Combos, CombosTimeOut, Queue, Virtual>,
					advance_active_skill::<
						Queue,
						Animation,
						AnimationDispatch,
						SkillExecuter,
						Virtual,
					>,
					SkillExecuter::<RunSkillBehavior>::execute_system.pipe(log_many),
					flush::<Queue>,
				)
					.chain()
					.run_if(in_state(self.play)),
			)
			.add_systems(
				Update,
				(
					trigger_primed_mouse_context,
					advance_just_triggered_mouse_context,
					release_triggered_mouse_context,
					advance_just_released_mouse_context,
				),
			);
	}

	fn set_player_items(
		mut commands: Commands,
		players: Query<Entity, Added<Player>>,
		asset_server: Res<AssetServer>,
	) {
		let Ok(player) = players.get_single() else {
			return;
		};
		let asset_server = asset_server.as_ref();

		commands.try_insert_on(
			player,
			(
				Self::get_inventory(asset_server),
				Self::get_loadout(asset_server),
				Self::get_combos(),
			),
		);
	}

	fn get_loadout(asset_server: &AssetServer) -> Loadout {
		let force_essence_material = EssenceMaterial {
			texture_color: CYAN_100.into(),
			fill_color: CYAN_200.into(),
			fresnel_color: (LIGHT_CYAN * 1.5).into(),
			..default()
		};

		Loadout::new([
			(
				SlotKey::TopHand(Side::Left),
				Some(SkillItem {
					name: "Plasma Pistol A",
					content: SkillItemContent {
						render: Renderer {
							model: ModelRender::Hand(AssetModel::Path("models/pistol.glb")),
							essence: EssenceRender::StandardMaterial,
						},
						skill: Some(asset_server.load(skill_asset!("shoot_hand_gun"))),
						item_type: SkillItemType::Pistol,
					},
				}),
			),
			(
				SlotKey::BottomHand(Side::Left),
				Some(SkillItem {
					name: "Plasma Pistol B",
					content: SkillItemContent {
						render: Renderer {
							model: ModelRender::Hand(AssetModel::Path("models/pistol.glb")),
							essence: EssenceRender::StandardMaterial,
						},
						skill: Some(asset_server.load(skill_asset!("shoot_hand_gun"))),
						item_type: SkillItemType::Pistol,
					},
				}),
			),
			(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem {
					name: "Force Essence A",
					content: SkillItemContent {
						render: Renderer {
							model: ModelRender::None,
							essence: EssenceRender::Material(force_essence_material.clone()),
						},
						skill: Some(asset_server.load(skill_asset!("force_shield"))),
						item_type: SkillItemType::ForceEssence,
					},
				}),
			),
			(
				SlotKey::TopHand(Side::Right),
				Some(SkillItem {
					name: "Force Essence B",
					content: SkillItemContent {
						render: Renderer {
							model: ModelRender::None,
							essence: EssenceRender::Material(force_essence_material.clone()),
						},
						skill: Some(asset_server.load(skill_asset!("force_shield"))),
						item_type: SkillItemType::ForceEssence,
					},
				}),
			),
		])
	}

	fn get_inventory(asset_server: &AssetServer) -> Inventory {
		Inventory::new([
			Some(SkillItem {
				name: "Plasma Pistol C",
				content: SkillItemContent {
					render: Renderer {
						model: ModelRender::Hand(AssetModel::Path("models/pistol.glb")),
						essence: EssenceRender::StandardMaterial,
					},
					skill: Some(asset_server.load(skill_asset!("shoot_hand_gun"))),
					item_type: SkillItemType::Pistol,
				},
			}),
			Some(SkillItem {
				name: "Plasma Pistol D",
				content: SkillItemContent {
					render: Renderer {
						model: ModelRender::Hand(AssetModel::Path("models/pistol.glb")),
						essence: EssenceRender::StandardMaterial,
					},
					skill: Some(asset_server.load(skill_asset!("shoot_hand_gun"))),
					item_type: SkillItemType::Pistol,
				},
			}),
			Some(SkillItem {
				name: "Plasma Pistol E",
				content: SkillItemContent {
					render: Renderer {
						model: ModelRender::Hand(AssetModel::Path("models/pistol.glb")),
						essence: EssenceRender::StandardMaterial,
					},
					skill: Some(asset_server.load(skill_asset!("shoot_hand_gun"))),
					item_type: SkillItemType::Pistol,
				},
			}),
		])
	}

	fn get_combos() -> ComboBundle {
		let timeout = CombosTimeOut::after(Duration::from_secs(2));

		ComboBundle::with_timeout(timeout)
	}

	fn item_essence_render(&self, app: &mut App) {
		app.add_systems(Update, EssenceRender::apply_material_exclusivity);
	}
}

impl<TState> Plugin for SkillsPlugin<TState>
where
	TState: States + Copy,
{
	fn build(&self, app: &mut App) {
		self.skill_load(app);
		self.skill_slot_load(app);
		self.skill_execution(app);

		self.item_essence_render(app);
	}
}
