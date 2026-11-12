import {useState} from "react";
import ADD from "@material-design-icons/svg/round/add.svg?raw";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import CHECK from "@material-design-icons/svg/round/check.svg?raw";
import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import EDIT_NOTE from "@material-design-icons/svg/round/edit_note.svg?raw";
import HOURGLASS from "@material-design-icons/svg/round/hourglass_empty.svg?raw";
import NOTE from "@material-design-icons/svg/round/sticky_note_2.svg?raw";
import OPEN from "@material-design-icons/svg/round/open_in_new.svg?raw";
import REPLAY from "@material-design-icons/svg/round/replay.svg?raw";
import {type CaseDetailDto, type CaseTimelineItemDto, commands, unwrap} from "../../ipc";
import {busyIn, useSession} from "../../session/session";
import {Button, dialogs, Icon, IconButton, TextField, waitFor} from "../../shared/components";
import {openDocument} from "../library/navigation";
import {StatusPill} from "./case-list";
import {casesScreen, change, documentPickStore, draftCaseLetter} from "./state";

/** The chosen case: what the letter asks for, its timeline and what can be done with it. */
export function CaseDetail({ detail }: { detail: CaseDetailDto }) {
  const session = useSession();
  const busy = busyIn(session, "library");
  const busyReason = busy === null ? undefined : waitFor(busy);
  const id = detail.id;
  const setStatus = (status: "open" | "waiting" | "done") => {
    void change(unwrap(commands.casesSetStatus(id, status))).then((done) => {
      if (done && status === "done") casesScreen.set({ filter: "done", selected: id });
      if (done && status === "open") casesScreen.set({ filter: "active", selected: id });
    });
  };
  const addNote = async () => {
    const text = await dialogs.askText({ title: "Add a note", label: "Note", initialValue: "", confirmLabel: "Add note" });
    if (text?.trim()) await change(unwrap(commands.casesAddNote(id, text)));
  };
  const remove = async () => {
    const confirmed = await dialogs.confirm({
      title: `Delete the case "${detail.title}"`,
      message: "The case, its follow-ups and notes are deleted. Its documents stay in the library.",
      affected: [],
      confirmLabel: "Delete case",
    });
    if (confirmed) await change(unwrap(commands.casesDelete(id, true)));
  };
  const status = detail.pill.status;
  return (
    <section className="case-detail" aria-label={detail.title}>
      <div className="row gap-s">
        <h2 className="title-medium grow ellipsis">{detail.title}</h2>
        <StatusPill pill={detail.pill} />
      </div>
      <span className="body-small muted">{detail.summary}</span>
      {session.aiEnabled && detail.requests.map((request) => (
        <div key={request.documentId} className="case-request">
          <div className="row gap-s">
            <span className="case-ai-badge"><Icon svg={AUTO_AWESOME} /> AI</span>
            <span className="label-large grow">The letter asks you to do something</span>
            <IconButton icon={OPEN} label={`Open ${request.documentName}`} small tooltip onClick={() => openDocument(request.documentId)} />
          </div>
          <ul className="body-medium case-request-actions">
            {request.actions.map((action) => <li key={action}>{action}</li>)}
          </ul>
          <span className="body-small muted">From {request.documentName}; each request is a follow-up below.</span>
        </div>
      ))}
      <div className="case-timeline">
        {detail.timeline.map((item, index) => (
          <TimelineItem key={`${item.kind}-${item.documentId ?? item.followUpId ?? index}`} caseId={id} item={item} busyReason={busyReason} />
        ))}
        {detail.timeline.length === 0 && <span className="body-medium muted">Nothing in this case yet. Add a document or a follow-up.</span>}
      </div>
      <FollowUpForm caseId={id} busyReason={busyReason} />
      <div className="row gap-s case-actions">
        {status === "waiting" && (
          <Button variant="tonal" icon={EDIT_NOTE} onClick={() => draftCaseLetter({ caseId: id, documentIds: detail.reminder.documentIds, recipient: detail.reminder.recipient ?? null })}>
            Draft reminder
          </Button>
        )}
        <Button icon={ADD} disabled={busy !== null} title={busyReason} onClick={() => documentPickStore.set({ caseId: id, query: "" })}>Add document</Button>
        <Button icon={NOTE} disabled={busy !== null} title={busyReason} onClick={() => void addNote()}>Add note</Button>
        {status === "done" ? (
          <Button icon={REPLAY} disabled={busy !== null} title={busyReason} onClick={() => setStatus("open")}>Reopen</Button>
        ) : (
          <>
            {status !== "waiting" && <Button icon={HOURGLASS} disabled={busy !== null} title={busyReason} onClick={() => setStatus("waiting")}>Waiting for answer</Button>}
            <Button icon={CHECK} disabled={busy !== null} title={busyReason} onClick={() => setStatus("done")}>Mark as done</Button>
          </>
        )}
        <span className="grow" />
        <Button variant="text" icon={DELETE} danger disabled={busy !== null} title={busyReason} onClick={() => void remove()}>Delete…</Button>
      </div>
    </section>
  );
}

/** One step: a document opens and can leave the case; an open follow-up is done or snoozed. */
function TimelineItem({ caseId, item, busyReason }: { caseId: string; item: CaseTimelineItemDto; busyReason: string | undefined }) {
  const classes = ["case-step", item.kind, item.done ? "done" : "", item.due ? "due" : ""].filter(Boolean).join(" ");
  const disabled = busyReason !== undefined;
  return (
    <div className={classes}>
      <span className="body-small muted">{item.marker}{item.due ? " · due" : ""}</span>
      <div className="row gap-s">
        <span className="body-medium grow case-step-title">{item.title}</span>
        {item.documentId && (
          <>
            <IconButton icon={OPEN} label="Open the document" small tooltip onClick={() => item.documentId && openDocument(item.documentId)} />
            <IconButton icon={CLOSE} label="Remove from the case" small tooltip disabled={disabled}
              onClick={() => item.documentId && void change(unwrap(commands.casesRemoveDocument(caseId, item.documentId)))} />
          </>
        )}
        {item.followUpId && !item.done && (
          <>
            <Button small variant="text" disabled={disabled} title={busyReason}
              onClick={() => item.followUpId && void change(unwrap(commands.casesSnoozeFollowUp(caseId, item.followUpId, 7)))}>Snooze 7 days</Button>
            <Button small icon={CHECK} disabled={disabled} title={busyReason}
              onClick={() => item.followUpId && void change(unwrap(commands.casesCompleteFollowUp(caseId, item.followUpId)))}>Done</Button>
          </>
        )}
      </div>
      {item.detail && <span className="body-small muted">{item.detail}</span>}
    </div>
  );
}

/** A new follow-up: what to do and by when, a week from today unless changed. */
function FollowUpForm({ caseId, busyReason }: { caseId: string; busyReason: string | undefined }) {
  const [text, setText] = useState("");
  const [date, setDate] = useState(() => isoDaysFromToday(7));
  const add = () =>
    void change(unwrap(commands.casesAddFollowUp(caseId, { text, date }))).then((done) => done && setText(""));
  return (
    <div className="row gap-s case-follow-up-form">
      <TextField label="New follow-up" value={text} onChange={setText} onEnter={() => text.trim() && add()} />
      <TextField label="By (YYYY-MM-DD)" value={date} onChange={setDate} />
      <Button icon={ADD} disabled={text.trim() === "" || busyReason !== undefined} title={busyReason} onClick={add}>Add follow-up</Button>
    </div>
  );
}

/** Today plus `days` in the local calendar, as YYYY-MM-DD. */
export function isoDaysFromToday(days: number, today: Date = new Date()): string {
  const date = new Date(today.getFullYear(), today.getMonth(), today.getDate() + days);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}
