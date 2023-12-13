pub mod inventory_panel;

use crate::plugins::ingame_menu::tools::PanelState;

pub trait GetPanelState {
	fn get_panel_state(&self) -> PanelState;
}
