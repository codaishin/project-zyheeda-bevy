use bevy::prelude::*;
use common::traits::{accessors::get::RefInto, handles_localization::Token};

#[derive(Debug, PartialEq)]
pub struct LoadoutItem {
	pub token: Token,
	pub skill_icon: Option<Handle<Image>>,
}

impl<'a> RefInto<'a, &'a Token> for LoadoutItem {
	fn ref_into(&self) -> &Token {
		&self.token
	}
}

impl<'a> RefInto<'a, &'a Option<Handle<Image>>> for LoadoutItem {
	fn ref_into(&self) -> &Option<Handle<Image>> {
		&self.skill_icon
	}
}
