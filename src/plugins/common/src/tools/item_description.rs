use crate::traits::{handles_localization::Token, inspect_able::InspectMarker};

#[derive(Debug, PartialEq)]
pub struct ItemToken;

impl InspectMarker for ItemToken {
	type TFieldRef<'a> = &'a Token;
}
