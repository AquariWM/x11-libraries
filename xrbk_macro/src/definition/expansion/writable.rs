// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
	definition::{Definition, DefinitionType, Event, Metadata, Reply, Request, Struct},
	element::{Content, Element},
	ext::TsExt,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

impl Definition {
	pub fn impl_writable(&self, tokens: &mut TokenStream2) {
		match self {
			Self::Structlike(metadata, content, ..) => {
				metadata.impl_writable(tokens, content);
			},

			Self::Enum(_enum) => todo!(),

			Self::Other(_) => {},
		}
	}
}

impl Metadata {
	pub fn impl_writable(&self, tokens: &mut TokenStream2, content: &Content) {
		match self {
			Self::Struct(r#struct) => r#struct.impl_writable(tokens, content),

			Self::Request(request) => request.impl_writable(tokens, content),
			Self::Reply(reply) => reply.impl_writable(tokens, content),
			Self::Event(event) => event.impl_writable(tokens, content),
		}
	}
}

impl Struct {
	pub fn impl_writable(&self, tokens: &mut TokenStream2, content: &Content) {
		let ident = &self.ident;

		// TODO: add generic bounds
		let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

		let declare_datasize = if content.contains_infer() {
			Some(quote!(let mut datasize: usize = 0;))
		} else {
			None
		};

		let pat = TokenStream2::with_tokens(|tokens| {
			content.fields_to_tokens(tokens);
		});

		let writes = TokenStream2::with_tokens(|tokens| {
			for element in content {
				element.serialize(tokens, DefinitionType::Basic);

				if content.contains_infer() {
					element.add_datasize_tokens(tokens);
				}
			}
		});

		tokens.append_tokens(|| {
			quote!(
				impl #impl_generics cornflakes::Writable for #ident #type_generics #where_clause {
					fn write_to(
						&self,
						// TODO: re-export `Buf` and `BufMut` in `cornflakes`
						buf: &mut impl bytes::BufMut,
					) -> Result<(), cornflakes::WriteError> {
						#declare_datasize
						// Destructure the struct's fields, if any.
						let Self #pat = self;

						#writes

						Ok(())
					}
				}
			)
		});
	}
}

impl Request {
	pub fn impl_writable(&self, tokens: &mut TokenStream2, content: &Content) {
		let ident = &self.ident;

		// TODO: add generic bounds
		let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

		let declare_datasize = if content.contains_infer() {
			// The datasize starts at `4` to account for the size of a request's header
			// being 4 bytes.
			Some(quote!(let mut datasize: usize = 4;))
		} else {
			None
		};

		let pat = TokenStream2::with_tokens(|tokens| {
			content.fields_to_tokens(tokens);
		});

		let writes = TokenStream2::with_tokens(|tokens| {
			for element in content {
				if !element.is_metabyte() && !element.is_sequence() {
					element.serialize(tokens, DefinitionType::Request);

					if content.contains_infer() {
						element.add_datasize_tokens(tokens);
					}
				}
			}
		});

		let metabyte = if self.minor_opcode.is_some() {
			quote!(
				buf.put_u8(<Self as xrb::Request>::minor_opcode().unwrap());
			)
		} else if let Some(element) = content.metabyte_element() {
			TokenStream2::with_tokens(|tokens| {
				element.serialize(tokens, DefinitionType::Request);
			})
		} else {
			quote!(
				buf.put_u8(0);
			)
		};

		tokens.append_tokens(|| {
			quote!(
				impl #impl_generics cornflakes::Writable for #ident #type_generics #where_clause {
					fn write_to(
						&self,
						// TODO: re-export `Buf` and `BufMut` in `cornflakes`
						buf: &mut impl bytes::BufMut,
					) -> Result<(), cornflakes::WriteError> {
						#declare_datasize
						// Destructure the request struct's fields, if any.
						let Self #pat = self;

						// Major opcode
						buf.put_u8(<Self as xrb::Request>::major_opcode());
						// Metabyte position
						#metabyte
						// Length
						buf.put_u16(<Self as xrb::Request>::length(&self));

						// Other elements
						#writes

						Ok(())
					}
				}
			)
		});
	}
}

impl Reply {
	pub fn impl_writable(&self, tokens: &mut TokenStream2, content: &Content) {
		let ident = &self.ident;

		// TODO: add generic bounds
		let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

		let declare_datasize = if content.contains_infer() {
			Some(quote!(let mut datasize: usize = 8;))
		} else {
			None
		};

		let pat = TokenStream2::with_tokens(|tokens| {
			content.fields_to_tokens(tokens);
		});

		let writes = TokenStream2::with_tokens(|tokens| {
			for element in content {
				if !element.is_metabyte() && !element.is_sequence() {
					element.serialize(tokens, DefinitionType::Reply);

					if content.contains_infer() {
						element.add_datasize_tokens(tokens);
					}
				}
			}
		});

		let metabyte = if let Some(element) = content.metabyte_element() {
			TokenStream2::with_tokens(|tokens| {
				element.serialize(tokens, DefinitionType::Reply);
			})
		} else {
			quote!(
				buf.put_u8(0);
			)
		};

		let sequence = match content.sequence_element() {
			Some(Element::Field(field)) => &field.formatted,
			_ => panic!("replies must have a sequence field"),
		};

		tokens.append_tokens(|| {
			quote!(
				impl #impl_generics cornflakes::Writable for #ident #type_generics #where_clause {
					fn write_to(
						&self,
						// TODO: re-export `Buf` and `BufMut` in `cornflakes`
						buf: &mut impl bytes::BufMut,
					) -> Result<(), cornflakes::WriteError> {
						#declare_datasize
						// Destructure the reply struct's fields, if any.
						let Self #pat = self;

						// `1` - indicates this is a reply
						buf.put_u8(1);
						// Metabyte position
						#metabyte
						// Sequence field
						#sequence
						// Length
						buf.put_u32(<Self as xrb::Reply>::length(&self));

						// Other elements
						#writes

						Ok(())
					}
				}
			)
		});
	}
}

impl Event {
	pub fn impl_writable(&self, tokens: &mut TokenStream2, content: &Content) {
		let ident = &self.ident;

		// TODO: add generic bounds
		let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

		let declare_datasize = if content.contains_infer() {
			let datasize: usize = if content.sequence_element().is_some() {
				4
			} else {
				1
			};

			Some(quote!(let mut datasize: usize = #datasize;))
		} else {
			None
		};

		let pat = TokenStream2::with_tokens(|tokens| {
			content.fields_to_tokens(tokens);
		});

		let writes = TokenStream2::with_tokens(|tokens| {
			for element in content {
				if !element.is_metabyte() && !element.is_sequence() {
					element.serialize(tokens, DefinitionType::Event);

					if content.contains_infer() {
						element.add_datasize_tokens(tokens);
					}
				}
			}
		});

		let metabyte = if content.sequence_element().is_none() {
			None
		} else if let Some(element) = content.metabyte_element() {
			Some(TokenStream2::with_tokens(|tokens| {
				element.serialize(tokens, DefinitionType::Event);
			}))
		} else {
			Some(quote!(
				buf.put_u8(0);
			))
		};

		let sequence = if let Some(Element::Field(field)) = content.sequence_element() {
			Some(&field.formatted)
		} else {
			None
		};

		tokens.append_tokens(|| {
			quote!(
				impl #impl_generics cornflakes::Writable for #ident #type_generics #where_clause {
					fn write_to(
						&self,
						// TODO: re-export `Buf` and `BufMut` in `cornflakes`
						buf: &mut impl bytes::BufMut,
					) -> Result<(), cornflakes::WriteError> {
						#declare_datasize
						// Destructure the event struct's fields, if any.
						let Self #pat = self;

						// Event code
						buf.put_u8(<Self as xrb::Event>::code());
						// Metabyte position
						#metabyte
						// Sequence field
						#sequence

						// Other elements
						#writes

						Ok(())
					}
				}
			)
		});
	}
}
