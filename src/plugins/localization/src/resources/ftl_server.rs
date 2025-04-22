use crate::assets::ftl::Ftl;
use bevy::prelude::*;
use fluent::{FluentResource, concurrent::FluentBundle};
use unic_langid::LanguageIdentifier;

#[derive(Resource)]
pub struct FtlServer {
	pub(crate) change_to: Option<LanguageIdentifier>,
	pub(crate) fallback: Locale,
	pub(crate) current: Option<Locale>,
}

struct Locale {
	pub(crate) index: LanguageIdentifier,
	pub(crate) handle: Handle<Ftl>,
	pub(crate) bundle: Option<FluentBundle<FluentResource>>,
}

impl From<(LanguageIdentifier, Handle<Ftl>)> for FtlServer {
	fn from((index, handle): (LanguageIdentifier, Handle<Ftl>)) -> Self {
		Self {
			change_to: None,
			fallback: Locale {
				index,
				handle,
				bundle: None,
			},
			current: None,
		}
	}
}
