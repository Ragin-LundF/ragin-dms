/// The areas of a German income tax return a document can belong to; the form is named in brackets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaxArea {
    /// Anlage N.
    Income,
    /// Vorsorgeaufwand.
    PensionInsurance,
    /// § 35a `EStG`.
    HouseholdServices,
    /// Sonderausgaben.
    Donations,
    /// Außergewöhnliche Belastungen.
    HealthCosts,
    /// Werbungskosten.
    WorkExpenses,
    /// Anlage KAP.
    CapitalIncome,
    /// Anlage V.
    Rental,
    Other,
}

impl TaxArea {
    pub const ALL: [Self; 9] = [
        Self::Income,
        Self::PensionInsurance,
        Self::HouseholdServices,
        Self::Donations,
        Self::HealthCosts,
        Self::WorkExpenses,
        Self::CapitalIncome,
        Self::Rental,
        Self::Other,
    ];
}

impl TaxArea {
    /// The English name the screens, the export and the missing-items page show.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Income => "Income",
            Self::PensionInsurance => "Insurance and pension",
            Self::HouseholdServices => "Household services and craftsmen",
            Self::Donations => "Donations",
            Self::HealthCosts => "Health costs",
            Self::WorkExpenses => "Work-related expenses",
            Self::CapitalIncome => "Capital income",
            Self::Rental => "Rental income",
            Self::Other => "Other",
        }
    }

    /// The part of the German return the area goes into.
    #[must_use]
    pub fn form(self) -> &'static str {
        match self {
            Self::Income => "Anlage N",
            Self::PensionInsurance => "Vorsorgeaufwand",
            Self::HouseholdServices => "§ 35a EStG",
            Self::Donations => "Sonderausgaben",
            Self::HealthCosts => "Außergewöhnliche Belastungen",
            Self::WorkExpenses => "Werbungskosten",
            Self::CapitalIncome => "Anlage KAP",
            Self::Rental => "Anlage V",
            Self::Other => "Other",
        }
    }
}
