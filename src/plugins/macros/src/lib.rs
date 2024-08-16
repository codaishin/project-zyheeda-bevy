use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, parse_macro_input, Data, DeriveInput, Fields, Ident, Type};

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

		impl Default for #ident {
			fn default() -> Self {
				Self(0.)
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

enum SetupMockCompileError {
	NotAStruct,
	NoNamedFields,
	FaultyFields,
}

impl From<SetupMockCompileError> for TokenStream {
	fn from(value: SetupMockCompileError) -> Self {
		match value {
			SetupMockCompileError::NotAStruct => TokenStream::from(quote! {
				compile_error!("SetupMock can only be derived for structs");
			}),
			SetupMockCompileError::NoNamedFields => TokenStream::from(quote! {
				compile_error!("SetupMock can only be derived for structs with named fields");
			}),
			SetupMockCompileError::FaultyFields => TokenStream::from(quote! {
				compile_error!("SetupMock can only be derived for structs with a single field named 'mock'");
			}),
		}
	}
}

#[proc_macro_derive(NestedMock)]
pub fn nested_mock_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let (mock_field_ident, mock_field_type) = match get_mock_field(&input) {
		Ok(value) => value,
		Err(error) => return TokenStream::from(error),
	};
	let ident = &input.ident;

	let implementation = quote! {
		impl common::traits::nested_mock::NestedMock<#mock_field_type> for #ident {
			fn new_mock(mut configure_mock_fn: impl FnMut(&mut #mock_field_type)) -> Self {
				let mut mock = #mock_field_type::default();
				configure_mock_fn(&mut mock);

				Self { #mock_field_ident: mock }
			}
		}
	};

	implementation.into()
}

fn get_mock_field(input: &DeriveInput) -> Result<(Option<&Ident>, &Type), SetupMockCompileError> {
	let Data::Struct(data_struct) = &input.data else {
		return Err(SetupMockCompileError::NotAStruct);
	};

	let Fields::Named(ref fields) = data_struct.fields else {
		return Err(SetupMockCompileError::NoNamedFields);
	};

	let fields = fields.named.iter().collect::<Vec<_>>();

	let [field] = fields[..] else {
		return Err(SetupMockCompileError::FaultyFields);
	};

	match &field.ident {
		Some(field_ident) if field_ident == "mock" => Ok((Some(field_ident), &field.ty)),
		_ => Err(SetupMockCompileError::FaultyFields),
	}
}
