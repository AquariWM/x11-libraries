// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! [Replies] defined in the [core X11 protocol] for
//! [requests that relate to an X client or the X server].
//!
//! [Replies] are messages sent from the X server to an X client in response to
//! a [request].
//!
//! [Replies]: Reply
//! [request]: crate::message::Request
//! [core X11 protocol]: crate::x11
//!
//! [requests that relate to an X client or the X server]: request::meta

extern crate self as xrb;

use derivative::Derivative;
use xrbk::pad;
use xrbk_macro::derive_xrb;

use crate::{message::Reply, x11::request, LengthString8};

derive_xrb! {
	/// The [reply] to a [`QueryExtension` request].
	///
	/// [reply]: Reply
	///
	/// [`QueryExtension` request]: request::QueryExtension
	#[derive(Derivative, Debug, X11Size, Readable, Writable)]
	#[derivative(Hash, PartialEq, Eq)]
	pub struct QueryExtension: Reply for request::QueryExtension {
		/// The sequence number identifying the [request] that generated this
		/// [reply].
		///
		/// See [`Reply::sequence`] for more information.
		///
		/// [request]: crate::message::Request
		/// [reply]: Reply
		///
		/// [`Reply::sequence`]: Reply::sequence
		#[sequence]
		#[derivative(Hash = "ignore", PartialEq = "ignore")]
		pub sequence: u16,

		/// Whether the specified extension is present.
		pub present: bool,

		/// The [major opcode] of the specified extension if the extension is
		/// present and it has has a [major opcode].
		///
		/// [major opcode]: crate::message::Request::MAJOR_OPCODE
		pub major_opcode: Option<u8>,
		/// The first [event code] defined by the specified extension if the
		/// extension is present and it defines any [events].
		///
		/// [events]: crate::message::Event
		/// [event code]: crate::message::Event::CODE
		pub first_event_code: Option<u8>,
		/// The first [error code] defined by the specified extension if the
		/// extension is present and it defines any [errors].
		///
		/// [errors]: crate::message::Error
		/// [event code]: crate::message::Event::CODE
		pub first_error_code: Option<u8>,
	}

	/// The [reply] to a [`ListExtensions` request].
	///
	/// [reply]: Reply
	///
	/// [`ListExtensions` request]: request::ListExtensions
	#[derive(Derivative, Debug, X11Size, Readable, Writable)]
	#[derivative(Hash, PartialEq, Eq)]
	pub struct ListExtensions: Reply for request::ListExtensions {
		/// The sequence number identifying the [request] that generated this
		/// [reply].
		///
		/// See [`Reply::sequence`] for more information.
		///
		/// [request]: crate::message::Request
		/// [reply]: Reply
		///
		/// [`Reply::sequence`]: Reply::sequence
		#[sequence]
		#[derivative(Hash = "ignore", PartialEq = "ignore")]
		pub sequence: u16,

		// The length of `names`.
		#[metabyte]
		#[allow(clippy::cast_possible_truncation)]
		let names_len: u8 = names => names.len() as u8,

		[_; 24],

		/// The names of all extensions supported by the X server.
		#[context(names_len => usize::from(*names_len))]
		pub names: Vec<LengthString8>,
		[_; names => pad(names)],
	}
}
