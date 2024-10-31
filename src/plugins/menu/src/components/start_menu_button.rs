use bevy::prelude::*;
use common::traits::accessors::get::GetterRef;

use crate::{
	tools::PanelState,
	traits::colors::{HasPanelColors, PanelColors, DEFAULT_PANEL_COLORS},
};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct StartMenuButton;

impl GetterRef<PanelState> for StartMenuButton {
	fn get(&self) -> &PanelState {
		&PanelState::Filled
	}
}

impl HasPanelColors for StartMenuButton {
	const PANEL_COLORS: PanelColors = DEFAULT_PANEL_COLORS;
}