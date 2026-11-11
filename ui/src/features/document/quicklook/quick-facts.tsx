import {type ReactNode, useState} from "react";
import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import {
    commands,
    type FieldFact_Serialize as FieldFact,
    type QuickFact,
    type QuickLook_Serialize,
    unwrap
} from "../../../ipc";
import {reportError} from "../../../session/session";
import {Icon, LabelChip, Menu, MenuItem, ReasonTooltip, useMenuAnchor} from "../../../shared/components";
import {editFact, endEdit, saveFact} from "../state";
import {InlineTextEditor} from "./inline-text-editor";
import {tagSuggestions} from "./tag-suggestions";

/**
 * The key facts as labels and values (Kotlin `QuickFactList`). A click on a value edits it in place; the category is
 * picked from a menu, and "+ tag" adds a tag with the library's tags as suggestions. While the library is busy the
 * values only say why they cannot change.
 */
export function QuickFactList({ look, editing, busyReason }: { look: QuickLook_Serialize; editing: QuickFact | null; busyReason: string | null }) {
  const { facts, documentId } = look;
  const text = (fact: QuickFact, label: string, value: string | null) => (
    <TextFact
      key={fact}
      label={label}
      value={value}
      editing={editing === fact}
      busyReason={busyReason}
      onEdit={() => editFact(documentId, fact)}
      onSave={(typed) => saveFact(documentId, fact, typed)}
    />
  );
  return (
    <div className="column">
      <CategoryFact look={look} busyReason={busyReason} />
      <FactRow label="Tags">
        <TagsValue look={look} adding={editing === "tag"} busyReason={busyReason} />
      </FactRow>
      {text("correspondent", "From", facts.correspondent)}
      {text("date", "Date", facts.documentDate)}
      {facts.amount && text("amount", facts.amount.label, facts.amount.value)}
      {facts.due &&
        (editing === "due" || facts.due.value === null ? (
          text("due", facts.due.label, facts.due.value)
        ) : (
          <DueFact due={facts.due} busyReason={busyReason} onEdit={() => editFact(documentId, "due")} />
        ))}
    </div>
  );
}

function FactRow({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="column">
      <div className="fact-row">
        <span className="body-small fact-label ellipsis">{label}</span>
        <div className="fact-value">{children}</div>
      </div>
      <div className="divider" />
    </div>
  );
}

function TextFact({ label, value, editing, busyReason, onEdit, onSave }: {
  label: string;
  value: string | null;
  editing: boolean;
  busyReason: string | null;
  onEdit: () => void;
  onSave: (typed: string) => void;
}) {
  return (
    <FactRow label={label}>
      {editing ? (
        <InlineTextEditor initial={value ?? ""} description={label} onSave={onSave} onCancel={endEdit} />
      ) : (
        <EditableValue description={`Edit ${label}`} busyReason={busyReason} onClick={onEdit}>
          <ValueText text={value} />
        </EditableValue>
      )}
    </FactRow>
  );
}

function CategoryFact({ look, busyReason }: { look: QuickLook_Serialize; busyReason: string | null }) {
  const menu = useMenuAnchor();
  const pick = (categoryId: string | null) => {
    menu.close();
    unwrap(commands.assignDocumentCategory(look.documentId, categoryId)).catch(reportError);
  };
  return (
    <FactRow label="Category">
      <EditableValue description="Edit Category" busyReason={busyReason} onClick={menu.open}>
        <ValueText text={look.facts.category} missing="None yet" />
      </EditableValue>
      {menu.anchor && (
        <Menu anchor={menu.anchor} onClose={menu.close}>
          <MenuItem label="No category" onClick={() => pick(null)} />
          {look.categories.map((category) => (
            <MenuItem key={category.id} label={<span style={{ paddingLeft: 16 * category.depth }}>{category.name}</span>} onClick={() => pick(category.id)} />
          ))}
        </Menu>
      )}
    </FactRow>
  );
}

function DueFact({ due, busyReason, onEdit }: { due: FieldFact; busyReason: string | null; onEdit: () => void }) {
  return (
    <FactRow label={due.label}>
      <EditableValue description={`Edit ${due.label}`} busyReason={busyReason} onClick={onEdit}>
        <div className="row gap-xs">
          <ValueText text={due.value} />
          {due.chip && <LabelChip text={due.chip} color={due.urgent ? "var(--status-warning)" : "var(--outline)"} />}
        </div>
      </EditableValue>
    </FactRow>
  );
}

function TagsValue({ look, adding, busyReason }: { look: QuickLook_Serialize; adding: boolean; busyReason: string | null }) {
  const { documentId, facts } = look;
  return (
    <div className="column gap-xs">
      <div className="row wrap gap-xs">
        {facts.tags.map((tag) => (
          <span key={tag} className="tag-chip label-medium">
            <span className="ellipsis">{tag}</span>
            <button
              type="button"
              aria-label={`Remove tag ${tag}`}
              disabled={busyReason !== null}
              onClick={() => unwrap(commands.removeDocumentTag(documentId, tag)).catch(reportError)}
            >
              <Icon svg={CLOSE} />
            </button>
          </span>
        ))}
        {!adding && (
          <ReasonTooltip reason={busyReason}>
            <button type="button" className="add-tag label-medium" disabled={busyReason !== null} onClick={() => editFact(documentId, "tag")}>
              + tag
            </button>
          </ReasonTooltip>
        )}
      </div>
      {adding && (
        <InlineTextEditor
          initial=""
          description="New tag"
          onSave={(typed) => saveFact(documentId, "tag", typed)}
          onCancel={endEdit}
          suggestionsFor={(typed) => tagSuggestions(typed, facts.tags, look.libraryTags)}
        />
      )}
    </div>
  );
}

/** A value that edits on click and shows "edit" while hovered; while busy, it says why it cannot. */
function EditableValue({ description, busyReason, onClick, children }: {
  description: string;
  busyReason: string | null;
  onClick: (event: React.MouseEvent<HTMLButtonElement>) => void;
  children: ReactNode;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <ReasonTooltip reason={busyReason}>
      <button
        type="button"
        className="editable-value"
        data-action={description}
        disabled={busyReason !== null}
        onClick={onClick}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
      >
        <span className="grow" style={{ minWidth: 0 }}>{children}</span>
        {hovered && busyReason === null && <span className="label-small edit-hint">edit</span>}
      </button>
    </ReasonTooltip>
  );
}

function ValueText({ text, missing = "—" }: { text: string | null; missing?: string }) {
  return <span className={text === null ? "body-small ellipsis value-missing" : "body-small ellipsis"}>{text ?? missing}</span>;
}
