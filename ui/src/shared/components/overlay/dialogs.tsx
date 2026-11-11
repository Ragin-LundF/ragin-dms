import { useState, useSyncExternalStore } from "react";
import { Button } from "../controls/buttons";
import { Dialog } from "./dialog";
import { TextField } from "../controls/fields";

/**
 * A question before a destructive action. `affected` names every file or folder the action touches, so the user
 * sees what will happen before confirming.
 */
export type ConfirmationRequest = { title: string; message: string; affected: string[]; confirmLabel: string };

/** A request for one line of text, such as a new name. */
export type TextInputRequest = { title: string; label: string; initialValue: string; confirmLabel: string };

type Open =
  | { kind: "confirm"; request: ConfirmationRequest; resolve: (confirmed: boolean) => void }
  | { kind: "text"; request: TextInputRequest; resolve: (text: string | null) => void };

let open: Open | null = null;
const listeners = new Set<() => void>();
function set(next: Open | null) {
  open = next;
  listeners.forEach((listener) => listener());
}

/**
 * The open question of the app (Kotlin's `DialogController` + `DialogHost`): one at a time. The action runs only
 * when the user confirms or submits.
 */
export const dialogs = {
  confirm(request: ConfirmationRequest): Promise<boolean> {
    open?.kind === "confirm" ? open.resolve(false) : open?.resolve(null);
    return new Promise((resolve) => set({ kind: "confirm", request, resolve }));
  },
  askText(request: TextInputRequest): Promise<string | null> {
    open?.kind === "confirm" ? open.resolve(false) : open?.resolve(null);
    return new Promise((resolve) => set({ kind: "text", request, resolve }));
  },
};

/** Shows the open question, if there is one; mounted once by the shell. */
export function DialogHost() {
  const current = useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => open,
  );
  if (!current) return null;
  if (current.kind === "confirm") {
    const answer = (confirmed: boolean) => {
      set(null);
      current.resolve(confirmed);
    };
    return <ConfirmDialog request={current.request} onConfirm={() => answer(true)} onDismiss={() => answer(false)} />;
  }
  const answer = (text: string | null) => {
    set(null);
    current.resolve(text);
  };
  return <TextInputDialog request={current.request} onSubmit={answer} onDismiss={() => answer(null)} />;
}

/** Asks before a destructive action and lists everything it affects. */
export function ConfirmDialog({ request, onConfirm, onDismiss }: { request: ConfirmationRequest; onConfirm: () => void; onDismiss: () => void }) {
  return (
    <Dialog
      title={request.title}
      onDismiss={onDismiss}
      buttons={
        <>
          <Button variant="text" onClick={onDismiss}>Cancel</Button>
          <Button variant="text" onClick={onConfirm}>{request.confirmLabel}</Button>
        </>
      }
    >
      <div className="dialog-text">
        <p>{request.message}</p>
        <div className="scroll" style={{ maxHeight: 240 }}>
          {request.affected.map((entry) => (
            <div key={entry} className="body-small ellipsis" style={{ minHeight: 16 }}>• {entry}</div>
          ))}
        </div>
      </div>
    </Dialog>
  );
}

/** Asks for one line of text; the caller validates what is submitted. */
export function TextInputDialog({ request, onSubmit, onDismiss }: { request: TextInputRequest; onSubmit: (text: string) => void; onDismiss: () => void }) {
  const [value, setValue] = useState(request.initialValue);
  const valid = value.trim().length > 0;
  return (
    <Dialog
      title={request.title}
      onDismiss={onDismiss}
      buttons={
        <>
          <Button variant="text" onClick={onDismiss}>Cancel</Button>
          <Button variant="text" disabled={!valid} onClick={() => onSubmit(value)}>{request.confirmLabel}</Button>
        </>
      }
    >
      <TextField
        label={request.label}
        value={value}
        onChange={setValue}
        autoFocus
        labelBackground="var(--surface-high)"
        onEnter={() => valid && onSubmit(value)}
      />
    </Dialog>
  );
}
