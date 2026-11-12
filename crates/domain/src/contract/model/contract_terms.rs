use chrono::{Datelike, Days, Months, NaiveDate};

use crate::DomainError;
use crate::contract::{Notice, NoticeUnit};
use crate::error::require;

/// When a contract ends, how it renews and how early it must be cancelled. A contract that can be cancelled any
/// time has no cancel-by date.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ContractTerms {
    pub term_end: Option<NaiveDate>,
    pub renews_by_months: Option<u16>,
    pub notice: Option<Notice>,
    pub cancellable_any_time: bool,
}

impl ContractTerms {
    /// # Errors
    /// Fails for a renewal of zero months or a notice of zero.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(self.renews_by_months.is_none_or(|months| months > 0), || {
            "A contract renews by at least 1 month".to_owned()
        })?;
        self.notice.as_ref().map_or(Ok(()), Notice::validate)
    }

    /// The last day on `today` or later the contract can be cancelled for the end of a term: the term end minus the
    /// notice, with the term end rolled forward by the renewal while that day has passed. `None` when the contract
    /// can be cancelled any time, has no term end, or its last term is over and it does not renew. Without a notice
    /// the term end itself is the last day.
    #[must_use]
    pub fn next_cancel_by(&self, today: NaiveDate) -> Option<NaiveDate> {
        self.next_cancellation(today).map(|(cancel_by, _)| cancel_by)
    }

    /// The end of the term a cancellation on `today` reaches: the term end that belongs to
    /// [`Self::next_cancel_by`].
    #[must_use]
    pub fn next_term_end(&self, today: NaiveDate) -> Option<NaiveDate> {
        self.next_cancellation(today).map(|(_, term_end)| term_end)
    }

    fn next_cancellation(&self, today: NaiveDate) -> Option<(NaiveDate, NaiveDate)> {
        if self.cancellable_any_time {
            return None;
        }
        let first_end = self.term_end?;
        let mut terms: u32 = 0;
        loop {
            let term_end = match self.renews_by_months {
                Some(months) => plus_months(first_end, u32::from(months).checked_mul(terms)?)?,
                None if terms == 0 => first_end,
                None => return None,
            };
            let cancel_by = self.notice.map_or(Some(term_end), |notice| minus_notice(term_end, notice))?;
            if cancel_by >= today {
                return Some((cancel_by, term_end));
            }
            terms = terms.checked_add(1)?;
        }
    }
}

/// `date` plus `months`; a term ending on the last day of a month keeps ending on the last day (30 June → 31 July).
fn plus_months(date: NaiveDate, months: u32) -> Option<NaiveDate> {
    let moved = date.checked_add_months(Months::new(months))?;
    if is_last_of_month(date) { last_of_month(moved) } else { Some(moved) }
}

fn minus_notice(term_end: NaiveDate, notice: Notice) -> Option<NaiveDate> {
    let count = u32::from(notice.count);
    match notice.unit {
        NoticeUnit::Days => term_end.checked_sub_days(Days::new(u64::from(count))),
        NoticeUnit::Weeks => term_end.checked_sub_days(Days::new(u64::from(count) * 7)),
        NoticeUnit::Months => {
            let moved = term_end.checked_sub_months(Months::new(count))?;
            if is_last_of_month(term_end) { last_of_month(moved) } else { Some(moved) }
        }
    }
}

fn is_last_of_month(date: NaiveDate) -> bool {
    date.succ_opt().is_none_or(|next| next.month() != date.month())
}

fn last_of_month(date: NaiveDate) -> Option<NaiveDate> {
    let first = date.with_day(1)?;
    first.checked_add_months(Months::new(1))?.pred_opt()
}
