pub(crate) trait UpdateCurrentLocaleFromFile {
	fn update_current_locale_from_file(&mut self) -> &mut bool;
}

pub(crate) trait UpdateCurrentLocaleFromFolder {
	fn update_current_locale_from_folder(&mut self) -> &mut bool;
}
