import { describe, expect, it } from "vitest";
import { relativeDay } from "./dates";
import { formatCount, formatFileSize, setNumberLocale } from "./sizes";

describe("relativeDay", () => {
  it("names the neighbouring days", () => {
    expect(relativeDay("2026-03-10", "2026-03-10")).toBe("today");
    expect(relativeDay("2026-03-09", "2026-03-10")).toBe("yesterday");
    expect(relativeDay("2026-03-11", "2026-03-10")).toBe("tomorrow");
  });

  it("switches from days to weeks, months and years like the Kotlin app", () => {
    expect(relativeDay("2026-02-25", "2026-03-10")).toBe("13 days ago");
    expect(relativeDay("2026-03-24", "2026-03-10")).toBe("in 2 weeks");
    expect(relativeDay("2026-01-01", "2026-03-10")).toBe("2 months ago");
    expect(relativeDay("2024-03-10", "2026-03-10")).toBe("2 years ago");
  });

  it("crosses a daylight saving change without losing a day", () => {
    expect(relativeDay("2026-03-30", "2026-03-28")).toBe("in 2 days");
  });
});

describe("formatFileSize", () => {
  it("keeps bytes below one kilobyte and rounds above", () => {
    expect(formatFileSize(512)).toBe("512 B");
    expect(formatFileSize(3482)).toBe("3.4 KB");
    expect(formatFileSize(12 * 1024 * 1024)).toBe("12.0 MB");
  });
});

describe("formatCount", () => {
  it("groups digits like the JVM for the system locale", () => {
    setNumberLocale("en-US");
    expect(formatCount(1234567)).toBe("1,234,567");
    setNumberLocale("de-DE");
    expect(formatCount(1234)).toBe("1.234");
    setNumberLocale("not a locale!");
    expect(formatCount(1234)).toBe("1.234");
    setNumberLocale("en-US");
  });
});
