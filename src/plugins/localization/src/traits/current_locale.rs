use crate::resources::ftl_server::Locale;
use std::ops::DerefMut;

pub(crate) trait CurrentLocaleMut {
	fn current_locale_mut(&mut self) -> &mut Locale;
}

impl<T> CurrentLocaleMut for T
where
	T: DerefMut<Target: CurrentLocaleMut>,
{
	fn current_locale_mut(&mut self) -> &mut Locale {
		self.deref_mut().current_locale_mut()
	}
}
