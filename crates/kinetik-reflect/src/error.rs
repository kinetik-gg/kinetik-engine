use core::fmt;

use kinetik_core::{
    Diagnostic, DiagnosticCode, DiagnosticLocation, DiagnosticSeverity, DiagnosticSource,
};

use crate::PropertyType;

/// Result type for reflection descriptor operations.
pub type ReflectResult<T> = Result<T, DescriptorError>;

/// Result type for reflection value operations.
pub type ValueResult<T> = Result<T, ValueError>;

/// Errors returned when descriptor metadata violates the reflection contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorError {
    /// Canonical property path was empty.
    EmptyPath,
    /// Canonical property path was not `PascalCase` dot-separated text.
    InvalidPath {
        /// Invalid path value.
        path: String,
    },
    /// Display name was empty.
    EmptyDisplayName {
        /// Property path being described.
        path: String,
    },
    /// Serialization key was empty.
    EmptySerializationKey {
        /// Property path being described.
        path: String,
    },
    /// Explicit default value type did not match the descriptor type.
    DefaultTypeMismatch {
        /// Property path being described.
        path: String,
        /// Expected descriptor type.
        expected: PropertyType,
        /// Actual default value type.
        actual: PropertyType,
    },
    /// A read-only editor property did not explain why it is locked.
    MissingReadOnlyReason {
        /// Property path being described.
        path: String,
    },
}

impl fmt::Display for DescriptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => f.write_str("property descriptor path must not be empty"),
            Self::InvalidPath { path } => {
                write!(f, "property descriptor path must be PascalCase: {path}")
            }
            Self::EmptyDisplayName { path } => {
                write!(
                    f,
                    "property descriptor display name must not be empty: {path}"
                )
            }
            Self::EmptySerializationKey { path } => {
                write!(
                    f,
                    "property descriptor serialization key must not be empty: {path}"
                )
            }
            Self::DefaultTypeMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "property descriptor default for {path} expected {expected}, got {actual}"
            ),
            Self::MissingReadOnlyReason { path } => {
                write!(f, "read-only property must include a reason: {path}")
            }
        }
    }
}

impl std::error::Error for DescriptorError {}

/// Errors returned when reflected values violate descriptor contracts.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueError {
    /// Descriptor validation failed before value validation.
    InvalidDescriptor(DescriptorError),
    /// Reflected type has no meaningful neutral default value.
    NoTypeDefault {
        /// Reflected type without a neutral default.
        value_type: PropertyType,
    },
    /// Value type did not match the descriptor type.
    TypeMismatch {
        /// Canonical property path.
        path: String,
        /// Expected descriptor type.
        expected: PropertyType,
        /// Actual value type.
        actual: PropertyType,
    },
}

impl ValueError {
    /// Stable diagnostic code for invalid descriptors during value validation.
    pub const INVALID_DESCRIPTOR_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_REFLECT_INVALID_DESCRIPTOR");

    /// Stable diagnostic code for missing reflected type defaults.
    pub const NO_TYPE_DEFAULT_CODE: DiagnosticCode =
        DiagnosticCode::new("KT_REFLECT_NO_TYPE_DEFAULT");

    /// Stable diagnostic code for reflected value type mismatches.
    pub const TYPE_MISMATCH_CODE: DiagnosticCode = DiagnosticCode::new("KT_REFLECT_TYPE_MISMATCH");

    /// Returns the stable diagnostic code for this value error.
    #[must_use]
    pub const fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::InvalidDescriptor(_) => Self::INVALID_DESCRIPTOR_CODE,
            Self::NoTypeDefault { .. } => Self::NO_TYPE_DEFAULT_CODE,
            Self::TypeMismatch { .. } => Self::TYPE_MISMATCH_CODE,
        }
    }

    /// Converts this error into a structured diagnostic.
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        let mut location = DiagnosticLocation::new();
        if let Self::TypeMismatch { path, .. } = self {
            location.property_path = Some(path.clone());
        }
        Diagnostic::new(
            self.diagnostic_code(),
            DiagnosticSeverity::Error,
            DiagnosticSource::new("Reflection"),
            self.to_string(),
        )
        .with_location(location)
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDescriptor(error) => write!(f, "invalid property descriptor: {error}"),
            Self::NoTypeDefault { value_type } => {
                write!(f, "reflected type has no neutral default: {value_type}")
            }
            Self::TypeMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "property {path} expected value type {expected}, got {actual}"
            ),
        }
    }
}

impl std::error::Error for ValueError {}
