use crate::{
	AddDropdown,
	AddTooltip,
	components::{
		dropdown::Dropdown,
		tooltip::{Tooltip, TooltipUiConfig},
	},
	tools::Layout,
	traits::{GetLayout, GetRootNode, LoadUi},
};
#[cfg(debug_assertions)]
use crate::{AddUI, traits::insert_ui_content::InsertUiContent};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{
	states::game_state::GameState,
	tools::Index,
	traits::{
		handles_graphics::StaticRenderLayers,
		handles_localization::LocalizeToken,
		iteration::IterFinite,
		thread_safe::ThreadSafe,
	},
};
use std::{fmt::Debug, marker::PhantomData, time::Duration};

#[derive(Component)]
#[require(Node = squared(), BackgroundColor = black())]
struct StateTime<TState>(Duration, Option<TState>);

fn squared() -> Node {
	Node {
		position_type: PositionType::Absolute,
		right: Val::Px(10.),
		bottom: Val::Px(10.),
		border: UiRect::all(Val::Px(2.)),
		..default()
	}
}

fn black() -> BackgroundColor {
	BackgroundColor(Color::BLACK)
}

impl<TState> Default for StateTime<TState> {
	fn default() -> Self {
		Self(Duration::ZERO, None)
	}
}

impl<TState> LoadUi<AssetServer> for StateTime<TState> {
	fn load_ui(_: &mut AssetServer) -> Self {
		StateTime::default()
	}
}

impl<TState> InsertUiContent for StateTime<TState>
where
	TState: Debug + Copy,
{
	fn insert_ui_content<TLocalization>(
		&self,
		_: &mut TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) {
		let state = self.1.map(|s| format!("{s:?}")).unwrap_or("???".into());
		parent.spawn((
			Text::new(format!(
				"{}.{:0>3} seconds in state: {state}",
				self.0.as_secs(),
				self.0.subsec_millis()
			)),
			TextFont {
				font_size: 20.,
				..default()
			},
		));
	}
}

fn update_state_time<TState>(
	mut run_times: Query<&mut StateTime<TState>>,
	time: Res<Time<Real>>,
	state: Res<State<TState>>,
) where
	TState: States + Copy,
{
	let Ok(mut run_time) = run_times.single_mut() else {
		return;
	};
	run_time.0 += time.delta();
	run_time.1 = Some(*state.get());
}

pub fn setup_run_time_display<TLocalization, TGraphics>(app: &mut App)
where
	TLocalization: LocalizeToken + Resource,
	TGraphics: StaticRenderLayers + 'static,
{
	for state in GameState::iterator() {
		app.add_ui::<StateTime<GameState>, TLocalization, TGraphics>(state);
	}
	app.add_systems(Update, update_state_time::<GameState>);
}

#[derive(Component)]
struct DropdownButton {
	text: &'static str,
}

impl DropdownButton {
	fn bundle() -> (Button, Node, BorderColor, BackgroundColor) {
		(
			Button,
			Node {
				width: Val::Px(60.),
				height: Val::Px(60.),
				border: UiRect::all(Val::Px(3.)),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			Color::srgb(0.8, 0.7, 0.23).into(),
			Color::WHITE.into(),
		)
	}

	fn text_style() -> (TextFont, TextColor) {
		(
			TextFont {
				font_size: 30.,
				..default()
			},
			TextColor(Color::BLACK),
		)
	}
}

#[derive(Clone)]
struct ButtonTooltip(String);

impl TooltipUiConfig for ButtonTooltip {
	fn node() -> Node {
		Node {
			top: Val::Px(-25.0),
			padding: UiRect::all(Val::Px(5.0)),
			..default()
		}
	}

	fn background_color() -> BackgroundColor {
		BackgroundColor(Color::WHITE)
	}
}

impl InsertUiContent for Tooltip<ButtonTooltip> {
	fn insert_ui_content<TLocalization>(
		&self,
		_: &mut TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) {
		parent.spawn((Text::new(&self.value().0), DropdownButton::text_style()));
	}
}

struct SingleRow;
struct SingleColumn;
struct TwoColumns;
struct WithSubDropdown<TLayout>(PhantomData<TLayout>);

impl<TLayout> Default for WithSubDropdown<TLayout> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

#[derive(Component)]
#[require(Node)]
struct ButtonOption<TLayout: ThreadSafe, TValue = &'static str> {
	phantom_data: PhantomData<TLayout>,
	value: TValue,
	target: Entity,
}

impl<TLayout: ThreadSafe, TValue: Clone> Clone for ButtonOption<TLayout, TValue> {
	fn clone(&self) -> Self {
		Self {
			phantom_data: self.phantom_data,
			value: self.value.clone(),
			target: self.target,
		}
	}
}

impl<TLayout: ThreadSafe, TValue> ButtonOption<TLayout, TValue> {
	fn new(value: TValue, target: Entity) -> Self {
		Self {
			phantom_data: PhantomData,
			value,
			target,
		}
	}
}

impl<TLayout> InsertUiContent for ButtonOption<TLayout>
where
	TLayout: ThreadSafe,
{
	fn insert_ui_content<TLocalization>(
		&self,
		_: &mut TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) {
		let option = (
			DropdownButton::bundle(),
			self.clone(),
			Tooltip::new(ButtonTooltip(format!("Button: {}", self.value))),
		);
		parent.spawn(option).with_children(|button| {
			button.spawn((Text::new(self.value), DropdownButton::text_style()));
		});
	}
}

impl<TLayout: ThreadSafe, TSubLayout: ThreadSafe> InsertUiContent
	for ButtonOption<TLayout, WithSubDropdown<TSubLayout>>
{
	fn insert_ui_content<TLocalization>(
		&self,
		_: &mut TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) {
		let option = (
			DropdownButton::bundle(),
			Dropdown {
				items: get_button_options_numbered::<TSubLayout>(self.target),
			},
			Tooltip::new(ButtonTooltip("Button: subs".to_owned())),
		);
		parent.spawn(option).with_children(|button| {
			button.spawn((Text::new("subs"), DropdownButton::text_style()));
		});
	}
}

impl<TLayout: ThreadSafe, TValue> GetRootNode for Dropdown<ButtonOption<TLayout, TValue>> {
	fn root_node(&self) -> Node {
		Node {
			position_type: PositionType::Absolute,
			top: Val::Percent(0.),
			right: Val::Percent(100.),
			..default()
		}
	}
}

impl<TValue> GetLayout for Dropdown<ButtonOption<SingleRow, TValue>> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_ROW
	}
}

impl<TValue> GetLayout for Dropdown<ButtonOption<SingleColumn, TValue>> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_COLUMN
	}
}

impl<TValue> GetLayout for Dropdown<ButtonOption<TwoColumns, TValue>> {
	fn layout(&self) -> Layout {
		Layout::LastColumn(Index(1))
	}
}

fn update_button_text(
	mut commands: Commands,
	buttons: Query<(Entity, &DropdownButton), Changed<DropdownButton>>,
) {
	for (entity, button) in &buttons {
		let Ok(mut entity) = commands.get_entity(entity) else {
			continue;
		};
		entity.despawn_related::<Children>();
		entity.with_children(|parent| {
			parent.spawn((Text::new(button.text), DropdownButton::text_style()));
		});
	}
}

fn replace_button_text<TLayout: ThreadSafe>(
	mut buttons: Query<&mut DropdownButton>,
	options: Query<(&ButtonOption<TLayout>, &Interaction), Changed<Interaction>>,
) {
	for (options, interaction) in &options {
		if interaction != &Interaction::Pressed {
			continue;
		}
		let Ok(mut button) = buttons.get_mut(options.target) else {
			continue;
		};

		button.text = options.value;
	}
}

fn get_button_options_numbered<TLayout: ThreadSafe>(target: Entity) -> Vec<ButtonOption<TLayout>> {
	vec![
		ButtonOption::new("1", target),
		ButtonOption::new("2", target),
		ButtonOption::new("3", target),
		ButtonOption::new("4", target),
		ButtonOption::new("5", target),
	]
}

fn get_button_options<TLayout: ThreadSafe, TExtra: ThreadSafe + Default>(
	target: Entity,
) -> Vec<ButtonOption<TLayout, TExtra>> {
	vec![
		ButtonOption::new(TExtra::default(), target),
		ButtonOption::new(TExtra::default(), target),
		ButtonOption::new(TExtra::default(), target),
		ButtonOption::new(TExtra::default(), target),
		ButtonOption::new(TExtra::default(), target),
	]
}

pub fn setup_dropdown_test<TLocalization>(app: &mut App)
where
	TLocalization: LocalizeToken + Resource,
{
	app.add_tooltip::<TLocalization, ButtonTooltip>()
		.add_dropdown::<TLocalization, ButtonOption<SingleRow>>()
		.add_dropdown::<TLocalization, ButtonOption<SingleColumn>>()
		.add_dropdown::<TLocalization, ButtonOption<TwoColumns>>()
		.add_dropdown::<TLocalization, ButtonOption<SingleRow, WithSubDropdown<SingleColumn>>>()
		.add_systems(
			Update,
			(
				replace_button_text::<SingleRow>,
				replace_button_text::<SingleColumn>,
				replace_button_text::<TwoColumns>,
				update_button_text,
			),
		)
		.world_mut()
		.spawn(Node {
			position_type: PositionType::Absolute,
			top: Val::Px(20.),
			right: Val::Px(20.),
			flex_direction: FlexDirection::Column,
			..default()
		})
		.with_children(|container| {
			let mut button = container.spawn_empty();
			button.insert((
				DropdownButton { text: "" },
				DropdownButton::bundle(),
				Dropdown {
					items: get_button_options_numbered::<SingleRow>(button.id()),
				},
			));
			let mut button = container.spawn_empty();
			button.insert((
				DropdownButton { text: "" },
				DropdownButton::bundle(),
				Dropdown {
					items: get_button_options_numbered::<SingleColumn>(button.id()),
				},
			));
			let mut button = container.spawn_empty();
			button.insert((
				DropdownButton { text: "" },
				DropdownButton::bundle(),
				Dropdown {
					items: get_button_options_numbered::<TwoColumns>(button.id()),
				},
			));
			let mut button = container.spawn_empty();
			button.insert((
				DropdownButton { text: "" },
				DropdownButton::bundle(),
				Dropdown {
					items: get_button_options::<SingleRow, WithSubDropdown<SingleColumn>>(
						button.id(),
					),
				},
			));
		});
}
