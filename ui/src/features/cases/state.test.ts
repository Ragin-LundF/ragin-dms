import {describe, expect, it} from "vitest";
import type {CasesReviewDto} from "../../ipc";
import {reviewChoiceOf} from "./state";

const review: CasesReviewDto = {
  requested: [{ text: "Send the invoices", date: "2026-11-30" }, { text: "Reply", date: "2026-11-25" }],
  matches: [],
  openCase: { id: "case-1", title: "Objection", status: "waiting" },
  preChecked: true,
  newTitle: "Your objection",
};

describe("reviewChoiceOf", () => {
  it("sends the checked requests to the open case or a new one", () => {
    const state = { checked: [true, false], target: "existing" as const, join: false };
    expect(reviewChoiceOf(review, true, state)).toEqual({ kind: "addToCase", caseId: "case-1", followUps: [review.requested[0]] });
    expect(reviewChoiceOf(review, true, { ...state, target: "new" })).toEqual({ kind: "newCase", title: "Your objection", followUps: [review.requested[0]] });
  });

  it("without AI or checked requests only joins the open case when ticked", () => {
    expect(reviewChoiceOf(review, false, { checked: [true, true], target: "new", join: true })).toEqual({ kind: "addToCase", caseId: "case-1", followUps: [] });
    expect(reviewChoiceOf(review, true, { checked: [false, false], target: "new", join: false })).toBeNull();
    expect(reviewChoiceOf({ ...review, openCase: null }, false, { checked: [], target: "new", join: true })).toBeNull();
  });
});
