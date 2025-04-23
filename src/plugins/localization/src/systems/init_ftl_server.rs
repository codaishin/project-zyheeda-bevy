use bevy::prelude::*;
use unic_langid::LanguageIdentifier;

impl<T> InitFtlServer for T where T: From<LanguageIdentifier> + Resource {}

pub(crate) trait InitFtlServer: From<LanguageIdentifier> + Resource {
	fn init_with(ln: LanguageIdentifier) -> impl Fn(Commands) {
		init_with::<Self>(ln)
	}
}

fn init_with<TFtlServer>(ln: LanguageIdentifier) -> impl Fn(Commands)
where
	TFtlServer: From<LanguageIdentifier> + Resource,
{
	move |mut commands| {
		let server = TFtlServer::from(ln.clone());
		commands.insert_resource(server);
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;
	use unic_langid::langid;

	#[derive(Resource, Debug, PartialEq)]
	struct _FtlServer {
		ln: LanguageIdentifier,
	}

	impl From<LanguageIdentifier> for _FtlServer {
		fn from(ln: LanguageIdentifier) -> Self {
			Self { ln }
		}
	}

	fn setup(ln: LanguageIdentifier) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, init_with::<_FtlServer>(ln));

		app
	}

	#[test]
	fn init_with_language() {
		let mut app = setup(langid!("JP"));

		app.update();

		assert_eq!(
			Some(&_FtlServer { ln: langid!("JP") }),
			app.world().get_resource::<_FtlServer>()
		);
	}
}
