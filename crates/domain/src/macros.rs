/// A typed id around a string that must not be blank, like the Kotlin `@JvmInline value class` ids.
macro_rules! non_blank_id {
    ($(#[$meta:meta])* $name:ident, $message:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            /// # Errors
            /// Fails when the value is blank.
            pub fn new(value: impl Into<String>) -> Result<Self, $crate::DomainError> {
                let value = value.into();
                $crate::error::require(!$crate::text::is_blank(&value), || $message.to_owned())?;
                Ok(Self(value))
            }

            #[must_use]
            pub fn value(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

pub(crate) use non_blank_id;
