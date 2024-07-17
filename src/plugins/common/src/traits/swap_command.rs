#[derive(Debug, PartialEq, Clone)]
pub struct SwappedOut<T>(pub Option<T>);

#[derive(Debug, PartialEq, Clone)]
pub struct SwapIn<T>(pub Option<T>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SwapError {
	TryAgain,
	Disregard,
}

pub type SwapResult<TValue> = Result<SwappedOut<TValue>, SwapError>;

pub trait SwapCommands<TKey, TValue> {
	fn try_swap(&mut self, swap_fn: impl FnMut(TKey, SwapIn<TValue>) -> SwapResult<TValue>);
	fn is_empty(&self) -> bool;
}
