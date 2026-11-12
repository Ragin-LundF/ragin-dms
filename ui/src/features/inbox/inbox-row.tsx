import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import CONTENT_CUT from "@material-design-icons/svg/round/content_cut.svg?raw";
import DRIVE_FILE_MOVE from "@material-design-icons/svg/round/drive_file_move.svg?raw";
import MAIL from "@material-design-icons/svg/round/mail.svg?raw";
import type {InboxRowDto_Serialize} from "../../ipc";
import {type ActionButton, ActionButtons, AppCard} from "../../shared/components";
import {formatFileSize} from "../../shared/format/sizes";
import {arrivalTime} from "./time";

/** One inbox document with its actions: import a mail file, review the AI's suggestion or analyze it, split a scanned stack, or move it by hand. */
export function InboxRow({ row, aiAnalysis, busyReason, aiBusyReason, onAnalyze, onReview, onSplit, onMove, onImportEmail }: {
  row: InboxRowDto_Serialize;
  aiAnalysis: boolean;
  busyReason: string | null;
  aiBusyReason: string | null;
  onAnalyze: () => void;
  onReview: () => void;
  /** Offered for a PDF of three or more pages while the inbox proposes splits; null otherwise. */
  onSplit: (() => void) | null;
  onMove: () => void;
  onImportEmail: () => void;
}) {
  const where = [row.subfolder || null, formatFileSize(row.size ?? 0), `arrived ${arrivalTime(row.addedAt)}`].filter(Boolean).join(" · ");
  const buttons: ActionButton[] = [];
  if (row.email) {
    buttons.push({ label: "Import email", icon: MAIL, style: "tonal", disabledReason: busyReason, onClick: onImportEmail });
  }
  if (aiAnalysis && !row.email) {
    buttons.push(
      row.hasSuggestion
        ? { label: "Review", icon: AUTO_AWESOME, style: "tonal", onClick: onReview }
        : { label: "Analyze", icon: AUTO_AWESOME, style: "tonal", disabledReason: row.analyzable ? aiBusyReason : "The AI does not read this kind of file", onClick: onAnalyze },
    );
  }
  if (onSplit) buttons.push({ label: "Split stack…", icon: CONTENT_CUT, style: "tonal", disabledReason: aiBusyReason, onClick: onSplit });
  buttons.push({ label: "Move to…", icon: DRIVE_FILE_MOVE, disabledReason: busyReason, onClick: onMove });
  return (
    <AppCard padding={16}>
      <div className="row gap-l">
        <div className="column grow">
          <span className="title-small ellipsis">{row.name}</span>
          <span className="body-small muted ellipsis">{where + row.suggestionNote}</span>
        </div>
        <ActionButtons buttons={buttons} />
      </div>
    </AppCard>
  );
}
