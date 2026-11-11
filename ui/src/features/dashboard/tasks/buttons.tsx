import type {ReactNode} from "react";
import type {DuplicateCopyDto_Serialize, TaskFileDto} from "../../../ipc";
import {useSession} from "../../../session/session";
import {Button, FileThumbnail, ReasonTooltip} from "../../../shared/components";
import {type DashboardButton, disabledReason, runButton} from "../actions";
import {iconOf} from "./icons";

/** One computed button; a disabled one names why in its tooltip. */
export function DashButton({ button }: { button: DashboardButton }) {
  const reason = disabledReason(button, useSession());
  return (
    <ReasonTooltip reason={reason}>
      <Button variant={button.style} icon={iconOf(button.icon)} disabled={reason !== null} onClick={() => void runButton(button)}>
        {button.label}
      </Button>
    </ReasonTooltip>
  );
}

/** A row of buttons that wraps when narrow (Kotlin `ActionButtons`); `inline` puts them into the parent's row. */
export function DashButtons({ buttons, className, inline }: { buttons: DashboardButton[]; className?: string; inline?: boolean }) {
  if (buttons.length === 0) return null;
  return (
    <div className={[inline ? "contents" : "row wrap gap-s", className ?? ""].join(" ")}>
      {buttons.map((button) => (
        <DashButton key={button.label} button={button} />
      ))}
    </div>
  );
}

/** The copies of a duplicates task: path and what is assigned, then Open and Keep side by side (Kotlin `DuplicateCopies`). */
export function DuplicateCopies({ copies }: { copies: DuplicateCopyDto_Serialize[] }) {
  if (copies.length === 0) return null;
  return (
    <div className="column gap-s duplicate-copies">
      {copies.map((copy) => (
        <div key={copy.path} className="row gap-s">
          <div className="column grow">
            <span className="body-medium ellipsis">{copy.path}</span>
            <span className="body-small muted clamp-2">{copy.detail}</span>
          </div>
          <div className="row gap-s">
            <DashButton button={copy.open} />
            <DashButton button={copy.keep} />
          </div>
        </div>
      ))}
    </div>
  );
}

/** What a click on a row's thumbnail does: its first Open button, else the first copy's (Kotlin rows). */
export function firstOpen(opens: DashboardButton[], copies: DuplicateCopyDto_Serialize[]): DashboardButton | undefined {
  return opens[0] ?? copies[0]?.open;
}

/** The small page picture at the start of a row, 32 × 40, or `fallback` (Kotlin `RowThumbnail`). */
export function RowThumbnail({ file, onClick, fallback }: { file: TaskFileDto | null | undefined; onClick: () => void; fallback: ReactNode }) {
  return (
    <div className="row-thumbnail">
      {file ? <FileThumbnail folderId={file.folderId} path={file.path} onClick={onClick} fallback={fallback} /> : fallback}
    </div>
  );
}
