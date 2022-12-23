// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod expansion;
pub mod parsing;

use syn::{token, Path, Token};

use crate::Source;

/// An attribute which places an [`Element`] in the metabyte position.
///
/// > **<sup>Syntax</sup>**\
/// > _MetabyteAttribute_ :\
/// > &nbsp;&nbsp; `#` `[` `metabyte` `]`
///
/// [`Element`]: crate::element::Element
pub struct MetabyteAttribute {
	/// A hash token: `#`.
	pub hash_token: Token![#],
	/// A pair of square brackets (`[` and `]`) surrounding the `path`.
	pub bracket_token: token::Bracket,

	/// The attribute path: `metabyte` for a `MetabyteAttribute`.
	pub path: Path,
}

/// An attribute which indicates that a [`Field`] represents the sequence number
/// of a reply or event.
///
/// > **<sup>Syntax</sup>**\
/// > _SequenceAttribute_ :\
/// > &nbsp;&nbsp; `#` `[` `sequence` `]`
///
/// [`Field`]: crate::element::Field
pub struct SequenceAttribute {
	/// A hash token: `#`.
	pub hash_token: Token![#],
	/// A pair of square brackets (`[` and `]`) surrounding the `path`.
	pub bracket_token: token::Bracket,

	/// The attribute path: `sequence` for a `SequenceAttribute`.
	pub path: Path,
}

/// An attribute which provides the [`ContextualReadable::Context`] for a type
/// implementing [`cornflakes::ContextualReadable`].
///
/// > **<sup>Syntax</sup>**\
/// > _ContextAttribute_ :\
/// > &nbsp;&nbsp; `#` `[` `context` [_Context_] `]`
/// >
/// > [_Context_]: Context
///
/// [`ContextualReadable::Context`]: https://docs.rs/cornflakes/latest/cornflakes/trait.ContextualReadable.html#associatedtype.Context
/// [`cornflakes::ContextualReadable`]: https://docs.rs/cornflakes/latest/cornflakes/trait.ContextualReadable.html
pub struct ContextAttribute {
	/// A hash token: `#`.
	pub hash_token: Token![#],
	/// A pair of square brackets (`[` and `]`) surrounding the `path`.
	pub bracket_token: token::Bracket,

	/// The attribute path: `context` for a `ContextAttribute`.
	pub path: Path,

	/// The provided context.
	pub context: Context,
}

/// The context provided within a [`ContextAttribute`].
///
/// > **<sup>Syntax</sup>**\
/// > _Context_ :\
/// > &nbsp;&nbsp; (`(` [_Source_] `)`) | (`=` [_Source_])
/// >
/// > [_Source_]: Source
pub enum Context {
	Paren {
		/// A pair of normal brackets (`(` and `)`) surrounding the [`source`].
		///
		/// [`source`]: Context::Paren::source
		paren_token: token::Paren,
		/// The [`Source`] providing the `Context`.
		source: Source,
	},

	Equals {
		/// An equals token (`=`) preceding the [`source`].
		///
		/// [`source`]: Context::Equals::source
		equals_token: Token![=],
		/// The [`Source`] providing the `Context`.
		source: Source,
	},
}

impl Context {
	pub const fn source(&self) -> &Source {
		match self {
			Self::Paren { source, .. } => source,
			Self::Equals { source, .. } => source,
		}
	}
}