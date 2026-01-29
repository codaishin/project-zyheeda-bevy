pub mod asset_server;

use bevy::{
	asset::{AssetPath, LoadedFolder},
	prelude::*,
};

pub trait LoadFolderAssets {
	fn load_folder_assets<'a, TPath>(&self, path: TPath) -> Handle<LoadedFolder>
	where
		TPath: Into<AssetPath<'a>>;
}

/// Contains mock asset server for tests
///
/// <div class="warning">
///   DO NOT USE FOR PRODUCTION CODE!!! ONLY FOR TESTS!!!
/// </div>
pub mod mock {
	use super::*;
	use crate::traits::load_asset::mock::MockAssetPath;
	use std::{
		collections::HashMap,
		sync::{Arc, RwLock},
	};
	use uuid::Uuid;

	/// Use for test mocks only
	#[derive(Resource, Default, Debug)]
	pub struct MockFolderAssetServer {
		assets: HashMap<MockAssetPath, Handle<LoadedFolder>>,
		calls: Arc<RwLock<HashMap<MockAssetPath, usize>>>,
	}

	impl MockFolderAssetServer {
		pub fn path<'a, TPath>(self, path: TPath) -> MockFolderAssetServerArg
		where
			TPath: Into<AssetPath<'a>>,
		{
			MockFolderAssetServerArg {
				server: self,
				path: MockAssetPath::from(path.into()),
			}
		}

		pub fn calls<'a, TPath>(&self, path: TPath) -> usize
		where
			TPath: Into<AssetPath<'a>>,
		{
			let path = MockAssetPath::from(path.into());

			#[allow(clippy::unwrap_used)]
			let calls = self.calls.read().unwrap();

			calls.get(&path).copied().unwrap_or_default()
		}
	}

	impl LoadFolderAssets for MockFolderAssetServer {
		fn load_folder_assets<'a, TPath>(&self, path: TPath) -> Handle<LoadedFolder>
		where
			TPath: Into<AssetPath<'a>>,
		{
			let path = MockAssetPath::from(path.into());

			#[allow(clippy::unwrap_used)]
			let mut calls = self.calls.write().unwrap();

			let calls = calls.entry(path.clone()).or_default();
			*calls += 1;

			match self.assets.get(&path).cloned() {
				Some(handle) => handle,
				None => {
					// Returning randomly generated handle when not configured instead of the bevy default.
					// Allows test setups to be shorter but avoids tests passing falsely, when actual
					// bevy default handles are expected.
					Handle::from(Uuid::new_v4())
				}
			}
		}
	}

	pub struct MockFolderAssetServerArg {
		server: MockFolderAssetServer,
		path: MockAssetPath,
	}

	impl MockFolderAssetServerArg {
		pub fn returns(mut self, handle: Handle<LoadedFolder>) -> MockFolderAssetServer {
			self.server.assets.insert(self.path, handle);
			self.server
		}
	}
}
