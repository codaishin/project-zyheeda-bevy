use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, DeriveInput};

#[proc_macro_derive(ClampZeroPositive)]
pub fn clamp_zero_positive_derive(input: TokenStream) -> TokenStream {
	let ast: DeriveInput = parse(input).unwrap();
	let ident = ast.ident;
	let implementation = quote! {
		impl ClampZeroPositive for #ident {
			fn new(value: f32) -> Self {
				if value < 0. {
					Self(0.)
				} else {
					Self(value)
				}
			}
		}

		impl Deref for #ident {
			type Target = f32;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}
	};

	implementation.into()
}
