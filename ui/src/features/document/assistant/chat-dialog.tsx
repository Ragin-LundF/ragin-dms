import { useEffect, useRef } from "react";
import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import DELETE_SWEEP from "@material-design-icons/svg/round/delete_sweep.svg?raw";
import SEND from "@material-design-icons/svg/round/send.svg?raw";
import STOP from "@material-design-icons/svg/round/stop.svg?raw";
import { Dialog, IconButton, Spinner, TextField } from "../../../shared/components";
import { useStore } from "../../../shared/store";
import { chatStore, closeChat, sendQuestion, startOver, stopAnswer, typeQuestion } from "../state";

const STARTERS = ["What is this document about?", "Do I have to do anything, and by when?", "Which amounts are mentioned?", "Does this matter for my tax return?"];
const MAX_INPUT_LINES = 4;

/** A conversation with the AI about one document; answers come from the document only (Kotlin `ChatDialog`). */
export function ChatDialog() {
  const chat = useStore(chatStore);
  const list = useRef<HTMLDivElement>(null);
  const turns = chat?.turns.length ?? 0;
  const waiting = chat?.waiting ?? false;
  useEffect(() => {
    list.current?.lastElementChild?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [turns, waiting]);
  if (!chat) return null;
  const lines = Math.min(MAX_INPUT_LINES, Math.max(1, chat.input.split("\n").length));
  return (
    <Dialog onDismiss={closeChat} className="chat-dialog">
      <div className="row">
        <div className="column grow" style={{ minWidth: 0 }}>
          <span className="title-large">Ask AI</span>
          <span className="body-small muted">{`About ${chat.fileName}. Answers come from the document; check what matters.`}</span>
        </div>
        <IconButton icon={DELETE_SWEEP} label="Start over" disabled={chat.turns.length === 0} onClick={startOver} />
        <IconButton icon={CLOSE} label="Close" onClick={closeChat} />
      </div>
      <div ref={list} className="chat-turns scroll">
        {chat.turns.length === 0 && !chat.waiting && (
          <div className="row wrap gap-s">
            {STARTERS.map((question) => (
              <button key={question} type="button" className="assist-chip label-large" onClick={() => sendQuestion(question)}>{question}</button>
            ))}
          </div>
        )}
        {chat.turns.map((turn, index) => (
          <div key={index} className={turn.role === "user" ? "chat-bubble-row mine" : "chat-bubble-row"}>
            <span className="chat-bubble body-large selectable pre-wrap">{turn.text}</span>
          </div>
        ))}
        {chat.waiting && <Spinner small />}
      </div>
      {chat.problem && <span className="body-medium error-text">{chat.problem}</span>}
      <div className="row gap-s">
        <TextField
          placeholder="Ask about this document"
          value={chat.input}
          onChange={typeQuestion}
          multiline
          rows={lines}
          onEnter={() => sendQuestion()}
          style={{ flex: 1 }}
        />
        {chat.waiting ? (
          <IconButton icon={STOP} label="Stop" className="filled-icon-button" onClick={stopAnswer} />
        ) : (
          <IconButton icon={SEND} label="Send" className="filled-icon-button" disabled={chat.input.trim() === ""} onClick={() => sendQuestion()} />
        )}
      </div>
    </Dialog>
  );
}
