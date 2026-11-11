import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import DRIVE_FILE_MOVE from "@material-design-icons/svg/round/drive_file_move.svg?raw";
import type {InboxRowDto_Serialize} from "../../ipc";
import {type ActionButton, ActionButtons, AppCard} from "../../shared/components";
import {formatFileSize} from "../../shared/format/sizes";
import {arrivalTime} from "./time";

/** One inbox document with its actions: review the AI's suggestion or analyze it, or move it by hand. */
export function InboxRow({ row, aiAnalysis, busyReason, aiBusyReason, onAnalyze, onReview, onMove }: {
  row: InboxRowDto_Serialize;
  aiAnalysis: boolean;
  busyReason: string | null;
  aiBusyReason: string | null;
  onAnalyze: () => void;
  onReview: () => void;
  onMove: () => void;
}) {
  const where = [row.subfolder || null, formatFileSize(row.size ?? 0), `arrived ${arrivalTime(row.addedAt)}`].filter(Boolean).join(" · ");
  const buttons: ActionButton[] = [];
  if (aiAnalysis) {
    buttons.push(
      row.hasSuggestion
        ? { label: "Review", icon: AUTO_AWESOME, style: "tonal", onClick: onReview }
        : { label: "Analyze", icon: AUTO_AWESOME, style: "tonal", disabledReason: row.analyzable ? aiBusyReason : "The AI does not read this kind of file", onClick: onAnalyze },
    );
  }
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
