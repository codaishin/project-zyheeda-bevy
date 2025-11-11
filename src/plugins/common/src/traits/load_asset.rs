pub mod asset_server;
pub mod load_context;

use bevy::{asset::AssetPath, prelude::*};
use std::collections::HashMap;

pub trait LoadAsset {
	fn load_asset<'a, TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
	where
		TAsset: Asset,
		TPath: Into<AssetPath<'a>>;
}

/// Contains mock asset server for tests
///
/// <div class="warning">
///   DO NOT USE FOR PRODUCTION CODE!!! ONLY FOR TESTS!!!
/// </div>
pub mod mock {
	use super::*;
	use bevy::asset::io::AssetSourceId;
	use std::path::PathBuf;
	use uuid::Uuid;

	#[derive(Resource, Default, Debug, PartialEq)]
	pub struct MockAssetServer {
		assets: HashMap<MockAssetPath, UntypedHandle>,
		calls: HashMap<MockAssetPath, usize>,
	}

	impl MockAssetServer {
		pub fn path<'a, TPath>(self, path: TPath) -> MockAssetServerArg
		where
			TPath: Into<AssetPath<'a>>,
		{
			MockAssetServerArg {
				server: self,
				path: MockAssetPath::from(path.into()),
			}
		}

		pub fn calls<'a, TPath>(&self, path: TPath) -> usize
		where
			TPath: Into<AssetPath<'a>>,
		{
			let path = MockAssetPath::from(path.into());

			self.calls.get(&path).copied().unwrap_or_default()
		}
	}

	impl LoadAsset for MockAssetServer {
		fn load_asset<'a, TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'a>>,
		{
			let path = MockAssetPath::from(path.into());

			let calls = self.calls.entry(path.clone()).or_default();
			*calls += 1;

			let Some(handle) = self.assets.get(&path).cloned() else {
				// Returning randomly generated handle when not configured instead of the bevy default.
				// Allows test setups to be shorter but avoids tests passing falsely, when actual
				// bevy default handles are expected.
				return Handle::Weak(AssetId::Uuid {
					uuid: Uuid::new_v4(),
				});
			};

			match handle.try_typed() {
				Ok(handle) => handle,
				Err(error) => panic!("{error}"),
			}
		}
	}

	#[derive(Debug, PartialEq, Eq, Hash, Clone)]
	pub(crate) struct MockAssetPath {
		source: Option<String>,
		path: PathBuf,
		label: Option<String>,
	}

	impl From<AssetPath<'_>> for MockAssetPath {
		fn from(value: AssetPath) -> Self {
			Self {
				source: match value.source() {
					AssetSourceId::Default => None,
					AssetSourceId::Name(name) => Some(name.to_string()),
				},
				path: value.path().to_owned(),
				label: value.label().map(|label| label.to_string()),
			}
		}
	}

	pub struct MockAssetServerArg {
		server: MockAssetServer,
		path: MockAssetPath,
	}

	impl MockAssetServerArg {
		pub fn returns<TAsset>(mut self, handle: Handle<TAsset>) -> MockAssetServer
		where
			TAsset: Asset,
		{
			self.server.assets.insert(self.path, handle.untyped());
			self.server
		}
	}
}
