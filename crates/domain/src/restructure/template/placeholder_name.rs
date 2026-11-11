/// The values a [`crate::restructure::PathTemplate`] can insert, by the name written between braces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaceholderName {
    /// The category's full path, such as `Insurance/Car`; one folder per level.
    Category,
    CategoryTop,
    Year,
    Month,
    Correspondent,
    Kind,
    /// A custom field, written `{field:<key>}`.
    Field,
}

impl PlaceholderName {
    const ALL: [Self; 7] =
        [Self::Category, Self::CategoryTop, Self::Year, Self::Month, Self::Correspondent, Self::Kind, Self::Field];

    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Self::Category => "category",
            Self::CategoryTop => "category.top",
            Self::Year => "year",
            Self::Month => "month",
            Self::Correspondent => "correspondent",
            Self::Kind => "kind",
            Self::Field => "field",
        }
    }

    #[must_use]
    pub fn of(token: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|name| name.token() == token)
    }
}
