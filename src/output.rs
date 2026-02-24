/*******************************************************************************
 * (c) 2025 Copyright, Real-Time Innovations.  All rights reserved.            *
 * No duplications, whole or partial, manual or electronic, may be made        *
 * without express written permission.  Any such copies, or revisions thereof, *
 * must display this notice unaltered.                                         *
 * This code contains trade secrets of Real-Time Innovations, Inc.             *
 *******************************************************************************/

#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/output.md"))]

use std::sync::{Arc, Mutex, atomic::AtomicUsize};

use crate::{
    ConnectorFallible, ConnectorResult, SelectedValue,
    ffi::FfiOutput,
    result::{ErrorKind, InvalidErrorKind},
};

#[cfg(doc)]
use crate::Connector;

/// An interface to modify the data held by a given [`Output`] instance.
///
/// ```rust
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/snippets/output/using_instance.rs"))]
/// ```
pub struct Instance<'a> {
    /// A reference to the parent [`Output`]
    output: &'a Output,

    /// The generation of the [`Instance`], used to detect staleness.
    generation: usize,
}

/// Display the [`Instance`] as a JSON string.
impl std::fmt::Display for Instance<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.get_as_json() {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, "<error retrieving instance as JSON>"),
        }
    }
}

impl Instance<'_> {
    fn parent(&self) -> ConnectorResult<&crate::connector::ConnectorInner> {
        if self.generation != self.output.generation() {
            ErrorKind::stale_entity_error(
                "Instance invalidated by Output::write or Output::write_with_params",
            )
            .into_err()
        } else {
            Ok(&self.output.parent)
        }
    }

    /// Clear a specific field of the underlying sample.
    pub fn clear(&mut self, field: &str) -> ConnectorFallible {
        self.parent()?
            .native()?
            .clear_member(self.output.name(), field)
    }

    /// Set the entire instance from a JSON string.
    pub fn set_as_json(&mut self, json_value: &str) -> ConnectorFallible {
        self.parent()?
            .native()?
            .set_json_instance(self.output.name(), json_value)
    }

    /// Set a specific field of the underlying sample.
    pub fn set_value(&mut self, field: &str, value: SelectedValue) -> ConnectorFallible {
        self.parent()?
            .native()?
            .set_into_samples(self.output.name(), field, value)
    }

    /// Set a numeric field of the underlying sample.
    pub fn set_number(&mut self, field: &str, value: f64) -> ConnectorFallible {
        self.parent()?
            .native()?
            .set_number_into_samples(self.output.name(), field, value)
    }

    /// Set a boolean field of the underlying sample.
    pub fn set_boolean(&mut self, field: &str, value: bool) -> ConnectorFallible {
        self.parent()?.native()?.set_boolean_into_samples(
            self.output.name(),
            field,
            value,
        )
    }

    /// Set a string field of the underlying sample.
    pub fn set_string(&mut self, field: &str, value: &str) -> ConnectorFallible {
        self.parent()?
            .native()?
            .set_string_into_samples(self.output.name(), field, value)
    }

    /// Set the instance data from a typed struct using Serde serialization.
    ///
    /// This method allows you to work with strongly-typed data structures
    /// instead of setting fields individually.
    ///
    /// # Example
    /// ```rust
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/snippets/output/using_serialization.rs"))]
    /// ```
    pub fn serialize<T>(&mut self, data: &T) -> ConnectorFallible
    where
        T: serde::Serialize,
    {
        let json = serde_json::to_string(data).map_err(|e| ErrorKind::Invalid {
            what: InvalidErrorKind::Serialization,
            context: std::format!(
                "Type '{}' could not be serialized: {}",
                std::any::type_name::<T>(),
                e
            ),
        })?;

        self.set_as_json(&json).map_err(|e| ErrorKind::Invalid {
            what: InvalidErrorKind::Serialization,
            context: std::format!(
                "Failed setting JSON serialied field ({}) of type '{}': {}",
                std::any::type_name::<T>(),
                json,
                e
            ),
        })?;

        Ok(())
    }

    /// Get the entire instance as a JSON string.
    pub(crate) fn get_as_json(&self) -> ConnectorResult<String> {
        self.parent()?
            .native()?
            .get_json_instance(self.output.name())
    }
}

/// An interface to write data to a DDS `Topic`.
///
/// Created with [`Connector::get_output`], an [`Output`] represents a DDS
/// `DataWriter` associated with a specific `Topic` within a `Participant`.
///
/// The main functionality of an [`Output`] is to provide access to an
/// [`Instance`], which allows modifying the data to be written, and [`Output::write`]
/// to publish said data to DDS.
///
/// ```rust
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/snippets/output/using_output.rs"))]
/// ```
#[derive(Clone)]
pub struct Output {
    /// A reference to the parent [`Connector`].
    parent: Arc<crate::connector::ConnectorInner>,

    /// Shared state of the [`Output`] object.
    inner: Arc<OutputInner>,
}

impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Output")
            .field("name", &self.name())
            .field("parent", &self.parent)
            .finish()
    }
}

pub(crate) struct OutputInner {
    /// The name of the output as known to the parent [`Connector`].
    name: String,

    /// Reference to the native Output entity, allowing per-entity locking.
    native: Mutex<crate::ffi::FfiOutput>,

    /// The generation of the samples, used to detect staleness.
    generation: AtomicUsize,
}

/// Action to perform when writing a sample.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WriteParamsAction {
    /// Write the sample and its contents
    #[default]
    Write,

    /// Dispose the sample by using its key fields
    Dispose,

    /// Unregister the sample by using its key fields
    Unregister,
}

/// Identity of a written sample.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct WriteParamsIdentity {
    /// The GUID of the writer as a list of 16 bytes.
    pub writer_guid: [u8; 16],

    /// The sequence number of the sample.
    pub sequence_number: u64,
}

/// Parameters for writing a sample.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct WriteParams {
    /// One of "write" (default), "dispose" or "unregister".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<WriteParamsAction>,

    /// The source timestamp, an integer representing the total number of nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_timestamp: Option<i64>,

    /// A dictionary containing the keys "writer_guid" (a list of 16 bytes) and "sequence_number" (an integer) that uniquely identifies this sample.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<WriteParamsIdentity>,

    /// elated_sample_identity (dict) – Used for request-reply communications. It has the same format as identity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_sample_identity: Option<WriteParamsIdentity>,
}

impl WriteParams {
    /// Create WriteParams for a write action.
    pub fn write() -> Self {
        WriteParams {
            action: Some(WriteParamsAction::Write),
            ..Default::default()
        }
    }

    /// Create WriteParams for a dispose action.
    pub fn dispose() -> Self {
        WriteParams {
            action: Some(WriteParamsAction::Dispose),
            ..Default::default()
        }
    }

    /// Create WriteParams for an unregister action.
    pub fn unregister() -> Self {
        WriteParams {
            action: Some(WriteParamsAction::Unregister),
            ..Default::default()
        }
    }

    /// Set the source timestamp.
    pub fn with_source_timestamp(mut self, timestamp: i64) -> Self {
        self.source_timestamp = Some(timestamp);
        self
    }

    /// Set the writer identity.
    pub fn with_identity(mut self, identity: WriteParamsIdentity) -> Self {
        self.identity = Some(identity);
        self
    }

    /// Set the related sample identity.
    pub fn with_related_sample_identity(
        mut self,
        related_sample_identity: WriteParamsIdentity,
    ) -> Self {
        self.related_sample_identity = Some(related_sample_identity);
        self
    }
}

impl Output {
    pub(crate) fn new(
        name: &str,
        output: crate::ffi::FfiOutput,
        connector: &Arc<crate::connector::ConnectorInner>,
    ) -> ConnectorResult<Output> {
        Ok(Output {
            parent: connector.clone(),
            inner: Arc::new(OutputInner {
                name: name.to_string(),
                native: Mutex::new(output),
                generation: AtomicUsize::new(0),
            }),
        })
    }

    /// Reconstruct an Output from an already-tracked OutputInner
    pub(crate) fn from_inner(
        inner: Arc<OutputInner>,
        connector: &Arc<crate::connector::ConnectorInner>,
    ) -> Output {
        Output {
            parent: connector.clone(),
            inner,
        }
    }

    pub(crate) fn inner(&self) -> &Arc<OutputInner> {
        &self.inner
    }

    pub(crate) fn name(&self) -> &str {
        &self.inner.name
    }

    pub(crate) fn generation(&self) -> usize {
        self.inner
            .generation
            .load(std::sync::atomic::Ordering::Acquire)
    }

    /// Get an [`Instance`] of the data held by this [`Output`].
    pub fn instance<'a>(&'a self) -> Instance<'a> {
        Instance {
            output: self,
            generation: self.generation(),
        }
    }

    /// Clear all fields of the underlying sample.
    pub fn clear_members(&mut self) -> ConnectorFallible {
        self.parent.native()?.clear(&self.name())
    }

    /// Write the output sample using the underlying `DataWriter`.
    pub fn write(&mut self) -> ConnectorFallible {
        self.impl_write(None)
    }

    /// Write the output sample with specific parameters.
    pub fn write_with_params(&mut self, params: &WriteParams) -> ConnectorFallible {
        let params_json =
            serde_json::to_string(params).map_err(|e| ErrorKind::Invalid {
                what: crate::result::InvalidErrorKind::Serialization,
                context: std::format!("WriteParams could not be serialized: {}", e),
            })?;

        self.impl_write(Some(params_json))
    }

    fn impl_write(&mut self, params_json: Option<String>) -> ConnectorFallible {
        let result = {
            let native = self.parent.native()?;

            if let Some(params_json) = params_json {
                native.write_with_params(&self.name(), &params_json)
            } else {
                native.write(&self.name())
            }
        };

        if let Err(e) = result {
            // At the moment, no special handling is needed
            Err(e)
        } else {
            // Increment the generation to invalidate any existing instances.
            self.inner
                .generation
                .fetch_add(1, std::sync::atomic::Ordering::Release);
            Ok(())
        }
    }

    /// Wait until all previously written samples have been acknowledged, indefinitely.
    pub fn wait(&self) -> ConnectorFallible {
        self.impl_wait(None)
    }

    /// Wait until all previously written samples have been acknowledged, or until the timeout expires.
    pub fn wait_with_timeout(&self, timeout: std::time::Duration) -> ConnectorFallible {
        self.impl_wait(Some(
            // Durations cannot be negative
            timeout.as_millis().try_into().unwrap_or(i32::MAX),
        ))
    }

    /// Implementation of wait functionality.
    fn impl_wait(&self, timeout_ms: Option<i32>) -> ConnectorFallible {
        self.inner.native()?.wait_for_acknowledgments(timeout_ms)
    }

    /// Wait until a subscription is matched, indefinitely.
    pub fn wait_for_subscriptions(&self) -> ConnectorResult<i32> {
        self.impl_wait_for_subscriptions(None)
    }

    /// Wait until a subscription is matched, or until the timeout expires.
    pub fn wait_for_subscriptions_with_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> ConnectorResult<i32> {
        self.impl_wait_for_subscriptions(Some(
            // Durations cannot be negative
            timeout.as_millis().try_into().unwrap_or(i32::MAX),
        ))
    }

    /// Implementation of wait for subscriptions functionality.
    fn impl_wait_for_subscriptions(
        &self,
        timeout_ms: Option<i32>,
    ) -> ConnectorResult<i32> {
        self.inner
            .native()?
            .wait_for_matched_subscription(timeout_ms)
    }

    /// Display the matched subscriptions as a JSON string.
    pub fn display_matched_subscriptions(&self) -> ConnectorResult<String> {
        self.inner.native()?.get_matched_subscriptions()
    }
}

impl OutputInner {
    /// Get access to the [`FfiOutput`] through a lock guard.
    pub(crate) fn native(&self) -> ConnectorResult<std::sync::MutexGuard<'_, FfiOutput>> {
        self.native.lock().map_err(|_| {
            ErrorKind::lock_poisoned_error(
                "Another thread panicked while holding the native output lock",
            )
            .into()
        })
    }
}
