import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import {ActionButtons, Icon, LabelChip} from "../../../shared/components";

/**
 * A suggestion of the AI that waits for this document's review, in the AI's gradient frame (Kotlin `SuggestionCard`):
 * its values as chips, "Accept all" to apply exactly these values, "Edit…" to open the review and "Dismiss" to
 * reject it. Accepting and dismissing wait for a busy library.
 */
export function SuggestionCard({ values, busyReason, onAccept, onEdit, onDismiss }: {
  values: string[];
  busyReason: string | null;
  onAccept: () => void;
  onEdit: () => void;
  onDismiss: () => void;
}) {
  return (
    <div className="suggestion-card">
      <div className="row gap-s">
        <span className="primary-text"><Icon svg={AUTO_AWESOME} size={18} /></span>
        <span className="title-small">Suggested by AI</span>
      </div>
      <div className="row wrap gap-xs">
        {values.map((value) => <LabelChip key={value} text={value} color="var(--primary)" />)}
      </div>
      <ActionButtons
        buttons={[
          { label: "Accept all", style: "filled", disabledReason: busyReason, onClick: onAccept },
          { label: "Edit…", style: "outlined", onClick: onEdit },
          { label: "Dismiss", style: "text", disabledReason: busyReason, onClick: onDismiss },
        ]}
      />
    </div>
  );
}
