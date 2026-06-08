use std::ops::DerefMut;

pub(crate) trait UpdateCurrentLocaleFromFile {
	fn update_current_locale_from_file(&mut self) -> &mut bool;
}

impl<T> UpdateCurrentLocaleFromFile for T
where
	T: DerefMut<Target: UpdateCurrentLocaleFromFile>,
{
	fn update_current_locale_from_file(&mut self) -> &mut bool {
		self.deref_mut().update_current_locale_from_file()
	}
}

pub(crate) trait UpdateCurrentLocaleFromFolder {
	fn update_current_locale_from_folder(&mut self) -> &mut bool;
}

impl<T> UpdateCurrentLocaleFromFolder for T
where
	T: DerefMut<Target: UpdateCurrentLocaleFromFolder>,
{
	fn update_current_locale_from_folder(&mut self) -> &mut bool {
		self.deref_mut().update_current_locale_from_folder()
	}
}
