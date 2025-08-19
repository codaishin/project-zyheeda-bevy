#[macro_export]
macro_rules! yield_eager {
	($($items:expr),* $(,)?) => {
		[$($items),*].into_iter()
	};
}

pub use yield_eager;
