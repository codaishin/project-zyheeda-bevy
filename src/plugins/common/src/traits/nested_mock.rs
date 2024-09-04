pub trait NestedMocks<TMock> {
	fn with_mock(self, configure_mock_fn: impl FnMut(&mut TMock)) -> Self;
}
