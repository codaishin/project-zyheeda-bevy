pub trait NestedMock<TInnerMock> {
	fn new_mock(configure_mock_fn: impl FnMut(&mut TInnerMock)) -> Self;
}
