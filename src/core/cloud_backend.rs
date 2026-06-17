use crate::core::error::DecapodError;

pub const PUBLIC_CLOUD_BACKEND_UNAVAILABLE: &str = "Cloud backend is not included in the public Decapod crate. Use local mode; future cloud integrations must attach through a public backend boundary without private git/path dependencies.";

pub fn unavailable_error() -> DecapodError {
    DecapodError::NotImplemented(PUBLIC_CLOUD_BACKEND_UNAVAILABLE.to_string())
}
