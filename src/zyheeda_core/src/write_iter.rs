#[macro_export]
macro_rules! write_iter {
	($fmt:expr, $iter:expr) => {{
		write!($fmt, "[")?;
		let mut iter = $iter.iter();
		if let Some(first) = iter.next() {
			write!($fmt, "{first}")?;
		}
		for item in iter {
			write!($fmt, ", {item}")?;
		}
		write!($fmt, "]")
	}};
	($fmt:expr, $label:literal, $iter:expr) => {{
		write!($fmt, $label)?;
		write_iter!($fmt, $iter)
	}};
}

pub use write_iter;

#[cfg(test)]
mod tests {
	use super::*;
	use std::fmt::Display;

	mod without_label {
		use super::*;

		struct _Items<const N: usize>([&'static str; N]);

		impl<const N: usize> Display for _Items<N> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write_iter!(f, self.0)
			}
		}

		#[test]
		fn write_empty() {
			let items = _Items([]);

			let str = items.to_string();

			assert_eq!("[]", str);
		}

		#[test]
		fn write_one_item() {
			let items = _Items(["a"]);

			let str = items.to_string();

			assert_eq!("[a]", str);
		}

		#[test]
		fn write_multiple_items() {
			let items = _Items(["a", "b", "c"]);

			let str = items.to_string();

			assert_eq!("[a, b, c]", str);
		}
	}

	mod with_label {
		use super::*;

		struct _Items<const N: usize>([&'static str; N]);

		impl<const N: usize> Display for _Items<N> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write_iter!(f, "items: ", self.0)
			}
		}

		#[test]
		fn write_empty() {
			let items = _Items([]);

			let str = items.to_string();

			assert_eq!("items: []", str);
		}

		#[test]
		fn write_one_item() {
			let items = _Items(["a"]);

			let str = items.to_string();

			assert_eq!("items: [a]", str);
		}

		#[test]
		fn write_multiple_items() {
			let items = _Items(["a", "b", "c"]);

			let str = items.to_string();

			assert_eq!("items: [a, b, c]", str);
		}
	}
}
