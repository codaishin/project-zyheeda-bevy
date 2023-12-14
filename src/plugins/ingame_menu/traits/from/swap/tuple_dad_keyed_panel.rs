use crate::components::{Dad, KeyedPanel, Swap};

impl<T1, T2> From<(Dad<T1>, KeyedPanel<T2>)> for Swap<T1, T2> {
	fn from((dad, keyed_panel): (Dad<T1>, KeyedPanel<T2>)) -> Self {
		Swap(dad.0, keyed_panel.0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from() {
		let dad = Dad(42_u32);
		let keyed_panel = KeyedPanel(100_f64);
		let swap = Swap::from((dad, keyed_panel));

		assert_eq!(Swap(42_u32, 100_f64), swap);
	}
}
