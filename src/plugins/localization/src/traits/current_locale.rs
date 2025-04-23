use crate::resources::ftl_server::Locale;

pub(crate) trait CurrentLocaleMut {
	fn current_locale_mut(&mut self) -> &mut Locale;
}
