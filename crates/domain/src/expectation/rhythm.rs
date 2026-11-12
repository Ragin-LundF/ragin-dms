/// How often a sender's documents arrive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rhythm {
    Monthly,
    Quarterly,
    HalfYearly,
    Yearly,
}

impl Rhythm {
    pub const ALL: [Self; 4] = [Self::Monthly, Self::Quarterly, Self::HalfYearly, Self::Yearly];

    /// The gaps in days, both ends included, that count as this rhythm.
    #[must_use]
    pub fn band(self) -> (i64, i64) {
        match self {
            Self::Monthly => (25, 35),
            Self::Quarterly => (80, 100),
            Self::HalfYearly => (170, 200),
            Self::Yearly => (330, 400),
        }
    }

    /// Days after the expected date until a document counts as missing.
    #[must_use]
    pub fn tolerance_days(self) -> i64 {
        match self {
            Self::Monthly => 10,
            Self::Quarterly => 21,
            Self::HalfYearly => 30,
            Self::Yearly => 45,
        }
    }

    #[must_use]
    pub fn contains(self, gap: i64) -> bool {
        let (low, high) = self.band();
        (low..=high).contains(&gap)
    }

    /// The rhythm whose band holds `gap`, if any; the bands do not overlap.
    #[must_use]
    pub fn of_gap(gap: i64) -> Option<Self> {
        Self::ALL.into_iter().find(|rhythm| rhythm.contains(gap))
    }
}
