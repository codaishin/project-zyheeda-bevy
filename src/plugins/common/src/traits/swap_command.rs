pub trait SwapCommand<TSource, TKey, TValue> {
	fn get(&self, from: &TSource) -> (TKey, Option<TValue>);
	fn insert(&self, item: Option<TValue>, container: &mut TSource) -> Self;
}

#[derive(Debug, PartialEq)]
pub struct SwappedOut<T>(pub Option<T>);

#[derive(Debug, PartialEq)]
pub struct SwapIn<T>(pub Option<T>);

pub enum SwapError {
	TryAgain,
	Disregard,
}

pub type SwapResult<TValue> = Result<SwappedOut<TValue>, SwapError>;

pub trait SwapCommands<TKey, TValue> {
	fn try_swap(&mut self, swap_fn: impl FnMut(TKey, SwapIn<TValue>) -> SwapResult<TValue>);
	fn is_empty(&self) -> bool;
}
