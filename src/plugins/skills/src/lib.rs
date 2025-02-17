pub mod components;
pub mod item;
pub mod resources;
pub mod skills;
pub mod systems;
pub mod traits;

mod behaviors;
mod bundles;

use bevy::prelude::*;
use bundles::{ComboBundle, Loadout};
use common::{
	resources::key_map::KeyMap,
	states::{game_state::GameState, mouse_context::MouseContext},
	systems::{log::log_many, track_components::TrackComponentInSelfAndChildren},
	tools::{
		item_type::ItemType,
		slot_key::{Side, SlotKey},
	},
	traits::{
		handles_assets_for_children::HandlesAssetsForChildren,
		handles_combo_menu::{
			ConfigureCombos,
			GetComboAbleSkills,
			GetCombosOrdered,
			HandlesComboMenu,
			InspectAble,
			NextKeys,
			SkillIcon,
		},
		handles_custom_assets::{HandlesCustomAssets, HandlesCustomFolderAssets},
		handles_effect::HandlesAllEffects,
		handles_equipment::{
			Combo,
			CompatibleItems,
			HandlesEquipment,
			IsTimedOut,
			ItemAssets,
			IterateQueue,
			PeekNext,
			WriteItem,
		},
		handles_lifetime::HandlesLifetime,
		handles_loadout_menus::{
			ConfigureInventory,
			GetItem,
			HandlesLoadoutMenu,
			ItemDescription,
			SkillExecution,
		},
		handles_orientation::HandlesOrientation,
		handles_player::{
			ConfiguresPlayerSkillAnimations,
			HandlesPlayer,
			HandlesPlayerCameras,
			HandlesPlayerMouse,
		},
		handles_skill_behaviors::HandlesSkillBehaviors,
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
	},
};
use components::{
	combos::Combos,
	combos_time_out::CombosTimeOut,
	inventory::Inventory,
	queue::Queue,
	skill_executer::SkillExecuter,
	skill_spawners::SkillSpawners,
	slots::{ForearmItemSlots, HandItemSlots, Slots, SubMeshEssenceSlots},
	swapper::Swapper,
};
use item::{dto::ItemDto, Item};
use macros::item_asset;
use skills::{dto::SkillDto, QueuedSkill, RunSkillBehavior, Skill};
use std::{
	collections::{hash_map::Entry, HashMap, HashSet},
	hash::Hash,
	marker::PhantomData,
	time::Duration,
};
use systems::{
	advance_active_skill::advance_active_skill,
	enqueue::enqueue,
	execute::ExecuteSkills,
	flush::flush,
	flush_skill_combos::flush_skill_combos,
	get_inputs::get_inputs,
	mouse_context::{
		advance::{advance_just_released_mouse_context, advance_just_triggered_mouse_context},
		release::release_triggered_mouse_context,
		trigger_primed::trigger_primed_mouse_context,
	},
	update_skill_combos::update_skill_combos,
};

pub struct SkillsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<
		TLifeCycles,
		TInteractions,
		TDispatchChildrenAssets,
		TLoading,
		TBehaviors,
		TPlayers,
		TMenu,
	>
	SkillsPlugin<(
		TLifeCycles,
		TInteractions,
		TDispatchChildrenAssets,
		TLoading,
		TBehaviors,
		TPlayers,
		TMenu,
	)>
where
	TLifeCycles: ThreadSafe + HandlesLifetime,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TDispatchChildrenAssets: ThreadSafe + HandlesAssetsForChildren,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerSkillAnimations,
	TMenu: ThreadSafe + HandlesLoadoutMenu + HandlesComboMenu,
{
	pub fn depends_on(
		_: &TLifeCycles,
		_: &TInteractions,
		_: &TDispatchChildrenAssets,
		_: &TLoading,
		_: &TBehaviors,
		_: &TPlayers,
		_: &TMenu,
	) -> Self {
		Self(PhantomData)
	}

	fn skill_load(&self, app: &mut App) {
		TLoading::register_custom_folder_assets::<Skill, SkillDto>(app);
	}

	fn item_load(&self, app: &mut App) {
		TLoading::register_custom_assets::<Item, ItemDto>(app);
	}

	fn loadout(&self, app: &mut App) {
		TDispatchChildrenAssets::register_child_asset::<Slots, HandItemSlots>(app);
		TDispatchChildrenAssets::register_child_asset::<Slots, ForearmItemSlots>(app);
		TDispatchChildrenAssets::register_child_asset::<Slots, SubMeshEssenceSlots>(app);

		app.add_systems(
			PreUpdate,
			SkillSpawners::track_in_self_and_children::<Name>().system(),
		)
		.add_systems(Update, Self::set_player_items)
		.add_systems(Update, Swapper::system);
	}

	fn skill_execution(&self, app: &mut App) {
		let execute_skill = SkillExecuter::<RunSkillBehavior>::execute_system::<
			TLifeCycles,
			TInteractions,
			TBehaviors,
			TPlayers,
		>;

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
					update_skill_combos::<Combos, Queue>,
					flush_skill_combos::<Combos, CombosTimeOut, Virtual, Queue>,
					advance_active_skill::<Queue, TPlayers, TBehaviors, SkillExecuter, Virtual>,
					execute_skill.pipe(log_many),
					flush::<Queue>,
				)
					.chain()
					.run_if(in_state(GameState::Play)),
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
		players: Query<Entity, Added<TPlayers::TPlayer>>,
		asset_server: Res<AssetServer>,
	) {
		let Ok(player) = players.get_single() else {
			return;
		};
		let asset_server = asset_server.as_ref();

		commands.try_insert_on(
			player,
			(
				Swapper::default(),
				Self::get_inventory(asset_server),
				Self::get_loadout(asset_server),
				Self::get_combos(),
			),
		);
	}

	fn get_loadout(asset_server: &AssetServer) -> Loadout {
		Loadout::new([
			(
				SlotKey::TopHand(Side::Left),
				Some(asset_server.load(item_asset!("pistol"))),
			),
			(
				SlotKey::BottomHand(Side::Left),
				Some(asset_server.load(item_asset!("pistol"))),
			),
			(
				SlotKey::BottomHand(Side::Right),
				Some(asset_server.load(item_asset!("force_essence"))),
			),
			(
				SlotKey::TopHand(Side::Right),
				Some(asset_server.load(item_asset!("force_essence"))),
			),
		])
	}

	fn get_inventory(asset_server: &AssetServer) -> Inventory {
		Inventory::new([
			Some(asset_server.load(item_asset!("pistol"))),
			Some(asset_server.load(item_asset!("pistol"))),
			Some(asset_server.load(item_asset!("pistol"))),
		])
	}

	fn get_combos() -> ComboBundle {
		let timeout = CombosTimeOut::after(Duration::from_secs(2));

		ComboBundle::with_timeout(timeout)
	}

	fn config_menus(&self, app: &mut App) {
		TMenu::loadout_with_swapper::<Swapper>().configure(
			app,
			Self::get_descriptors::<Inventory>,
			Self::get_descriptors::<Slots>,
		);
		TMenu::configure_quickbar_menu(app, Self::get_quickbar_descriptors);
		TMenu::combos_with_skill::<Skill>().configure(
			app,
			Self::get_equipment_info,
			Self::update_combos,
		);
	}

	// FIXME: NEEDS CLEANING UP
	#[allow(clippy::type_complexity)]
	fn get_equipment_info(
		slots: Query<(Ref<Slots>, Ref<Combos>), With<TPlayers::TPlayer>>,
		items: Res<Assets<Item>>,
		skills: Res<Assets<Skill>>,
	) -> Option<EquipmentDescriptor> {
		let (slots, combos) = slots.get_single().ok()?;

		if !slots.is_changed() && !combos.is_changed() {
			return None;
		}

		let mut compatible_skills = HashMap::<SlotKey, Vec<Skill>>::default();
		let mut compatible_map = HashMap::<ItemType, Vec<SlotKey>>::default();
		let mut seen = vec![];

		for (key, handle) in slots.item_assets() {
			let Some(handle) = handle else {
				continue;
			};
			let Some(item) = items.get(handle) else {
				continue;
			};
			match compatible_map.entry(item.item_type) {
				Entry::Occupied(mut occupied_entry) => {
					occupied_entry.get_mut().push(key);
				}
				Entry::Vacant(vacant_entry) => {
					vacant_entry.insert(vec![key]);
				}
			};
		}

		for (id, skill) in skills.iter() {
			if seen.contains(&id) {
				continue;
			}
			seen.push(id);
			let CompatibleItems(item_types) = &skill.compatible_items;
			let keys = item_types
				.iter()
				.flat_map(|item_type| compatible_map.get(item_type).cloned().unwrap_or_default())
				.collect::<HashSet<_>>();

			for key in keys {
				match compatible_skills.entry(key) {
					Entry::Occupied(mut occupied_entry) => {
						if occupied_entry.get().contains(skill) {
							continue;
						}
						occupied_entry.get_mut().push(skill.clone());
					}
					Entry::Vacant(vacant_entry) => {
						vacant_entry.insert(vec![skill.clone()]);
					}
				}
			}
		}
		let combo_keys = combos
			.combos_ordered()
			.iter()
			.flat_map(|combo| combo.iter())
			.map(|(combo_keys, ..)| combo_keys.clone())
			.collect();
		let combos = combos.combos_ordered();

		Some(EquipmentDescriptor {
			compatible_skills,
			combo_keys,
			combos,
		})
	}

	fn update_combos(
		In(updated_combos): In<Combo<Option<Skill>>>,
		mut combos: Query<&mut Combos, With<TPlayers::TPlayer>>,
	) {
		let Ok(mut combos) = combos.get_single_mut() else {
			return;
		};

		for (combo_keys, skill) in updated_combos {
			combos.write_item(&combo_keys, skill);
		}
	}

	fn get_descriptors<TContainer>(
		containers: Query<&TContainer, (With<TPlayers::TPlayer>, Changed<TContainer>)>,
		items: Res<Assets<Item>>,
		skills: Res<Assets<Skill>>,
	) -> Option<Cache<TContainer::TKey, InventoryItem>>
	where
		TContainer: ItemAssets<TItem = Item> + Component,
		TContainer::TKey: Eq + Hash + Copy,
	{
		let container = containers.get_single().ok()?;
		let map = container
			.item_assets()
			.filter_map(|(key, handle)| {
				let handle = handle.as_ref()?;
				let item = items.get(handle)?;
				let image = item
					.skill
					.as_ref()
					.and_then(|handle| skills.get(handle))
					.and_then(|skill| skill.icon.clone());

				Some((
					key,
					InventoryItem {
						name: item.name.clone(),
						skill_icon: image.clone(),
					},
				))
			})
			.collect();

		Some(Cache(map))
	}

	#[allow(clippy::type_complexity)]
	fn get_quickbar_descriptors(
		queues: Query<
			(Ref<Slots>, Ref<Queue>, Ref<Combos>, Option<&CombosTimeOut>),
			With<TPlayers::TPlayer>,
		>,
		items: Res<Assets<Item>>,
		skills: Res<Assets<Skill>>,
	) -> Option<Cache<SlotKey, QuickbarItem>> {
		let (slots, queue, combos, combos_time_out) = queues.get_single().ok()?;

		if !Self::any_true(&[slots.is_changed(), queue.is_changed(), combos.is_changed()]) {
			return None;
		}

		let mut queue = queue.iterate();
		let active = queue.next();
		let queued_keys = queue.map(|skill| skill.slot_key).collect::<Vec<_>>();
		let combo_active = combos_time_out
			.map(|time_out| !time_out.is_timed_out())
			.unwrap_or(true);

		let map = slots
			.item_assets()
			.filter_map(|(key, handle)| {
				let handle = handle.as_ref()?;
				let item = items.get(handle)?;
				let skill = skills.get(item.skill.as_ref()?)?;
				let active = active.and_then(|skill| {
					if skill.slot_key != key {
						return None;
					}

					Some((skill.skill.icon.clone(), skill.skill.name.clone()))
				});

				let execution = match active {
					Some(_) => SkillExecution::Active,
					None if queued_keys.contains(&key) => SkillExecution::Queued,
					_ => SkillExecution::None,
				};

				let (icon, name) = match (active, combos.peek_next(&key, &item.item_type)) {
					(Some((active_icon, active_name)), _) => (active_icon, active_name),
					(_, Some(next)) if combo_active => (next.icon.clone(), next.name.clone()),
					_ => (skill.icon.clone(), skill.name.clone()),
				};

				Some((
					key,
					QuickbarItem {
						name,
						icon,
						execution,
					},
				))
			})
			.collect();

		Some(Cache(map))
	}

	fn any_true(values: &[bool]) -> bool {
		values.contains(&true)
	}
}

impl<
		TLifeCycles,
		TInteractions,
		TDispatchChildrenAssets,
		TLoading,
		TBehaviors,
		TPlayers,
		TMenu,
	> Plugin
	for SkillsPlugin<(
		TLifeCycles,
		TInteractions,
		TDispatchChildrenAssets,
		TLoading,
		TBehaviors,
		TPlayers,
		TMenu,
	)>
where
	TLifeCycles: ThreadSafe + HandlesLifetime,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TDispatchChildrenAssets: ThreadSafe + HandlesAssetsForChildren,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerSkillAnimations,
	TMenu: ThreadSafe + HandlesLoadoutMenu + HandlesComboMenu,
{
	fn build(&self, app: &mut App) {
		self.skill_load(app);
		self.item_load(app);
		self.loadout(app);
		self.skill_execution(app);
		self.config_menus(app);
	}
}

impl<T> HandlesEquipment for SkillsPlugin<T> {
	type TItem = Item;
	type TInventory = Inventory;
	type TSlots = Slots;
	type TCombos = Combos;
	type TSkill = Skill;
	type TQueue = Queue;
	type TQueuedSkill = QueuedSkill;
	type TCombosTimeOut = CombosTimeOut;
	type TSwap = Swapper;
}

// FIXME: NEEDS CLEANUP
#[derive(Debug, PartialEq)]
pub struct Cache<TKey, TItem>(pub HashMap<TKey, TItem>)
where
	TKey: Eq + Hash;

impl<TKey, TItem> GetItem<TKey> for Cache<TKey, TItem>
where
	TKey: Eq + Hash,
{
	type TItem = TItem;

	fn get_item(&self, key: TKey) -> Option<&TItem> {
		self.0.get(&key)
	}
}

pub struct QuickbarItem {
	pub name: String,
	pub icon: Option<Handle<Image>>,
	pub execution: SkillExecution,
}

impl InspectAble<ItemDescription> for QuickbarItem {
	fn get_inspect_able_field(&self) -> String {
		self.name.clone()
	}
}

impl InspectAble<SkillIcon> for QuickbarItem {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.icon
	}
}

impl InspectAble<SkillExecution> for QuickbarItem {
	fn get_inspect_able_field(&self) -> &SkillExecution {
		&self.execution
	}
}

pub struct InventoryItem {
	pub name: String,
	pub skill_icon: Option<Handle<Image>>,
}

impl InspectAble<ItemDescription> for InventoryItem {
	fn get_inspect_able_field(&self) -> String {
		self.name.clone()
	}
}

impl InspectAble<SkillIcon> for InventoryItem {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.skill_icon
	}
}

#[derive(Debug, PartialEq)]
pub struct EquipmentDescriptor {
	pub compatible_skills: HashMap<SlotKey, Vec<Skill>>,
	pub combo_keys: HashSet<Vec<SlotKey>>,
	pub combos: Vec<Combo<Skill>>,
}

impl GetComboAbleSkills<Skill> for EquipmentDescriptor {
	fn get_combo_able_skills(&self, key: &SlotKey) -> Vec<Skill> {
		self.compatible_skills.get(key).cloned().unwrap_or_default()
	}
}

impl NextKeys for EquipmentDescriptor {
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey> {
		self.combo_keys
			.iter()
			.filter(|combo| combo.starts_with(combo_keys))
			.filter_map(|combo| combo.get(combo_keys.len()))
			.cloned()
			.collect()
	}
}

impl GetCombosOrdered<Skill> for EquipmentDescriptor {
	fn combos_ordered(&self) -> Vec<Combo<Skill>> {
		self.combos.clone()
	}
}
