use crate::components::{Dad, DadPanel, Swap};

impl<T1, T2> From<(Dad<T1>, DadPanel<T2>)> for Swap<T1, T2> {
	fn from((dad, dad_panel): (Dad<T1>, DadPanel<T2>)) -> Self {
		Swap(dad.0, dad_panel.0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from() {
		let dad = Dad(42_u32);
		let dad_panel = DadPanel(100_f64);
		let swap = Swap::from((dad, dad_panel));

		assert_eq!(Swap(42_u32, 100_f64), swap);
	}
}
