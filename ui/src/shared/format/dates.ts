/**
 * How every screen writes dates (Kotlin `DateTexts`): exact dates as `YYYY-MM-DD`, the same form the user types,
 * and relative ones in words ("today", "3 days ago", "in 2 weeks") with the exact date beside them in a tooltip.
 */

const DAYS_PER_WEEK = 7;
const DAYS_PER_MONTH = 30;
const DAYS_PER_YEAR = 365;
const MS_PER_DAY = 86_400_000;

/** The local calendar day of `instant` as `YYYY-MM-DD`. */
export function absoluteDate(instant: Date | string): string {
  const date = typeof instant === "string" ? new Date(instant) : instant;
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** `date` (`YYYY-MM-DD`) relative to `today` (`YYYY-MM-DD`): "today", "yesterday", "tomorrow", "N days ago", "in N weeks", … */
export function relativeDay(date: string, today: string): string {
  const days = Math.round((Date.parse(`${date}T00:00:00Z`) - Date.parse(`${today}T00:00:00Z`)) / MS_PER_DAY);
  if (days === 0) return "today";
  if (days === -1) return "yesterday";
  if (days === 1) return "tomorrow";
  const amount = amountOf(Math.abs(days));
  return days < 0 ? `${amount} ago` : `in ${amount}`;
}

/** `instant` relative to `now`, by local calendar days. */
export function relativeInstant(instant: Date | string, now: Date = new Date()): string {
  return relativeDay(absoluteDate(instant), absoluteDate(now));
}

/** `HH:mm` of an instant, local time. */
export function timeOfDay(instant: Date | string): string {
  const date = typeof instant === "string" ? new Date(instant) : instant;
  return `${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`;
}

function amountOf(days: number): string {
  if (days < DAYS_PER_WEEK * 2) return plural(days, "day");
  if (days < DAYS_PER_MONTH * 2) return plural(Math.floor(days / DAYS_PER_WEEK), "week");
  if (days < DAYS_PER_YEAR) return plural(Math.floor(days / DAYS_PER_MONTH), "month");
  return plural(Math.floor(days / DAYS_PER_YEAR), "year");
}

function plural(count: number, unit: string): string {
  return count === 1 ? `1 ${unit}` : `${count} ${unit}s`;
}
