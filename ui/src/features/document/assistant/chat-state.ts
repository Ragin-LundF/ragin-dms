import type { ChatMessage } from "../../../ipc";

/**
 * The conversation about one document (Kotlin `ChatStateModel`): the turns so far, the question being typed, whether
 * the AI is answering, and the problem of the last question, which is then back in the input to send again.
 */
export type Chat = {
  documentId: string;
  fileName: string;
  turns: ChatMessage[];
  input: string;
  waiting: boolean;
  problem: string | null;
  /** Counts the questions, so an answer to a stopped or cleared question is dropped. */
  asked: number;
};

export function newChat(documentId: string, fileName: string): Chat {
  return { documentId, fileName, turns: [], input: "", waiting: false, problem: null, asked: 0 };
}

/** The chat after sending `question`; null when there is nothing to send or the AI still answers. */
export function ask(chat: Chat, question: string): Chat | null {
  const text = question.trim();
  if (text === "" || chat.waiting) return null;
  return { ...chat, turns: [...chat.turns, { role: "user", text }], input: "", waiting: true, problem: null, asked: chat.asked + 1 };
}

/** The AI's answer `text`, or its `problem`: then the question is taken back and returns to the input. */
export function answer(chat: Chat, asked: number, result: { text: string | null; problem: string | null }): Chat {
  if (!chat.waiting || chat.asked !== asked) return chat;
  if (result.text !== null) return { ...chat, turns: [...chat.turns, { role: "assistant", text: result.text }], waiting: false };
  return { ...takeBack(chat), problem: result.problem };
}

/** Stops the AI; the unanswered question goes back into the input. */
export function stop(chat: Chat): Chat {
  return chat.waiting ? takeBack(chat) : chat;
}

/** Start over: no turns, nothing waiting. */
export function clear(chat: Chat): Chat {
  return { ...stop(chat), turns: [], problem: null };
}

function takeBack(chat: Chat): Chat {
  const last = chat.turns[chat.turns.length - 1];
  const question = last?.role === "user" ? last : null;
  return {
    ...chat,
    turns: question ? chat.turns.slice(0, -1) : chat.turns,
    input: question?.text ?? chat.input,
    waiting: false,
    asked: chat.asked + 1,
  };
}
