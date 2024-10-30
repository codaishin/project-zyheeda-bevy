use crate::{
	components::{dropdown::Dropdown, tooltip::Tooltip},
	tools::Layout,
	traits::{GetLayout, LoadUi, RootStyle},
	AddDropdown,
	AddTooltip,
};
#[cfg(debug_assertions)]
use crate::{
	traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn},
	AddUI,
};
use bevy::prelude::*;
use common::{tools::Index, traits::iteration::IterFinite};
use std::{fmt::Debug, marker::PhantomData, time::Duration};

#[derive(Component)]
struct StateTime<TState>(Duration, Option<TState>);

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

impl<TState> GetNode for StateTime<TState> {
	fn node(&self) -> NodeBundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				right: Val::Px(10.),
				bottom: Val::Px(10.),
				border: UiRect::all(Val::Px(2.)),
				..default()
			},
			background_color: Color::BLACK.into(),
			..default()
		}
	}
}

impl<TState> InstantiateContentOn for StateTime<TState>
where
	TState: Debug + Copy,
{
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let state = self.1.map(|s| format!("{s:?}")).unwrap_or("???".into());
		parent.spawn(TextBundle::from_section(
			format!(
				"{}.{:0>3} seconds in state: {state}",
				self.0.as_secs(),
				self.0.subsec_millis()
			),
			TextStyle {
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
	let Ok(mut run_time) = run_times.get_single_mut() else {
		return;
	};
	run_time.0 += time.delta();
	run_time.1 = Some(*state.get());
}

pub fn setup_run_time_display<TState>(app: &mut App)
where
	TState: IterFinite + States + Copy,
{
	for state in TState::iterator() {
		app.add_ui::<StateTime<TState>>(state);
	}
	app.add_systems(Update, update_state_time::<TState>);
}

#[derive(Component)]
struct Button {
	text: &'static str,
}

impl Button {
	fn bundle() -> ButtonBundle {
		ButtonBundle {
			style: Style {
				width: Val::Px(60.),
				height: Val::Px(60.),
				border: UiRect::all(Val::Px(3.)),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			border_color: Color::srgb(0.8, 0.7, 0.23).into(),
			background_color: Color::WHITE.into(),
			..default()
		}
	}

	fn text_style() -> TextStyle {
		TextStyle {
			font_size: 30.,
			color: Color::BLACK,
			..default()
		}
	}
}

struct ButtonTooltip(String);

impl GetNode for Tooltip<ButtonTooltip> {
	fn node(&self) -> NodeBundle {
		NodeBundle {
			style: Style {
				top: Val::Px(-25.0),
				padding: UiRect::all(Val::Px(5.0)),
				..default()
			},
			background_color: Color::WHITE.into(),
			..default()
		}
	}
}

impl InstantiateContentOn for Tooltip<ButtonTooltip> {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		parent.spawn(TextBundle::from_section(
			&self.value().0,
			Button::text_style(),
		));
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
struct ButtonOption<TLayout: Sync + Send + 'static, TValue = &'static str> {
	phantom_data: PhantomData<TLayout>,
	value: TValue,
	target: Entity,
}

impl<TLayout: Sync + Send + 'static, TValue: Clone> Clone for ButtonOption<TLayout, TValue> {
	fn clone(&self) -> Self {
		Self {
			phantom_data: self.phantom_data,
			value: self.value.clone(),
			target: self.target,
		}
	}
}

impl<TLayout: Sync + Send + 'static, TValue> ButtonOption<TLayout, TValue> {
	fn new(value: TValue, target: Entity) -> Self {
		Self {
			phantom_data: PhantomData,
			value,
			target,
		}
	}
}

impl<TLayout: Sync + Send + 'static, TValue> GetNode for ButtonOption<TLayout, TValue> {
	fn node(&self) -> NodeBundle {
		NodeBundle::default()
	}
}

impl<TLayout: Sync + Send + 'static> InstantiateContentOn for ButtonOption<TLayout> {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let option = (
			Button::bundle(),
			self.clone(),
			Tooltip::new(ButtonTooltip(format!("Button: {}", self.value))),
		);
		parent.spawn(option).with_children(|button| {
			button.spawn(TextBundle::from_section(self.value, Button::text_style()));
		});
	}
}

impl<TLayout: Sync + Send + 'static, TSubLayout: Sync + Send + 'static> InstantiateContentOn
	for ButtonOption<TLayout, WithSubDropdown<TSubLayout>>
{
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let option = (
			Button::bundle(),
			Dropdown {
				items: get_button_options_numbered::<TSubLayout>(self.target),
			},
			Tooltip::new(ButtonTooltip("Button: subs".to_owned())),
		);
		parent.spawn(option).with_children(|button| {
			button.spawn(TextBundle::from_section("subs", Button::text_style()));
		});
	}
}

impl<TLayout: Sync + Send + 'static, TValue> RootStyle for Dropdown<ButtonOption<TLayout, TValue>> {
	fn root_style(&self) -> Style {
		Style {
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

fn update_button_text(mut commands: Commands, buttons: Query<(Entity, &Button), Changed<Button>>) {
	for (entity, button) in &buttons {
		let Some(mut entity) = commands.get_entity(entity) else {
			continue;
		};
		entity.despawn_descendants();
		entity.with_children(|parent| {
			parent.spawn(TextBundle::from_section(button.text, Button::text_style()));
		});
	}
}

fn replace_button_text<TLayout: Sync + Send + 'static>(
	mut buttons: Query<&mut Button>,
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

fn get_button_options_numbered<TLayout: Sync + Send + 'static>(
	target: Entity,
) -> Vec<ButtonOption<TLayout>> {
	vec![
		ButtonOption::new("1", target),
		ButtonOption::new("2", target),
		ButtonOption::new("3", target),
		ButtonOption::new("4", target),
		ButtonOption::new("5", target),
	]
}

fn get_button_options<TLayout: Sync + Send + 'static, TExtra: Sync + Send + 'static + Default>(
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

pub fn setup_dropdown_test(app: &mut App) {
	app.add_tooltip::<ButtonTooltip>()
		.add_dropdown::<ButtonOption<SingleRow>>()
		.add_dropdown::<ButtonOption<SingleColumn>>()
		.add_dropdown::<ButtonOption<TwoColumns>>()
		.add_dropdown::<ButtonOption<SingleRow, WithSubDropdown<SingleColumn>>>()
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
		.spawn(NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				top: Val::Px(20.),
				right: Val::Px(20.),
				flex_direction: FlexDirection::Column,
				..default()
			},
			..default()
		})
		.with_children(|container| {
			let mut button = container.spawn_empty();
			button.insert((
				Button { text: "" },
				Button::bundle(),
				Dropdown {
					items: get_button_options_numbered::<SingleRow>(button.id()),
				},
			));
			let mut button = container.spawn_empty();
			button.insert((
				Button { text: "" },
				Button::bundle(),
				Dropdown {
					items: get_button_options_numbered::<SingleColumn>(button.id()),
				},
			));
			let mut button = container.spawn_empty();
			button.insert((
				Button { text: "" },
				Button::bundle(),
				Dropdown {
					items: get_button_options_numbered::<TwoColumns>(button.id()),
				},
			));
			let mut button = container.spawn_empty();
			button.insert((
				Button { text: "" },
				Button::bundle(),
				Dropdown {
					items: get_button_options::<SingleRow, WithSubDropdown<SingleColumn>>(
						button.id(),
					),
				},
			));
		});
}
