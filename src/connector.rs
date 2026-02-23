/*******************************************************************************
 * (c) 2025 Copyright, Real-Time Innovations.  All rights reserved.            *
 * No duplications, whole or partial, manual or electronic, may be made        *
 * without express written permission.  Any such copies, or revisions thereof, *
 * must display this notice unaltered.                                         *
 * This code contains trade secrets of Real-Time Innovations, Inc.             *
 *******************************************************************************/

#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/connector.md"))]

use crate::{
    ConnectorFallible, ConnectorResult, Input, Output, ffi::FfiConnector,
    result::ErrorKind,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

/// A variant type that can hold a [number][selected_number],
/// a [boolean][selected_boolean], or a [string][selected_string] value.
///
/// This type is used for both [setting][set_value] and [retrieving][get_value]
/// values from DDS samples in a type-safe manner, respectively with
/// [`Instance::set_value`][set_value] and [`Sample::get_value`][get_value].
///
/// Note that complex types (such as nested structures) are
/// internally represented as JSON strings, and should be set and retrieved
/// using [`SelectedValue::String`].
///
/// # Examples
///
/// ```rust
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/snippets/connector/using_selected_value.rs"))]
/// ```
///
/// [selected_number]: SelectedValue::Number
/// [selected_boolean]: SelectedValue::Boolean
/// [selected_string]: SelectedValue::String
/// [set_value]: crate::Instance::set_value
/// [get_value]: crate::Sample::get_value
#[derive(Debug, Clone, PartialEq)]
pub enum SelectedValue {
    /// A numeric value
    Number(f64),

    /// A boolean value
    Boolean(bool),

    /// A string value
    String(String),
}

/// Allows quick conversion from [f64] to [SelectedValue::Number].
impl From<f64> for SelectedValue {
    fn from(v: f64) -> Self {
        SelectedValue::Number(v)
    }
}

/// Allows quick conversion from [bool] to [SelectedValue::Boolean].
impl From<bool> for SelectedValue {
    fn from(v: bool) -> Self {
        SelectedValue::Boolean(v)
    }
}

/// Allows quick conversion from [String] to [SelectedValue::String].
impl From<String> for SelectedValue {
    fn from(v: String) -> Self {
        SelectedValue::String(v)
    }
}

/// Allows quick conversion from [str] to [SelectedValue::String].
impl From<&str> for SelectedValue {
    fn from(v: &str) -> Self {
        v.to_string().into()
    }
}

/// The main interface to the RTI Connector for Rust API.
///
/// Representing a DDS `DomainParticipant` and its contained
/// `DataReader`s and `DataWriter`s, a `Connector` object is
/// used to create [`Input`] and [`Output`] objects for reading
/// and writing DDS data, respectively.
///
/// [`Connector::get_input`] and [`Connector::get_output`] are the main
/// methods of this struct, allowing to acquire references to
/// [`Input`] and [`Output`] objects for reading and writing DDS data.
///
/// # Examples
/// ```rust
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/snippets/connector/using_connector.rs"))]
/// ```
#[derive(Clone)]
#[repr(transparent)]
pub struct Connector {
    /// Shared state of the [`Connector`] object.
    inner: Arc<ConnectorInner>,
}

impl std::fmt::Debug for Connector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ConnectorInner::fmt(self.inner.as_ref(), f)
    }
}

pub(crate) struct ConnectorInner {
    /// The name of the configuration used to create this Connector.
    name: String,

    /// The native connector instance.
    native: Mutex<FfiConnector>,

    /// Tracking cache for created Inputs with weak references
    inputs: Mutex<HashMap<String, Weak<crate::input::InputInner>>>,

    /// Tracking cache for created Outputs with weak references
    outputs: Mutex<HashMap<String, Weak<crate::output::OutputInner>>>,
}

impl std::fmt::Debug for ConnectorInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connector")
            .field("name", &self.name)
            .finish()
    }
}

impl Connector {
    /// Retrieve a string describing the version of the RTI Connector for Rust
    /// and the underlying [RTI Connext] installation.
    ///
    /// [RTI Connext]: https://www.rti.com/products/dds "RTI Connext Professional"
    pub fn get_versions_string() -> String {
        static VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

        let (ndds_build_id_string, rtiddsconnector_build_id_string) =
            FfiConnector::get_build_versions().unwrap_or((
                "<Unknown RTI Connext version>".to_string(),
                "<Unknown RTI Connector for Rust version>".to_string(),
            ));

        format!(
            "RTI Connector for Rust, version {}\n{}\n{}",
            VERSION_STRING, ndds_build_id_string, rtiddsconnector_build_id_string
        )
    }

    /// Get the last error message from the underlying RTI Connector C API.
    pub(crate) fn get_last_error_message() -> Option<String> {
        FfiConnector::get_last_error_message()
    }

    /// Create a new [`Connector`] from a named configuration contained
    /// in an external XML file.
    pub fn new(config_name: &str, config_file: &str) -> ConnectorResult<Connector> {
        static NATIVE_CONNECTOR_CREATION_LOCK: Mutex<()> = Mutex::new(());

        let native: FfiConnector = {
            let _guard = NATIVE_CONNECTOR_CREATION_LOCK
                .lock()
                .inspect_err(|_| {
                    eprintln!("An error occurred while trying to lock the global native connector creation lock, continuing anyway...");
                })
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            FfiConnector::new(config_name, config_file)?
        };

        Ok(Connector {
            inner: Arc::new(ConnectorInner {
                name: config_name.into(),
                native: Mutex::new(native),
                inputs: Mutex::new(HashMap::new()),
                outputs: Mutex::new(HashMap::new()),
            }),
        })
    }

    /// Wait until data is available to read from any of its [`Input`], indefinitely.
    pub fn wait_for_data(&self) -> ConnectorFallible {
        self.impl_wait_for_data(None)
    }

    /// Wait until data is available to read from any of its [`Input`], with a timeout.
    pub fn wait_for_data_with_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> ConnectorFallible {
        self.impl_wait_for_data(Some(
            // Durations cannot be negative
            timeout.as_millis().try_into().unwrap_or(i32::MAX),
        ))
    }

    /// Implementation of wait for data functionality.
    fn impl_wait_for_data(&self, timeout: Option<i32>) -> ConnectorFallible {
        self.inner.native()?.wait_for_data(timeout)
    }

    /// Get an [`Input`] instance contained in this [`Connector`].
    ///
    /// An error will be returned if the named [`Input`] is not contained in
    /// the Connector.
    pub fn get_input(&self, name: &str) -> ConnectorResult<Input> {
        let inner = &self.inner;

        // First, check if we already have this input
        let mut inputs = inner
            .inputs
            .lock()
            .map_err(|_| ErrorKind::lock_poisoned_error("Input cache lock poisoned"))?;

        // Try to upgrade existing weak reference
        if let Some(weak) = inputs.get(name)
            && let Some(input_inner) = weak.upgrade()
        {
            // Reconstruct Input from already-tracked InputInner
            return Ok(Input::from_inner(input_inner, inner));
        }

        // Not tracked yet, get the native and create new Input
        let native = inner.native()?.get_input(name)?;
        let input = Input::new(name, native, inner)?;

        // Store weak reference for future lookups
        inputs.insert(name.to_string(), Arc::downgrade(input.inner()));

        Ok(input)
    }

    /// Get an [`Output`] instance contained in this [`Connector`].
    ///
    /// An error will be returned if the named [`Output`] is not contained in
    /// the Connector.
    pub fn get_output(&self, name: &str) -> ConnectorResult<Output> {
        let inner = &self.inner;
        // First, check if we already have this output
        let mut outputs = inner
            .outputs
            .lock()
            .map_err(|_| ErrorKind::lock_poisoned_error("Output cache lock poisoned"))?;

        // Try to upgrade existing weak reference
        if let Some(weak) = outputs.get(name)
            && let Some(output_inner) = weak.upgrade()
        {
            // Reconstruct Output from already-tracked OutputInner
            return Ok(Output::from_inner(output_inner, inner));
        }

        // Not tracked yet, get the native and create new Output
        let native = inner.native()?.get_output(name)?;
        let output = Output::new(name, native, inner)?;

        // Store weak reference for future lookups
        outputs.insert(name.to_string(), Arc::downgrade(output.inner()));

        Ok(output)
    }
}

impl ConnectorInner {
    /// Get access to the [`FfiConnector`] through a lock guard.
    pub(crate) fn native(
        &self,
    ) -> ConnectorResult<std::sync::MutexGuard<'_, FfiConnector>> {
        self.native.lock().map_err(|_| {
            ErrorKind::lock_poisoned_error(
                "Another thread panicked while holding the native connector lock",
            )
            .into()
        })
    }
}
