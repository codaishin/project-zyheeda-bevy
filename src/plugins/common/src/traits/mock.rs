pub trait Mock {
	fn new_mock(configure: impl FnMut(&mut Self)) -> Self;
}

#[macro_export]
macro_rules! simple_init {
	($ident:ident) => {
		impl Mock for $ident {
			fn new_mock(mut configure: impl FnMut(&mut Self)) -> Self {
				let mut mock = Self::default();
				configure(&mut mock);
				mock
			}
		}
	};
}

pub use simple_init;
