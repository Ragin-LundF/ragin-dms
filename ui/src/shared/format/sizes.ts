const STEP = 1024;
const UNITS = ["KB", "MB", "GB", "TB"];

/** File sizes as people read them: 512 B, 3.4 KB, 12.0 MB. */
export function formatFileSize(bytes: number): string {
  if (bytes < STEP) return `${bytes} B`;
  let value = bytes / STEP;
  let unit = 0;
  while (value >= STEP && unit < UNITS.length - 1) {
    value /= STEP;
    unit++;
  }
  return `${value.toFixed(1)} ${UNITS[unit]}`;
}

let counts = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });

/** Formats counts for `locale` (BCP 47), the system locale Rust reports at start; invalid tags keep the default. */
export function setNumberLocale(locale: string) {
  try {
    counts = new Intl.NumberFormat(locale.replace("_", "-"), { maximumFractionDigits: 0 });
  } catch {
    // An unknown tag keeps en-US grouping.
  }
}

/** A count with thousands separators, as Kotlin's `%,d` wrote it in the system locale. */
export function formatCount(count: number): string {
  return counts.format(count);
}
