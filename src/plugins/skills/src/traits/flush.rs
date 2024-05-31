use super::{Flush, SetNextCombo};
use crate::components::combos::ComboNode;

impl<T: SetNextCombo<Option<ComboNode>>> Flush for T {
	fn flush(&mut self) {
		self.set_next_combo(None)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use mockall::{mock, predicate::eq};

	mock! {
		_Combos {}
		impl SetNextCombo<Option<ComboNode>> for _Combos {
			fn set_next_combo(&mut self, value: Option<ComboNode>);
		}
	}

	#[test]
	fn call_set_update_with_none() {
		let mut combos = Mock_Combos::default();
		combos
			.expect_set_next_combo()
			.times(1)
			.with(eq(None))
			.return_const(());
		combos.flush();
	}
}
