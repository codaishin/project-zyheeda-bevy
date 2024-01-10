use crate::plugins::ingame_menu::{components::QuickbarPanel, tools::PanelState, traits::get::Get};

impl Get<(), PanelState> for QuickbarPanel {
	fn get(&self, _: ()) -> PanelState {
		PanelState::Filled
	}
}
