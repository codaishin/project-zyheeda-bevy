#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum LoadState {
	LoadAssets,
	ResoleDependencies,
}
