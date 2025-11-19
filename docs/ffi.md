# Foreign Function Interface (FFI) and related utilities

This module contains everything related to the Foreign Function Interface
(FFI) bindings used by the `rtiddsconnector` crate to interface with the
_underlying C implementation of the RTI Connector API_.

It's a bundle of low-level abstractions that are not intended to be used
directly by end users, but rather to provide safe abstractions on top of
the C API. Users should be using [`Connector`][connector], [`Input`][input],
and [`Output`][output] abstractions instead.

[connector]: crate::Connector
[input]: crate::Input
[output]: crate::Output
