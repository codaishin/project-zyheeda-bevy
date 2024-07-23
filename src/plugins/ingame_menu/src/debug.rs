use crate::{components::dropdown::Dropdown, tools::Layout, AddDropdown};
#[cfg(debug_assertions)]
use crate::{
	tools::menu_state::MenuState,
	traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn},
	AddUI,
};
use bevy::{
	app::{App, Update},
	color::Color,
	ecs::system::Res,
	hierarchy::{BuildChildren, BuildWorldChildren, ChildBuilder, DespawnRecursiveExt},
	prelude::{Changed, Commands, Component, Entity, Query, State},
	text::TextStyle,
	time::{Real, Time},
	ui::{
		node_bundles::{ButtonBundle, NodeBundle, TextBundle},
		AlignItems,
		FlexDirection,
		Interaction,
		JustifyContent,
		PositionType,
		Style,
		UiRect,
		Val,
	},
	utils::default,
};
use common::{tools::Index, traits::iteration::IterFinite};
use std::time::Duration;

#[derive(Component, Default)]
struct StateTime(Duration, Option<MenuState>);

impl GetNode for StateTime {
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

impl InstantiateContentOn for StateTime {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let state = self.1.map(|s| format!("{s:?}")).unwrap_or("???".into());
		parent.spawn(TextBundle::from_section(
			format!(
				"{}.{:0>3} seconds in menu: {state}",
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

fn update_state_time(
	mut run_times: Query<&mut StateTime>,
	time: Res<Time<Real>>,
	state: Res<State<MenuState>>,
) {
	let Ok(mut run_time) = run_times.get_single_mut() else {
		return;
	};
	run_time.0 += time.delta();
	run_time.1 = Some(*state.get());
}

pub fn setup_run_time_display(app: &mut App) {
	for state in MenuState::iterator() {
		app.add_ui::<StateTime>(state);
	}
	app.add_systems(Update, update_state_time);
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

#[derive(Component, Clone)]
struct ButtonOption {
	target: Entity,
	text: &'static str,
}

impl GetNode for ButtonOption {
	fn node(&self) -> NodeBundle {
		NodeBundle::default()
	}
}

impl InstantiateContentOn for ButtonOption {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let option = (Button::bundle(), self.clone());
		parent.spawn(option).with_children(|button| {
			button.spawn(TextBundle::from_section(self.text, Button::text_style()));
		});
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

fn replace_button_text(
	mut buttons: Query<&mut Button>,
	options: Query<(&ButtonOption, &Interaction), Changed<Interaction>>,
) {
	for (options, interaction) in &options {
		if interaction != &Interaction::Pressed {
			continue;
		}
		let Ok(mut button) = buttons.get_mut(options.target) else {
			continue;
		};

		button.text = options.text;
	}
}

pub fn setup_dropdown_test(app: &mut App) {
	fn get_items(target: Entity) -> Vec<ButtonOption> {
		vec![
			ButtonOption { text: "1", target },
			ButtonOption { text: "2", target },
			ButtonOption { text: "3", target },
			ButtonOption { text: "4", target },
			ButtonOption { text: "5", target },
		]
	}

	fn get_style() -> Style {
		Style {
			position_type: PositionType::Absolute,
			top: Val::Percent(0.),
			right: Val::Percent(100.),
			..default()
		}
	}

	app.add_dropdown::<ButtonOption>();
	app.add_systems(Update, (replace_button_text, update_button_text));
	app.world_mut()
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
					layout: Layout::SINGLE_ROW,
					style: get_style(),
					items: get_items(button.id()),
				},
			));
			let mut button = container.spawn_empty();
			button.insert((
				Button { text: "" },
				Button::bundle(),
				Dropdown {
					layout: Layout::SINGLE_COLUMN,
					style: get_style(),
					items: get_items(button.id()),
				},
			));
			let mut button = container.spawn_empty();
			button.insert((
				Button { text: "" },
				Button::bundle(),
				Dropdown {
					layout: Layout::LastColumn(Index(1)),
					style: get_style(),
					items: get_items(button.id()),
				},
			));
		});
}
