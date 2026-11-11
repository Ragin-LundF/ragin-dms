import { describe, expect, it } from "vitest";
import { answer, ask, clear, newChat, stop } from "./chat-state";

describe("chat about a document", () => {
  const started = newChat("d-1", "bill.pdf");

  it("sends a trimmed question and waits; blank input or a second question while waiting sends nothing", () => {
    const waiting = ask(started, "  What is due?  ");
    expect(waiting?.turns).toEqual([{ role: "user", text: "What is due?" }]);
    expect(waiting?.waiting).toBe(true);
    expect(waiting?.input).toBe("");
    expect(ask(started, "   ")).toBeNull();
    expect(ask(waiting!, "another")).toBeNull();
  });

  it("adds the answer, or takes the question back into the input with the problem", () => {
    const waiting = ask(started, "What is due?")!;
    const answered = answer(waiting, waiting.asked, { text: "The bill, by Friday.", problem: null });
    expect(answered.turns.map((turn) => turn.role)).toEqual(["user", "assistant"]);
    expect(answered.waiting).toBe(false);

    const failed = answer(waiting, waiting.asked, { text: null, problem: "The AI took too long" });
    expect(failed.turns).toEqual([]);
    expect(failed.input).toBe("What is due?");
    expect(failed.problem).toBe("The AI took too long");
  });

  it("stop puts the unanswered question back and drops its late answer", () => {
    const waiting = ask(started, "What is due?")!;
    const stopped = stop(waiting);
    expect(stopped.turns).toEqual([]);
    expect(stopped.input).toBe("What is due?");
    expect(answer(stopped, waiting.asked, { text: "late", problem: null })).toBe(stopped);
  });

  it("start over clears the turns and the problem", () => {
    const answered = answer(ask(started, "Q")!, 1, { text: "A", problem: null });
    const cleared = clear({ ...answered, problem: "old" });
    expect(cleared.turns).toEqual([]);
    expect(cleared.problem).toBeNull();
  });
});
