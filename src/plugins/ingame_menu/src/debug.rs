#[cfg(debug_assertions)]
use crate::{
	tools::menu_state::MenuState,
	traits::{children::Children, colors::HasBackgroundColor, get_node::GetNode},
	AddUI,
};
use bevy::{
	app::{App, Update},
	ecs::system::Res,
	hierarchy::ChildBuilder,
	prelude::{Component, Query, State},
	render::color::Color,
	text::TextStyle,
	time::{Real, Time},
	ui::{
		node_bundles::{NodeBundle, TextBundle},
		PositionType,
		Style,
		UiRect,
		Val,
	},
	utils::default,
};
use common::traits::iteration::IterFinite;
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

impl Children for StateTime {
	fn children(&self, parent: &mut ChildBuilder) {
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

impl HasBackgroundColor for StateTime {
	const BACKGROUND_COLOR: Option<Color> = Some(Color::BLACK);
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
