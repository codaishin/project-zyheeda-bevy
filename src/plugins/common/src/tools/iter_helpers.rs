use crate::traits::iteration::{Iter, IterFinite};

/// Get the first element of an [`IterFinite`] type and wrap it in a container
///
/// # Example
/// ```
/// use common::{
///   states::{game_state::GameState, menu_state::MenuState},
///   tools::iter_helpers::first,
/// };
///
/// let state = first(GameState::IngameMenu);
///
/// assert_eq!(state, Some(GameState::IngameMenu(MenuState::Inventory)));
/// ```
pub fn first<TOuter, TInner>(wrap: impl Fn(TInner) -> TOuter) -> Option<TOuter>
where
	TInner: IterFinite,
{
	TInner::iterator().0.map(wrap)
}

/// Get the next element of an [`IterFinite`] type and wrap it in a container
///
/// # Example
/// ```
/// use common::{
///   states::{game_state::GameState, menu_state::MenuState},
///   tools::iter_helpers::next,
/// };
///
/// let state = next(GameState::IngameMenu, MenuState::Inventory);
///
/// assert_eq!(state, Some(GameState::IngameMenu(MenuState::ComboOverview)));
/// ```
pub fn next<TOuter, TInner>(wrap: impl Fn(TInner) -> TOuter, key: TInner) -> Option<TOuter>
where
	TInner: IterFinite,
{
	TInner::next(&Iter(Some(key))).map(wrap)
}
