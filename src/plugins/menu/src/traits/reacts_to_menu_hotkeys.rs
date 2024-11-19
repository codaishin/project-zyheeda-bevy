mod game_state;

pub trait ReactsToMenuHotkeys {
	fn reacts_to_menu_hotkeys(&self) -> bool;
}
