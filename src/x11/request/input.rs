// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! [Requests] defined in the [core X11 protocol] that relate to input devices,
//! grabs, and coordinates.
//!
//! [Requests] are messages sent from an X client to the X server.
//!
//! [Requests]: Request
//! [core X11 protocol]: crate::x11

extern crate self as xrb;

use xrbk::{Buf, BufMut, ConstantX11Size, ReadResult, Readable, Writable, WriteResult, X11Size};
use xrbk_macro::{derive_xrb, Readable, Writable, X11Size};

use crate::{
	message::Request,
	x11::{error, reply},
	Any,
	AnyModifierKeyMask,
	Button,
	Coords,
	CurrentableTime,
	CursorAppearance,
	CursorEventMask,
	FocusWindow,
	FreezeMode,
	Keycode,
	Window,
};

macro_rules! request_error {
	(
		$(#[$meta:meta])*
		$vis:vis enum $Name:ident for $Request:ty {
			$($($Error:ident),+$(,)?)?
		}
	) => {
		#[doc = concat!(
			"An [error](crate::message::Error) generated because of a failed [`",
			stringify!($Request),
			"` request](",
			stringify!($Request),
			")."
		)]
		#[doc = ""]
		$(#[$meta])*
		$vis enum $Name {
			$($(
				#[doc = concat!(
					"A [`",
					stringify!($Error),
					"` error](error::",
					stringify!($Error),
					")."
				)]
				$Error(error::$Error)
			),+)?
		}
	};
}

request_error! {
	pub enum GrabCursorError for GrabCursor {
		CursorAppearance,
		Value,
		Window,
	}
}

derive_xrb! {
	/// A [request] that actively grabs control of the cursor.
	///
	/// This [request] generates [`EnterWindow`] and [`LeaveWindow`] events.
	///
	/// # Replies
	/// This [request] generates a [`GrabCursor` reply].
	///
	/// # Errors
	/// A [`Window` error] is generated if either the `grab_window` or the
	/// `confine_to` [window] do not refer to defined [windows][window].
	///
	/// A [`CursorAppearance` error] is generated if the `cursor_appearance` is
	/// [`Some`] and does not refer to a defined [cursor appearance].
	///
	/// [cursor appearance]: CursorAppearance
	/// [window]: Window
	/// [request]: Request
	///
	/// [`EnterWindow`]: crate::x11::event::EnterWindow
	/// [`LeaveWindow`]: crate::x11::event::LeaveWindow
	/// [`GrabCursor` reply]: reply::GrabCursor
	///
	/// [`Window` error]: error::Window
	/// [`CursorAppearance` error]: error::CursorAppearance
	#[doc(alias = "GrabPointer")]
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct GrabCursor: Request(26, GrabCursorError) -> reply::GrabCursor {
		/// Whether cursor [events] which would normally be reported to this
		/// client are reported normally.
		///
		/// [events]: crate::message::Event
		#[metabyte]
		pub owner_events: bool,

		/// The [window] on which the cursor is grabbed.
		///
		/// # Errors
		/// A [`Window` error] is generated if this does not refer to a defined
		/// [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		pub grab_window: Window,

		/// A mask of the cursor [events] which are to be reported to the
		/// your client.
		///
		/// [events]: crate::message::Event
		pub event_mask: CursorEventMask,

		/// The [freeze mode] applied to the cursor.
		///
		/// For [`FreezeMode::Unfrozen`], cursor [event] processing continues
		/// as normal.
		///
		/// For [`FreezeMode::Frozen`], cursor [event] processing appears to
		/// freeze - cursor [events][event] generated during this time are not
		/// lost: they are queued to be processed later. The freeze ends when
		/// either the grabbing client sends an [`AllowEvents` request], or when
		/// the cursor grab is released.
		///
		/// [event]: crate::message::Event
		/// [freeze mode]: FreezeMode
		///
		/// [`AllowEvents` request]: AllowEvents
		#[doc(alias("pointer_mode", "cursor_mode"))]
		pub cursor_freeze: FreezeMode,
		/// The [freeze mode] applied to the keyboard.
		///
		/// For [`FreezeMode::Unfrozen`], keyboard [event] processing
		/// continues as normal.
		///
		/// For [`FreezeMode::Frozen`], keyboard [event] processing appears
		/// to freeze - keyboard [events][event] generated during this time are
		/// not lost: they are queued to be processed later. The freeze ends
		/// when either the grabbing client sends an [`AllowEvents` request], or
		/// when the keyboard grab is released.
		///
		/// [event]: crate::message::Event
		/// [freeze mode]: FreezeMode
		///
		/// [`AllowEvents` request]: AllowEvents
		#[doc(alias = "keyboard_mode")]
		pub keyboard_freeze: FreezeMode,

		/// Optionally confines the cursor to the given [window].
		///
		/// This [window] does not need to have any relation to the
		/// `grab_window`.
		///
		/// The cursor will be warped to the closest edge of this [window] if it
		/// is not already within it. Subsequent changes to the configuration of
		/// the [window] which cause the cursor to be outside of the [window]
		/// will also trigger the cursor to be warped to the [window] again.
		///
		/// # Errors
		/// A [`Window` error] is generated if this is [`Some`] and does not
		/// refer to a defined [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		pub confine_to: Option<Window>,

		/// Optionally overrides the [appearance of the cursor], no matter which
		/// [window] it is within, for the duration of the grab.
		///
		/// # Errors
		/// A [`CursorAppearance` error] is generated if this does not refer to
		/// a defined [cursor appearance].
		///
		/// [cursor appearance]: CursorAppearance
		/// [appearance of the cursor]: CursorAppearance
		/// [window]: Window
		///
		/// [`CursorAppearance` error]: error::CursorAppearance
		#[doc(alias = "cursor")]
		pub cursor_appearance: Option<CursorAppearance>,

		/// The [time] at which this grab is recorded as having been initiated.
		///
		/// [time]: crate::Timestamp
		pub time: CurrentableTime,
	}

	/// A [request] that ends an active cursor grab by your client.
	///
	/// Any queued [events] are released.
	///
	/// This [request] generates [`EnterWindow`] and [`LeaveWindow`] events.
	///
	/// [request]: Request
	/// [events]: crate::message::Event
	///
	/// [`EnterWindow`]: crate::x11::event::EnterWindow
	/// [`LeaveWindow`]: crate::x11::event::LeaveWindow
	#[doc(alias = "UngrabPointer")]
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct UngrabCursor: Request(27) {
		/// The [time] at which the grab is recorded as having been released.
		///
		/// [time]: crate::Timestamp
		pub time: CurrentableTime,
	}
}

request_error! {
	pub enum GrabButtonError for GrabButton {
		Access,
		CursorAppearance,
		Value,
		Window,
	}
}

derive_xrb! {
	/// A [request] that establishes a passive cursor grab for a given `button`
	/// and `modifiers` combination.
	///
	/// If the following conditions are true, the grab is converted into an
	/// active cursor grab (as described in the [`GrabCursor` request]):
	/// - the cursor is not already actively grabbed; and
	/// - the specified `button` and specified `modifiers` are held; and
	/// - the cursor is within the `grab_window`; and
	/// - if the `confine_to` [window] is specified, it is viewable; and
	/// - a passive grab for the same `button` and `modifiers` combination does
	///   not exist for any ancestor of the `grab_window`.
	///
	/// # Errors
	/// A [`Window` error] is generated if either the `grab_window` or the
	/// `confine_to` [window] do not refer to defined [windows][window].
	///
	/// A [`CursorAppearance` error] is generated if the `cursor_appearance` is
	/// [`Some`] and does not refer to a defined [cursor appearance].
	///
	/// An [`Access` error] is generated if some other client has already sent a
	/// `GrabButton` [request] with the same `button` and `modifiers`
	/// combination on the same `grab_window`.
	///
	/// [cursor appearance]: CursorAppearance
	/// [window]: Window
	/// [request]: Request
	///
	/// [`GrabCursor` request]: GrabCursor
	///
	/// [`Access` error]: error::Access
	/// [`Window` error]: error::Window
	/// [`CursorAppearance` error]: error::CursorAppearance
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct GrabButton: Request(28, GrabButtonError) {
		/// Whether cursor [events] which would normally be reported to this
		/// client are reported normally.
		///
		/// [events]: crate::message::Event
		#[metabyte]
		pub owner_events: bool,

		/// The [window] on which the `button` is grabbed.
		///
		/// # Errors
		/// A [`Window` error] is generated if this does not refer to a defined
		/// [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		pub grab_window: Window,

		/// A mask of the cursor [events] which are to be reported to the
		/// grabbing client.
		///
		/// [events]: crate::message::Event
		pub event_mask: CursorEventMask,

		/// The [freeze mode] applied to the cursor.
		///
		/// For [`FreezeMode::Unfrozen`], cursor [event] processing continues
		/// as normal.
		///
		/// For [`FreezeMode::Frozen`], cursor [event] processing appears to
		/// freeze - cursor [events][event] generated during this time are not
		/// lost: they are queued to be processed later. The freeze ends when
		/// either the grabbing client sends an [`AllowEvents` request], or when
		/// the cursor grab is released.
		///
		/// [event]: crate::message::Event
		/// [freeze mode]: FreezeMode
		///
		/// [`AllowEvents` request]: AllowEvents
		#[doc(alias("pointer_mode", "cursor_mode"))]
		pub cursor_freeze: FreezeMode,
		/// The [freeze mode] applied to the keyboard.
		///
		/// For [`FreezeMode::Unfrozen`], keyboard [event] processing
		/// continues as normal.
		///
		/// For [`FreezeMode::Frozen`], keyboard [event] processing appears
		/// to freeze - keyboard [events][event] generated during this time are
		/// not lost: they are queued to be processed later. The freeze ends
		/// when either the grabbing client sends an [`AllowEvents` request], or
		/// when the keyboard grab is released.
		///
		/// [event]: crate::message::Event
		/// [freeze mode]: FreezeMode
		///
		/// [`AllowEvents` request]: AllowEvents
		#[doc(alias = "keyboard_mode")]
		pub keyboard_freeze: FreezeMode,

		/// Optionally confines the cursor to the given [window].
		///
		/// This [window] does not need to have any relation to the
		/// `grab_window`.
		///
		/// The cursor will be warped to the closest edge of this [window] if it
		/// is not already within it. Subsequent changes to the configuration of
		/// the [window] which cause the cursor to be outside of the [window]
		/// will also trigger the cursor to be warped to the [window] again.
		///
		/// # Errors
		/// A [`Window` error] is generated if this is [`Some`] and does not
		/// refer to a defined [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		pub confine_to: Option<Window>,

		/// Optionally overrides the [appearance of the cursor], no matter which
		/// [window] it is within, for the duration of the grab.
		///
		/// # Errors
		/// A [`CursorAppearance` error] is generated if this does not refer to
		/// a defined [cursor appearance].
		///
		/// [cursor appearance]: CursorAppearance
		/// [appearance of the cursor]: CursorAppearance
		/// [window]: Window
		///
		/// [`CursorAppearance` error]: error::CursorAppearance
		pub cursor_appearance: Option<CursorAppearance>,

		/// The [button] for which this grab is established.
		///
		/// [`Any`] means that the grab is effectively established for all
		/// possible [buttons][button].
		///
		/// When this button and the given `modifiers`,
		///
		/// [button]: Button
		///
		/// [`Any`]: Any::Any
		pub button: Any<Button>,
		_,

		/// The combination of modifiers which must be held for a press of the
		/// `button` to activate the active cursor grab.
		///
		/// [`ANY_MODIFIER`] means _any_ modifiers: that includes no modifiers
		/// at all.
		///
		/// [`ANY_MODIFIER`]: AnyModifierKeyMask::ANY_MODIFIER
		pub modifiers: AnyModifierKeyMask,
	}
}

request_error! {
	pub enum UngrabButtonError for UngrabButton {
		Value,
		Window,
	}
}

derive_xrb! {
	/// A [request] that releases a [passive button grab] on the specified
	/// `grab_window` if the grab was established by your client.
	///
	/// # Errors
	/// A [`Window` error] is generated if `grab_window` does not refer to a
	/// defined [window].
	///
	/// [window]: Window
	/// [request]: Request
	///
	/// [passive button grab]: GrabButton
	///
	/// [`Window` error]: error::Window
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct UngrabButton: Request(29, UngrabButtonError) {
		/// The [button] which the [passive button grab] was established for.
		///
		/// [`Any`] matches any `button` specified in the [passive button grab].
		/// It is equivalent to sending this `UngrabButton` [request] for all
		/// possible [buttons][button].
		///
		/// [button]: Button
		/// [request]: Request
		///
		/// [passive button grab]: GrabButton
		///
		/// [`Any`]: Any::Any
		#[metabyte]
		pub button: Any<Button>,

		/// The [window] on which the [passive button grab] was established.
		///
		/// # Errors
		/// A [`Window` error] is generated if this does not refer to a defined
		/// [window].
		///
		/// [window]: Window
		///
		/// [passive button grab]: GrabButton
		///
		/// [`Window` error]: error::Window
		pub grab_window: Window,

		/// The modifier combination specified by the [passive button grab].
		///
		/// [`ANY_MODIFIER`] matches any `modifiers` specified in the
		/// [passive button grab] (including no modifiers). It is equivalent to
		/// sending this `UngrabButton` [request] for all possible `modifiers`
		/// combinations.
		///
		/// [request]: Request
		///
		/// [passive button grab]: GrabButton
		///
		/// [`ANY_MODIFIER`]: AnyModifierKeyMask::ANY_MODIFIER
		pub modifiers: AnyModifierKeyMask,
		[_; 2],
	}
}

request_error! {
	pub enum ChangeActiveCursorGrabError for ChangeActiveCursorGrab {
		CursorAppearance,
		Value,
	}
}

derive_xrb! {
	/// A [request] that modifies the `event_mask` or `cursor_appearance` of an
	/// [active cursor grab].
	///
	/// # Errors
	/// A [`CursorAppearance` error] is generated if `cursor_appearance` does
	/// not refer to a defined [cursor appearance].
	///
	/// [cursor appearance]: CursorAppearance
	/// [request]: Request
	///
	/// [active cursor grab]: GrabCursor
	///
	/// [`CursorAppearance` error]: error::CursorAppearance
	#[doc(alias = "ChangeActivePointerGrab")]
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct ChangeActiveCursorGrab: Request(30, ChangeActiveCursorGrabError) {
		/// Optionally overrides the [appearance of the cursor], no matter which
		/// [window] it is within, for the duration of the grab.
		///
		/// This replaces the previously specified `cursor_appearance` for the
		/// grab - [`None`] means that the `cursor_appearance` is no longer
		/// overridden.
		///
		/// # Errors
		/// A [`CursorAppearance` error] is generated if this does not refer to
		/// a defined [cursor appearance].
		///
		/// [cursor appearance]: CursorAppearance
		/// [appearance of the cursor]: CursorAppearance
		/// [window]: Window
		///
		/// [`CursorAppearance` error]: error::CursorAppearance
		#[doc(alias = "cursor")]
		pub cursor_appearance: Option<CursorAppearance>,

		/// The [time] at which this change is recorded as having taken place.
		///
		/// This must be later than the [time] of the last cursor grab, and
		/// equal to or earlier than the X server's [current time].
		///
		/// [time]: crate::Timestamp
		/// [current time]: CurrentableTime::CurrentTime
		pub time: CurrentableTime,

		/// A mask of the cursor [events] which are to be reported to the
		/// your client.
		///
		/// [events]: crate::message::Event
		pub event_mask: CursorEventMask,
		[_; 2],
	}
}

request_error! {
	pub enum GrabKeyboardError for GrabKeyboard {
		Value,
		Window,
	}
}

derive_xrb! {
	/// A [request] that actively grabs control of the keyboard.
	///
	/// This [request] generates [`Focus`] and [`Unfocus`] events.
	///
	/// # Replies
	/// This [request] generates a [`GrabKeyboard` reply].
	///
	/// # Errors
	/// A [`Window` error] is generated if the `grab_window` does not refer to a
	/// defined [window].
	///
	/// [window]: Window
	/// [request]: Request
	///
	/// [`Focus`]: crate::x11::event::Focus
	/// [`Unfocus`]: crate::x11::event::Unfocus
	///
	/// [`GrabKeyboard` reply]: reply::GrabKeyboard
	///
	/// [`Window` error]: error::Window
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct GrabKeyboard: Request(31, GrabKeyboardError) -> reply::GrabKeyboard {
		/// Whether key [events] which would normally be reported to this client
		/// are reported normally.
		///
		/// Both [`KeyPress`] and [`KeyRelease`] events are always reported, no
		/// matter what events you have selected.
		///
		/// [events]: crate::message::Event
		///
		/// [`KeyPress`]: crate::x11::event::KeyPress
		/// [`KeyRelease`]: crate::x11::event::KeyRelease
		#[metabyte]
		pub owner_events: bool,

		/// The [window] on which the keyboard is grabbed.
		///
		/// # Errors
		/// A [`Window` error] is generated if this does not refer to a defined
		/// [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		pub grab_window: Window,

		/// The [time] at which this grab is recorded as having been initiated.
		///
		/// [time]: crate::Timestamp
		pub time: CurrentableTime,

		/// The [freeze mode] applied to the cursor.
		///
		/// For [`FreezeMode::Unfrozen`], cursor [event] processing continues
		/// as normal.
		///
		/// For [`FreezeMode::Frozen`], cursor [event] processing appears to
		/// freeze - cursor [events][event] generated during this time are not
		/// lost: they are queued to be processed later. The freeze ends when
		/// either the grabbing client sends an [`AllowEvents` request], or when
		/// the cursor grab is released.
		///
		/// [event]: crate::message::Event
		/// [freeze mode]: FreezeMode
		///
		/// [`AllowEvents` request]: AllowEvents
		#[doc(alias("pointer_mode", "cursor_mode"))]
		pub cursor_freeze: FreezeMode,
		/// The [freeze mode] applied to the keyboard.
		///
		/// For [`FreezeMode::Unfrozen`], keyboard [event] processing
		/// continues as normal.
		///
		/// For [`FreezeMode::Frozen`], keyboard [event] processing appears
		/// to freeze - keyboard [events][event] generated during this time are
		/// not lost: they are queued to be processed later. The freeze ends
		/// when either the grabbing client sends an [`AllowEvents` request], or
		/// when the keyboard grab is released.
		///
		/// [event]: crate::message::Event
		/// [freeze mode]: FreezeMode
		///
		/// [`AllowEvents` request]: AllowEvents
		#[doc(alias = "keyboard_mode")]
		pub keyboard_freeze: FreezeMode,
		[_; 2],
	}

	/// A [request] that ends an active keyboard grab by your client.
	///
	/// Any queued [events] are released.
	///
	/// This [request] generates [`Focus`] and [`Unfocus`] events.
	///
	/// [request]: Request
	/// [events]: crate::message::Event
	///
	/// [`Focus`]: crate::x11::event::Focus
	/// [`Unfocus`]: crate::x11::event::Unfocus
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct UngrabKeyboard: Request(32) {
		/// The [time] at which the grab is recorded as having been released.
		///
		/// [time]: crate::Timestamp
		pub time: CurrentableTime,
	}
}

request_error! {
	pub enum GrabKeyError for GrabKey {
		Access,
		CursorAppearance,
		Value,
		Window,
	}
}

derive_xrb! {
	/// A [request] that establishes a passive key grab for a particular `key`
	/// and `modifiers` combination.
	///
	/// If the following conditions are true, the grab is converted into an
	/// active keyboard grab (as described in the [`GrabKeyboard` request]):
	/// - the keyboard is not already actively grabbed; and
	/// - the specified `key` and specified `modifiers` are held; and
	/// - either the `grab_window` is an ancestor, or is, the currently focused
	///   [window], or the `grab_window` is a descendent of the currently
	///   focused [window] and contains the cursor; and
	/// - a passive grab for the same `key` and `modifiers` combination does
	///   not exist for any ancestor of the `grab_window`.
	///
	/// # Errors
	/// A [`Window` error] is generated if the `grab_window` does not refer to a
	/// defined [window].
	///
	/// An [`Access` error] is generated if some other client has already sent a
	/// `GrabKey` [request] with the same `key` and `modifiers` combination on
	/// the same `grab_window`.
	///
	/// [window]: Window
	/// [request]: Request
	///
	/// [`GrabKeyboard` request]: GrabKeyboard
	///
	/// [`Access` error]: error::Access
	/// [`Window` error]: error::Window
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct GrabKey: Request(33, GrabKeyError) {
		/// Whether key [events] which would normally be reported to this client
		/// are reported normally.
		///
		/// Both [`KeyPress`] and [`KeyRelease`] events are always reported, no
		/// matter what events you have selected.
		///
		/// [events]: crate::message::Event
		///
		/// [`KeyPress`]: crate::x11::event::KeyPress
		/// [`KeyRelease`]: crate::x11::event::KeyRelease
		#[metabyte]
		pub owner_events: bool,

		/// The [window] on which the `key` is grabbed.
		///
		/// # Errors
		/// A [`Window` error] is generated if this does not refer to a defined
		/// [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		pub grab_window: Window,

		/// The combination of modifiers which must be held for a press of the
		/// `key` to activate the active key grab.
		///
		/// [`ANY_MODIFIER`] means _any_ modifiers: that includes no modifiers
		/// at all.
		///
		/// [`ANY_MODIFIER`]: AnyModifierKeyMask::ANY_MODIFIER
		pub modifiers: AnyModifierKeyMask,
		/// The key for which this grab is established.
		///
		/// [`Any`] means that the grab is effectively established for all
		/// possible keys.
		///
		/// When this key and the given `modifiers`,
		///
		/// [button]: Button
		///
		/// [`Any`]: Any::Any
		pub key: Any<Keycode>,

		/// The [freeze mode] applied to the cursor.
		///
		/// For [`FreezeMode::Unfrozen`], cursor [event] processing continues
		/// as normal.
		///
		/// For [`FreezeMode::Frozen`], cursor [event] processing appears to
		/// freeze - cursor [events][event] generated during this time are not
		/// lost: they are queued to be processed later. The freeze ends when
		/// either the grabbing client sends an [`AllowEvents` request], or when
		/// the cursor grab is released.
		///
		/// [event]: crate::message::Event
		/// [freeze mode]: FreezeMode
		///
		/// [`AllowEvents` request]: AllowEvents
		#[doc(alias("pointer_mode", "cursor_mode"))]
		pub cursor_freeze: FreezeMode,
		/// The [freeze mode] applied to the keyboard.
		///
		/// For [`FreezeMode::Unfrozen`], keyboard [event] processing
		/// continues as normal.
		///
		/// For [`FreezeMode::Frozen`], keyboard [event] processing appears
		/// to freeze - keyboard [events][event] generated during this time are
		/// not lost: they are queued to be processed later. The freeze ends
		/// when either the grabbing client sends an [`AllowEvents` request], or
		/// when the keyboard grab is released.
		///
		/// [event]: crate::message::Event
		/// [freeze mode]: FreezeMode
		///
		/// [`AllowEvents` request]: AllowEvents
		#[doc(alias = "keyboard_mode")]
		pub keyboard_freeze: FreezeMode,
		[_; 3],
	}
}

request_error! {
	pub enum UngrabKeyError for UngrabKey {
		Value,
		Window,
	}
}

derive_xrb! {
	/// A [request] that releases a [passive key grab] on the specified
	/// `grab_window` if the grab was established by your client.
	///
	/// # Errors
	/// A [`Window` error] is generated if `grab_window` does not refer to a
	/// defined [window].
	///
	/// [window]: Window
	/// [request]: Request
	///
	/// [passive key grab]: GrabKey
	///
	/// [`Window` error]: error::Window
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct UngrabKey: Request(34, UngrabKeyError) {
		/// The key which the [passive key grab] was established for.
		///
		/// [`Any`] matches any `key` specified in the [passive key grab].
		/// It is equivalent to sending this `UngrabKey` [request] for all
		/// possible keys.
		///
		/// [request]: Request
		///
		/// [passive key grab]: GrabKey
		///
		/// [`Any`]: Any::Any
		#[metabyte]
		pub key: Any<Keycode>,

		/// The [window] on which the [passive key grab] was established.
		///
		/// # Errors
		/// A [`Window` error] is generated if this does not refer to a defined
		/// [window].
		///
		/// [window]: Window
		///
		/// [passive key grab]: GrabKey
		///
		/// [`Window` error]: error::Window
		pub grab_window: Window,

		/// The modifier combination specified by the [passive key grab].
		///
		/// [`ANY_MODIFIER`] matches any `modifiers` specified in the
		/// [passive key grab] (including no modifiers). It is equivalent to
		/// sending this `UngrabKey` [request] for all possible `modifiers`
		/// combinations.
		///
		/// [request]: Request
		///
		/// [passive key grab]: GrabKey
		///
		/// [`ANY_MODIFIER`]: AnyModifierKeyMask::ANY_MODIFIER
		pub modifiers: AnyModifierKeyMask,
		[_; 2],
	}
}

/// Specifies the conditions under which queued events should be released for an
/// [`AllowEvents` request].
///
/// [`AllowEvents` request]: AllowEvents
#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable)]
pub enum AllowEventsMode {
	/// Unfreezes the cursor if it is frozen and you have active grab on the
	/// cursor.
	UnfreezeCursor,
	/// Unfreezes the cursor, but freezes it again after the next
	/// [`ButtonPress`] or [`ButtonRelease`].
	///
	/// Your client must have an active grab on the cursor.
	///
	/// The cursor is frozen again specifically after the next [`ButtonPress`]
	/// [`ButtonRelease`] event reported to your client which does not cause
	/// grab to be released.
	///
	/// [`ButtonPress`]: crate::x11::event::ButtonPress
	/// [`ButtonRelease`]: crate::x11::event::ButtonRelease
	RefreezeCursor,
	/// If the cursor is frozen as a result of the activation of a passive grab
	/// or [`RefreezeCursor`] mode from your client, the grab is released and
	/// the [event] is completely reprocessed.
	///
	/// [`RefreezeCursor`]: AllowEventsMode::RefreezeCursor
	///
	/// [event]: crate::message::Event
	ReplayCursor,

	/// Unfreezes the keyboard if it is frozen and you have an active grab on
	/// the keyboard.
	UnfreezeKeyboard,
	/// Unfreezes the keyboard, but freezes it again after the next
	/// [`KeyPress`] or [`KeyPress`].
	///
	/// Your client must have an active grab on the keyboard.
	///
	/// The keyboard is frozen again specifically after the next [`KeyPress`]
	/// [`KeyRelease`] event reported to your client which does not cause
	/// grab to be released.
	///
	/// [`KeyPress`]: crate::x11::event::KeyPress
	/// [`KeyRelease`]: crate::x11::event::KeyRelease
	RefreezeKeyboard,
	/// If the keyboard is frozen as a result of the activation of a passive
	/// grab or [`RefreezeKeyboard`] mode from your client, the grab is released
	/// and the [event] is completely reprocessed.
	///
	/// [`RefreezeKeyboard`]: AllowEventsMode::RefreezeKeyboard
	///
	/// [event]: crate::message::Event
	ReplayKeyboard,

	/// If both the cursor and the keyboard are frozen by your client, both are
	/// unfrozen.
	UnfreezeBoth,
	/// If both the cursor and the keyboard are frozen by your client, both are
	/// unfrozen but are both frozen again on the next button or key press or
	/// release event.
	///
	/// Any [`ButtonPress`], [`ButtonRelease`], [`KeyPress`], or [`KeyRelease`]
	/// event reported to your client will unfreeze both the cursor and the
	/// keyboard.
	///
	/// [`ButtonPress`]: crate::x11::event::ButtonPress
	/// [`ButtonRelease`]: crate::x11::event::ButtonRelease
	///
	/// [`KeyPress`]: crate::x11::event::KeyPress
	/// [`KeyRelease`]: crate::x11::event::KeyRelease
	RefreezeBoth,
}

derive_xrb! {
	/// A [request] that releases some queued events if your client has caused a
	/// device to be [frozen].
	///
	/// [frozen]: FreezeMode::Frozen
	/// [request]: Request
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct AllowEvents: Request(35, error::Value) {
		/// The conditions under which the queued [events] are released.
		///
		/// [events]: crate::message::Event
		#[metabyte]
		pub mode: AllowEventsMode,

		/// The [time] at which this `AllowEvents` [request] is recorded as
		/// having taken place.
		///
		/// This [request] has no effect if this time is earlier than the time
		/// of your most recent active grab or later than the X server's
		/// [current time].
		///
		/// [request]: Request
		/// [time]: crate::Timestamp
		/// [current time]: CurrentableTime::CurrentTime
		pub time: CurrentableTime,
	}

	/// A [request] that freezes processing of [requests][request] and
	/// connection closes on all other clients' connections.
	///
	/// [request]: Request
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct GrabServer: Request(36);

	/// A [request] that unfreezes processing of [requests][request] and
	/// connection closes on all other clients' connections.
	///
	/// [request]: Request
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct UngrabServer: Request(37);

	/// A [request] that gets the current location of the cursor.
	///
	/// # Errors
	/// A [`Window` error] is generated if the `target` does not refer to a
	/// defined [window].
	///
	/// [window]: Window
	/// [request]: Request
	///
	/// [`Window` error]: error::Window
	#[doc(alias("QueryPointer, QueryCursor, GetCursorPos, GetCursorLocation"))]
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct QueryCursorLocation: Request(38, error::Window) -> reply::QueryCursorLocation {
		/// Specifies a [window] to receive relative coordinates of the cursor
		/// in relation to, if the cursor is on the same screen.
		///
		/// # Errors
		/// A [`Window` error] is generated if this does not refer to a defined
		/// [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		#[doc(alias = "window")]
		pub target: Window,
	}

	/// A [request] that returns the recorded cursor motion between the given
	/// `start` and `end` times.
	///
	/// The `start` and `end` times are inclusive.
	///
	/// # Errors
	/// A [`Window` error] is generated if `target` does not refer to a defined
	/// [window].
	///
	/// [window]: Window
	/// [request]: Request
	///
	/// [`Window` error]: error::Window
	#[doc(alias = "GetMotionEvents")]
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct GetMotionHistory: Request(39, error::Window) -> reply::GetMotionHistory {
		/// The [window] for which the motion history is returned.
		///
		/// # Errors
		/// A [`Window` error] is generated if this does not refer to a defined
		/// [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		pub target: Window,

		/// The start of the time period for which motion events are returned.
		///
		/// This is inclusive.
		pub start: CurrentableTime,
		/// The end of the time period for which motion events are returned.
		///
		/// This is inclusive.
		pub end: CurrentableTime,
	}
}

derive_xrb! {
	/// A [request] that converts coordinates relative to the given `original`
	/// [window] to `output_coords` relative to the given `output` [window].
	///
	/// # Errors
	/// A [`Window` error] is generated if either `original` or `output` do not
	/// refer to defined [windows][window].
	///
	/// [window]: Window
	/// [request]: Request
	///
	/// [`Window` error]: error::Window
	#[doc(alias = "TranslateCoordinates")]
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct ConvertCoordinates: Request(40, error::Window) -> reply::ConvertCoordinates {
		/// The [window] which the `original_coords` are relative to.
		///
		/// # Errors
		/// A [`Window` error] is generated if this does not refer to a defined
		/// [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		#[doc(alias("src_window", "source", "input"))]
		pub original: Window,
		/// The [window] which the `output_coords` will be relative to.
		///
		/// The `original_coords` are converted to coordinates relative to the
		/// top-left corner of this [window].
		///
		/// # Errors
		/// A [`Window` error] is generated if this does not refer to a defined
		/// [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		#[doc(alias("dst_window", "destination"))]
		pub output: Window,

		/// The coordinates, relative to the `original` [window]'s top-left
		/// corner, which will be converted.
		///
		/// These coordinates will be converted such that the `output_coords`
		/// are relative to the `output` [window].
		///
		/// [window]: Window
		pub original_coords: Coords,
	}
}

/// Represents dimensions within the `source` [window] of a
/// [`WarpCursor` request].
///
/// [window]: Window
///
/// [`WarpCursor` request]: WarpCursor
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum WarpSourceDimension {
	/// Set the `source_width` to the width of the `source` [window] minus the x
	/// coordinate or the `source_height` to the height of the `source` [window]
	/// minus the y coordinate.
	///
	/// [window]: Window
	FillRemaining,
	/// This specific width or height.
	Other(u16),
}

impl ConstantX11Size for WarpSourceDimension {
	const X11_SIZE: usize = 2;
}

impl X11Size for WarpSourceDimension {
	fn x11_size(&self) -> usize {
		Self::X11_SIZE
	}
}

impl Readable for WarpSourceDimension {
	fn read_from(buf: &mut impl Buf) -> ReadResult<Self>
	where
		Self: Sized,
	{
		Ok(match buf.get_u16() {
			zero if zero == 0 => Self::FillRemaining,
			other => Self::Other(other),
		})
	}
}

impl Writable for WarpSourceDimension {
	fn write_to(&self, buf: &mut impl BufMut) -> WriteResult {
		match self {
			Self::FillRemaining => buf.put_u16(0),
			Self::Other(other) => other.write_to(buf)?,
		}

		Ok(())
	}
}

derive_xrb! {
	/// A [request] that instantly moves the cursor to a new location.
	///
	/// # Errors
	/// A [`Window` error] is generated if either the `source` or the `destination` are [`Some`] and
	/// do not refer to defined [windows].
	///
	/// [windows]: Window
	/// [request]: Request
	///
	/// [`Window` error]: error::Window
	#[doc(alias = "WarpPointer")]
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct WarpCursor: Request(41, error::Window) {
		/// The [window] which the cursor is being warped from.
		///
		/// # Errors
		/// A [`Window` error] is generated if this is [`Some`] but does not
		/// refer to a defined [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		#[doc(alias("src", "src_window"))]
		pub source: Option<Window>,
		/// The [window] which the cursor is being warped to.
		///
		/// If this is [`None`], the cursor is simply offset by the `coords`. If
		/// this is [`Some`], the cursor is set to the `coords` relative to this
		/// [window].
		///
		/// # Errors
		/// A [`Window` error] is generated if this is [`Some`] but does not
		/// refer to a defined [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		#[doc(alias("dst", "dst_window"))]
		pub destination: Option<Window>,

		/// The coordinates of the top-left corner of the rectangular area
		/// within the `source` [window] which the cursor must be within for it
		/// to be warped.
		///
		/// [window]: Window
		#[doc(alias("src_coords", "src_x", "src_y", "source_x", "source_y"))]
		pub source_coords: Coords,
		/// The width of the rectangular area within the `source` [window] which
		/// the cursor must be within for it to be warped.
		///
		/// [window]: Window
		#[doc(alias = "src_width")]
		pub source_width: WarpSourceDimension,
		/// The height of the rectangular area within the `source` [window]
		/// which the cursor must be within for it to be warped.
		///
		/// [window]: Window
		#[doc(alias = "src_height")]
		pub source_height: WarpSourceDimension,

		/// The coordinates applied to the cursor.
		///
		/// If `destination` is [`None`], the cursor is offset by these
		/// coordinates. Otherwise, the cursor is moved to these coordinates
		/// relative to the `destination` [window].
		///
		/// [window]: Window
		#[doc(alias("dst_x", "dst_y", "dst_coords", "destination_coords"))]
		pub coords: Coords,
	}
}

request_error! {
	pub enum SetFocusError for SetFocus {
		Match,
		Value,
		Window,
	}
}

/// What the focus should revert to if the focused [window] becomes unviewable.
///
/// This is used in the [`SetFocus` request].
///
/// [window]: Window
///
/// [`SetFocus` request]: SetFocus
#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable)]
pub enum RevertFocus {
	/// Revert the focus to no [window].
	///
	/// It is recommended to use [`CursorRoot`] in place of this, because at
	/// least the root [window] will have focus with [`CursorRoot`].
	///
	/// [`CursorRoot`]: RevertFocus::CursorRoot
	///
	/// [window]: Window
	None,

	/// Revert the focus to the root [window] which the cursor is on at the
	/// time.
	///
	/// [window]: Window
	CursorRoot,
	/// Revert the focus to the parent of the [window] which the cursor is in at
	/// the time.
	///
	/// [window]: Window
	Parent,
}

derive_xrb! {
	/// A [request] that changes the current focus.
	///
	/// This [request] generates [`Focus`] and [`Unfocus`] events.
	///
	/// # Errors
	/// A [`Match` error] is generated of the specified `new_focus` is not
	/// viewable at the time of the [request].
	///
	/// A [`Window` error] is generated if `new_focus` is [`FocusWindow::Other`]
	/// and does not refer to a defined [window].
	///
	/// [window]: Window
	/// [request]: Request
	///
	/// [`Focus`]: crate::x11::event::Focus
	/// [`Unfocus`]: crate::x11::event::Unfocus
	///
	/// [`Match` error]: error::Match
	/// [`Window` error]: error::Window
	#[doc(alias("SetInputFocus", "Focus", "FocusWindow"))]
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct SetFocus: Request(42, SetFocusError) {
		/// What the focus should revert to if the focused [window] becomes
		/// unviewable.
		///
		/// [window]: Window
		#[metabyte]
		pub revert_to: RevertFocus,

		/// The new focus.
		///
		/// # Errors
		/// A [`Window` error] is generated if this is [`FocusWindow::Other`]
		/// but does not refer to a defined [window].
		///
		/// [window]: Window
		///
		/// [`Window` error]: error::Window
		#[doc(alias = "focus")]
		pub new_focus: FocusWindow,

		/// The [time] at which the focus is recorded as having changed.
		///
		/// [time]: crate::Timestamp
		pub time: CurrentableTime,
	}

	/// A [request] that returns the current focus.
	///
	/// [request]: Request
	#[doc(alias = "GetInputFocus")]
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct GetFocus: Request(43) -> reply::GetFocus;

	/// A [request] that returns a bit vector of the currently held keys on the
	/// keyboard.
	///
	/// [request]: Request
	#[doc(alias = "QueryKeymap")]
	#[derive(Debug, Hash, PartialEq, Eq, X11Size, Readable, Writable, ConstantX11Size)]
	pub struct QueryKeyboard: Request(44) -> reply::QueryKeyboard;
}