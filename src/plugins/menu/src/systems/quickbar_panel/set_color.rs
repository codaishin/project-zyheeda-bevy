use crate::{
	components::{
		ColorOverride,
		dispatch_text_color::DispatchTextColor,
		quickbar_panel::QuickbarPanel,
	},
	traits::colors::ColorConfig,
};
use bevy::prelude::*;
use common::{
	components::ui_input_primer::{IsPrimed, UiInputPrimer},
	tools::{
		action_key::{
			slot::{PlayerSlot, SlotKey},
			user_input::UserInput,
		},
		skill_execution::SkillExecution,
	},
	traits::{
		accessors::get::{
			AssociatedStaticSystemParam,
			AssociatedSystemParam,
			DynProperty,
			GetFromSystemParam,
			GetProperty,
		},
		handles_loadout::loadout::NoSkill,
		key_mappings::GetInput,
	},
};

impl QuickbarPanel {
	pub(crate) fn set_color<TAgent, TMap, TSlots>(
		commands: Commands,
		buttons: Query<(Entity, &Self, &UiInputPrimer)>,
		map: Res<TMap>,
		slots: Query<&TSlots, With<TAgent>>,
		param: AssociatedStaticSystemParam<TSlots, SlotKey>,
	) where
		TAgent: Component,
		TMap: Resource + GetInput<PlayerSlot, TInput = UserInput>,
		TSlots: Component + GetFromSystemParam<SlotKey>,
		for<'i> TSlots::TItem<'i>: GetProperty<Result<SkillExecution, NoSkill>>,
	{
		set_color(commands, buttons, map, slots, param)
	}
}

fn set_color<TAgent, TMap, TPrimer, TSlots>(
	mut commands: Commands,
	buttons: Query<(Entity, &QuickbarPanel, &TPrimer)>,
	map: Res<TMap>,
	slots: Query<&TSlots, With<TAgent>>,
	param: AssociatedStaticSystemParam<TSlots, SlotKey>,
) where
	TAgent: Component,
	TMap: Resource + GetInput<PlayerSlot, TInput = UserInput>,
	TPrimer: Component + GetProperty<UserInput> + GetProperty<IsPrimed>,
	TSlots: Component + GetFromSystemParam<SlotKey>,
	for<'i> TSlots::TItem<'i>: GetProperty<Result<SkillExecution, NoSkill>>,
{
	for slots in &slots {
		for (entity, panel, primer) in &buttons {
			let Ok(entity) = commands.get_entity(entity) else {
				continue;
			};
			let color = get_color_override(map.as_ref(), panel, primer, slots, &param);
			update_color_override(color, entity);
		}
	}
}

fn get_color_override<TSlots, TMap, TPrimer>(
	map: &TMap,
	QuickbarPanel { key, .. }: &QuickbarPanel,
	primer: &TPrimer,
	slots: &TSlots,
	param: &AssociatedSystemParam<TSlots, SlotKey>,
) -> Option<ColorConfig>
where
	TMap: GetInput<PlayerSlot, TInput = UserInput>,
	TPrimer: Component + GetProperty<UserInput> + GetProperty<IsPrimed>,
	TSlots: Component + GetFromSystemParam<SlotKey>,
	for<'i> TSlots::TItem<'i>: GetProperty<Result<SkillExecution, NoSkill>>,
{
	let item = slots.get_from_param(&SlotKey::from(*key), param)?;
	let state = item.dyn_property::<Result<SkillExecution, NoSkill>>();

	if state == Ok(SkillExecution::Active) {
		return Some(QuickbarPanel::ACTIVE_COLORS);
	}

	if state == Ok(SkillExecution::Queued) {
		return Some(QuickbarPanel::QUEUED_COLORS);
	}

	let primer_is_primed = primer.dyn_property::<IsPrimed>();
	let primer_key = primer.dyn_property::<UserInput>();
	if primer_is_primed && map.get_input(*key) == primer_key {
		return Some(QuickbarPanel::PANEL_COLORS.pressed);
	}

	None
}

fn update_color_override(color: Option<ColorConfig>, mut entity: EntityCommands) {
	match color {
		Some(ColorConfig { background, text }) => {
			entity.try_insert((
				ColorOverride,
				BackgroundColor::from(background),
				DispatchTextColor::from(text),
			));
		}
		None => {
			entity.remove::<ColorOverride>();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{ColorOverride, dispatch_text_color::DispatchTextColor};
	use bevy::{ecs::system::SystemParam, state::app::StatesPlugin};
	use common::{components::ui_input_primer::IsPrimed, tools::action_key::user_input::UserInput};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Primer {
		key: UserInput,
		is_primed: IsPrimed,
	}

	impl GetProperty<UserInput> for _Primer {
		fn get_property(&self) -> UserInput {
			self.key
		}
	}

	impl GetProperty<IsPrimed> for _Primer {
		fn get_property(&self) -> bool {
			self.is_primed.0
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	impl Default for _Map {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_get_input()
					.return_const(UserInput::from(KeyCode::KeyA));
			})
		}
	}

	#[automock]
	impl GetInput<PlayerSlot> for _Map {
		type TInput = UserInput;

		fn get_input(&self, value: PlayerSlot) -> UserInput {
			self.mock.get_input(value)
		}
	}

	#[derive(Component)]
	struct _Agent;

	#[derive(Component)]
	struct _Slots(HashMap<SlotKey, _Item>);

	impl<T> From<T> for _Slots
	where
		T: IntoIterator<Item = (SlotKey, _Item)>,
	{
		fn from(items: T) -> Self {
			Self(HashMap::from_iter(items))
		}
	}

	impl GetFromSystemParam<SlotKey> for _Slots {
		type TParam<'w, 's> = _Param;
		type TItem<'i> = _Item;

		fn get_from_param(&self, key: &SlotKey, _: &_Param) -> Option<Self::TItem<'_>> {
			self.0.get(key).cloned()
		}
	}

	#[derive(SystemParam)]
	struct _Param;

	#[derive(Clone)]
	struct _Item(Option<SkillExecution>);

	impl GetProperty<Result<SkillExecution, NoSkill>> for _Item {
		fn get_property(&self) -> Result<SkillExecution, NoSkill> {
			match self.0 {
				Some(e) => Ok(e),
				None => Err(NoSkill),
			}
		}
	}

	fn setup(key_map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, set_color::<_Agent, _Map, _Primer, _Slots>);
		app.add_plugins(StatesPlugin);
		app.insert_resource(key_map);

		app
	}

	#[test]
	fn set_to_active_when_matching_skill_active() {
		let mut app = setup(_Map::default());
		app.world_mut().spawn((
			_Agent,
			_Slots::from([(
				SlotKey::from(PlayerSlot::LOWER_L),
				_Item(Some(SkillExecution::Active)),
			)]),
		));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(
				QuickbarPanel::ACTIVE_COLORS.background,
				true,
				Some(&DispatchTextColor::from(QuickbarPanel::ACTIVE_COLORS.text))
			),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}

	#[test]
	fn no_override_when_no_matching_skill_active() {
		let mut app = setup(_Map::default());
		app.world_mut().spawn((_Agent, _Slots::from([])));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		assert_eq!(
			(Color::NONE, false),
			(color.0, panel.contains::<ColorOverride>())
		);
	}

	#[test]
	fn no_override_when_no_skill_not_active() {
		let mut app = setup(_Map::default());
		app.world_mut().spawn((
			_Agent,
			_Slots::from([(
				SlotKey::from(PlayerSlot::LOWER_L),
				_Item(Some(SkillExecution::None)),
			)]),
		));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		assert_eq!(
			(Color::NONE, false),
			(color.0, panel.contains::<ColorOverride>())
		);
	}

	#[test]
	fn set_to_pressed_when_matching_key_primed() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_get_input()
				.times(1)
				.with(eq(PlayerSlot::LOWER_L))
				.return_const(UserInput::from(KeyCode::KeyQ));
		}));
		app.world_mut().spawn((
			_Agent,
			_Slots::from([(
				SlotKey::from(PlayerSlot::LOWER_L),
				_Item(Some(SkillExecution::None)),
			)]),
		));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyQ),
					is_primed: IsPrimed(true),
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(
				QuickbarPanel::PANEL_COLORS.pressed.background,
				true,
				Some(&DispatchTextColor::from(
					QuickbarPanel::PANEL_COLORS.pressed.text
				))
			),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}

	#[test]
	fn set_to_queued_when_matching_with_queued_skill() {
		let mut app = setup(_Map::default());
		app.world_mut().spawn((
			_Agent,
			_Slots::from([(
				SlotKey::from(PlayerSlot::LOWER_L),
				_Item(Some(SkillExecution::Queued)),
			)]),
		));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(
				QuickbarPanel::QUEUED_COLORS.background,
				true,
				Some(&DispatchTextColor::from(QuickbarPanel::QUEUED_COLORS.text))
			),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}

	#[test]
	fn do_nothing_if_slots_has_no_agent() {
		let mut app = setup(_Map::default());
		app.world_mut().spawn(_Slots::from([(
			SlotKey::from(PlayerSlot::LOWER_L),
			_Item(Some(SkillExecution::Active)),
		)]));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(Color::NONE, false, None),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}
}
