use super::ReactsToMenuHotkeys;
use common::states::game_state::GameState;

impl ReactsToMenuHotkeys for GameState {
	fn reacts_to_menu_hotkeys(&self) -> bool {
		match self {
			Self::None => false,
			Self::StartMenu => false,
			Self::Loading => false,
			Self::NewGame => false,
			Self::Play => true,
			Self::Saving => false,
			Self::IngameMenu(_) => true,
		}
	}
}
