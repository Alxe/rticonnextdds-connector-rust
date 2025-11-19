/*******************************************************************************
 * (c) 2025 Copyright, Real-Time Innovations.  All rights reserved.            *
 * No duplications, whole or partial, manual or electronic, may be made        *
 * without express written permission.  Any such copies, or revisions thereof, *
 * must display this notice unaltered.                                         *
 * This code contains trade secrets of Real-Time Innovations, Inc.             *
 *******************************************************************************/

#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/result.md"))]

/// A type alias for results returned by Connector operations
pub type ConnectorResult<T> = std::result::Result<T, ConnectorError>;

/// A type alias for results that return no value on success
pub type ConnectorFallible = ConnectorResult<()>;

/// An error returned by Connector operations
#[derive(Debug)]
pub struct ConnectorError {
    /// The kind of error that occurred
    pub(crate) kind: ErrorKind,
    /// The last error message from the native library, if any
    last_error_message: Option<String>,
}

impl ConnectorError {
    /// Check if the error is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self.kind, ErrorKind::Timeout)
    }

    /// Check if the error is a not found entity error
    pub fn is_entity_not_found(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::NotFound {
                what: NotFoundErrorKind::Entity,
                ..
            }
        )
    }

    /// Check if the error is a not found field error
    pub fn is_field_not_found(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::NotFound {
                what: NotFoundErrorKind::Field,
                ..
            }
        )
    }

    /// Check if the error is a native error
    pub fn is_native_error(&self) -> bool {
        matches!(self.kind, ErrorKind::Native { .. })
    }

    pub(crate) fn is_native_error_code(&self, code: crate::ffi::ReturnCode) -> bool {
        matches!(self.kind, ErrorKind::Native { code: c } if c == code)
    }

    /// Get the last error message from the native library, if any
    pub fn last_error_message(&self) -> Option<&str> {
        self.last_error_message.as_deref()
    }
}

impl<T> From<ConnectorError> for ConnectorResult<T> {
    fn from(value: ConnectorError) -> Self {
        Err(value)
    }
}

/// Check if the error message indicates an invalid field error
/// Returns the field name if found, otherwise None
fn invalid_field_error_from_message(message: &str) -> Option<&str> {
    // Extract field name from error string. Two examples:
    // - Sample field: Cannot find a member (name = \"non_existent_field\", id = 0) in type SimpleStruct\n
    // - Info field: ERROR RTILuaSampleInfo_get:Unknown SampleInfo field: unknown_field\n
    if message.contains("Cannot find a member") {
        // Sample field error
        message
            .split("name = \"")
            .nth(1)
            .and_then(|s| s.split('"').next())
    } else if message.contains("Unknown SampleInfo field:") {
        // Info field error
        message
            .split("Unknown SampleInfo field:")
            .nth(1)
            .and_then(|s| s.split('\n').next())
    } else {
        None
    }
}

impl From<ErrorKind> for ConnectorError {
    fn from(kind: ErrorKind) -> Self {
        // Only fetch error message for errors that come from native code
        let last_error_message = crate::Connector::get_last_error_message();

        // Special case for transforming error messages about missing fields
        if let Some(message) = &last_error_message
            && let Some(field_name) = invalid_field_error_from_message(message)
        {
            Self {
                kind: ErrorKind::field_not_found_error(field_name),
                last_error_message,
            }
        } else {
            Self {
                kind,
                last_error_message,
            }
        }
    }
}

impl std::error::Error for ConnectorError {}

impl std::fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::Native { code } => {
                write!(f, "Native error with code '{}'", code)
            }

            ErrorKind::NotFound { what, name } => match what {
                NotFoundErrorKind::Entity => write!(f, "Entity '{}' was not found", name),
                NotFoundErrorKind::Field => write!(f, "Field '{}' was not found", name),
            },

            ErrorKind::Invalid {
                what,
                context: reason,
            } => match what {
                InvalidErrorKind::Argument => write!(f, "Invalid argument: {}", reason),
                InvalidErrorKind::Conversion => {
                    write!(f, "Invalid conversion: {}", reason)
                }
                InvalidErrorKind::Serialization => {
                    write!(f, "Invalid serialization: {}", reason)
                }
                InvalidErrorKind::Deserialization => {
                    write!(f, "Invalid deserialization: {}", reason)
                }
                InvalidErrorKind::Assertion => write!(f, "Assertion failed: {}", reason),
            },

            ErrorKind::Busy {
                resource,
                context: reason,
            } => match resource {
                BusyErrorKind::Entity => write!(f, "Entity is busy: {}", reason),
                BusyErrorKind::Lock => write!(f, "Lock is busy: {}", reason),
            },

            ErrorKind::Timeout => {
                write!(f, "Operation timed out")
            }
        }?;

        if let Some(msg) = &self.last_error_message {
            write!(f, " - Last error message: {}", msg)
        } else {
            Ok(())
        }
    }
}

/// An enumeration of possible errors returned by Connector operations
#[derive(Debug)]
pub enum ErrorKind {
    /// Some error occurred in the Native libraries
    Native {
        /// The return code from the native library
        code: crate::ffi::ReturnCode,
    },

    /// Some element was not found
    NotFound {
        /// What type of thing was not found
        what: NotFoundErrorKind,
        /// The name of the thing that was not found
        name: String,
    },

    /// Some operation was found to be invalid
    Invalid {
        /// What was invalid
        what: InvalidErrorKind,
        /// The reason why it's invalid
        context: String,
    },

    /// Some resource was found to be busy
    Busy {
        /// The resource that is busy
        resource: BusyErrorKind,
        /// The reason why it's busy
        context: String,
    },

    /// Operation timed out
    Timeout,
}

/// What type of thing was not found
#[derive(Debug, Clone, PartialEq)]
pub enum NotFoundErrorKind {
    /// An entity (Input, Output, Connector) was not found
    Entity,
    /// A field in a sample or instance was not found
    Field,
}

/// What type of invalid input was encountered
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidErrorKind {
    /// An argument passed to a function was invalid
    Argument,
    /// A conversion or serialization operation failed
    Conversion,
    /// A serialization operation failed
    Serialization,
    /// A deserialization operation failed
    Deserialization,
    /// An assertion failed, this indicates a bug in the library
    Assertion,
}

/// What type of resource is busy
#[derive(Debug, Clone, PartialEq)]
pub enum BusyErrorKind {
    /// An entity is busy (e.g., has outstanding loans)
    Entity,
    /// A lock could not be acquired
    Lock,
}

impl ErrorKind {
    /// Helper to create an InvalidArgument error
    pub fn invalid_argument_error(context: impl Into<String>) -> Self {
        Self::Invalid {
            what: InvalidErrorKind::Argument,
            context: context.into(),
        }
    }

    pub fn entity_busy_error(context: impl Into<String>) -> Self {
        Self::Busy {
            resource: BusyErrorKind::Entity,
            context: context.into(),
        }
    }

    pub fn lock_poisoned_error(context: impl Into<String>) -> Self {
        Self::Busy {
            resource: BusyErrorKind::Lock,
            context: context.into(),
        }
    }

    pub fn invalid_string_conversion_error() -> Self {
        Self::Invalid {
            what: InvalidErrorKind::Conversion,
            context: "string conversion failed".into(),
        }
    }

    /// Helper to create a TimeoutError
    pub fn timeout_error() -> Self {
        Self::Timeout
    }

    /// Helper to create an EntityNotFound error
    pub fn entity_not_found_error(entity_name: impl Into<String>) -> Self {
        Self::NotFound {
            what: NotFoundErrorKind::Entity,
            name: entity_name.into(),
        }
    }

    /// Helper to create a FieldNotFound error
    pub fn field_not_found_error(field_name: impl Into<String>) -> Self {
        Self::NotFound {
            what: NotFoundErrorKind::Field,
            name: field_name.into(),
        }
    }

    /// Helper to create an [`Native`][ErrorKind::Native] variant from a FFI return code
    pub fn native_error(code: crate::ffi::ReturnCode) -> Self {
        Self::Native { code }
    }

    /// Helper to create an [`InvalidErrorKind::Assertion`] error
    pub fn assertion_failed_error(context: impl Into<String>) -> Self {
        Self::Invalid {
            what: InvalidErrorKind::Assertion,
            context: context.into(),
        }
    }

    /// Turn this error into a [Err] variant of [ConnectorResult]
    pub fn into_err<R>(self) -> ConnectorResult<R> {
        ConnectorError::from(self).into()
    }
}
