pub trait Set<TKey, TValue> {
	fn set(&mut self, key: TKey, value: TValue);
}

pub trait Setter<TValue> {
	fn set(&mut self, value: TValue);
}

impl<TValue, T> Setter<TValue> for T
where
	T: Set<(), TValue>,
{
	fn set(&mut self, value: TValue) {
		self.set((), value);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{simple_init, traits::mock::Mock};
	use mockall::{mock, predicate::eq};

	mock! {
		_T {}
		impl Set<(), u32> for _T {
			fn set(&mut self, key: (), value: u32);
		}
	}

	simple_init!(Mock_T);

	fn as_setter(v: Mock_T) -> impl Setter<u32> {
		v
	}

	#[test]
	fn call_set() {
		let mut mock = as_setter(Mock_T::new_mock(|mock| {
			mock.expect_set()
				.times(1)
				.with(eq(()), eq(42))
				.return_const(());
		}));

		mock.set(42);
	}
}
