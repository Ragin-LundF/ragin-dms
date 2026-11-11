import {useState} from "react";
import {useQuery} from "@tanstack/react-query";
import ADD_LINK from "@material-design-icons/svg/round/add_link.svg?raw";
import CHECK from "@material-design-icons/svg/round/check.svg?raw";
import LINK_OFF from "@material-design-icons/svg/round/link_off.svg?raw";
import OPEN_IN_NEW from "@material-design-icons/svg/round/open_in_new.svg?raw";
import PLAYLIST_ADD from "@material-design-icons/svg/round/playlist_add.svg?raw";
import RULE from "@material-design-icons/svg/round/rule.svg?raw";
import {commands, type QuickLook_Serialize, unwrap} from "../../../ipc";
import {LIBRARY_QUERY, reportError} from "../../../session/session";
import {ActionButtons, IconButton, ReasonTooltip, SectionHeader, TextField} from "../../../shared/components";
import {linkPickStore} from "../state";

const SURFACE = "var(--surface)";

function run(call: Promise<{ status: "ok"; data: null } | { status: "error"; error: { code: string; message: string } }>) {
  unwrap(call).catch(reportError);
}

/** A free-text note, saved when the field loses the focus (Kotlin `NoteEditor`). */
export function NoteSection({ look, busyReason }: { look: QuickLook_Serialize; busyReason: string | null }) {
  const saved = look.details.note ?? "";
  return (
    <>
      <SectionHeader title="Note" />
      <NoteEditor key={`${look.documentId}\n${saved}`} documentId={look.documentId} saved={saved} disabled={busyReason !== null} />
    </>
  );
}

function NoteEditor({ documentId, saved, disabled }: { documentId: string; saved: string; disabled: boolean }) {
  const [text, setText] = useState(saved);
  return (
    <TextField
      label="Note"
      placeholder="Anything worth remembering about this document"
      value={text}
      onChange={setText}
      multiline
      rows={3}
      disabled={disabled}
      labelBackground={SURFACE}
      onBlur={() => run(commands.saveDocumentNote(documentId, text))}
    />
  );
}

/** The linked documents, each to open or unlink, and "Link to…" for another (Kotlin `LinksEditor`). */
export function LinksSection({ look, busyReason }: { look: QuickLook_Serialize; busyReason: string | null }) {
  const id = look.documentId;
  const links = look.details.links;
  return (
    <>
      <SectionHeader title="Linked documents" />
      <div className="column gap-xs">
        {links.length === 0 && <span className="body-small muted">No linked documents</span>}
        {links.map((link) => (
          <div key={link.documentId} className="row">
            <div className="column grow" style={{ minWidth: 0 }}>
              <span className="body-large ellipsis">{link.name}</span>
              <span className="body-small muted ellipsis">{[link.label, link.path].filter(Boolean).join(" · ")}</span>
            </div>
            <IconButton icon={OPEN_IN_NEW} label={`Open ${link.name}`} onClick={() => run(commands.openDocumentFile(link.documentId))} />
            <ReasonTooltip reason={busyReason}>
              <IconButton icon={LINK_OFF} label={`Unlink ${link.name}`} disabled={busyReason !== null} onClick={() => run(commands.unlinkDocuments(id, link.documentId))} />
            </ReasonTooltip>
          </div>
        ))}
        <ActionButtons
          buttons={[{ label: "Link to…", icon: ADD_LINK, onClick: () => linkPickStore.set({ documentId: id, query: "", label: "" }) }]}
          enabled={busyReason === null}
          busyReason={busyReason}
        />
      </div>
    </>
  );
}

/** The documents closest in meaning, from the Ask index; nothing while it has none. */
export function RelatedSection({ documentId }: { documentId: string }) {
  const related = useQuery({
    queryKey: [LIBRARY_QUERY, "document", "related", documentId],
    queryFn: () => unwrap(commands.relatedDocumentsOf(documentId)),
  });
  if (!related.data || related.data.length === 0) return null;
  return (
    <>
      <SectionHeader title="Related documents" />
      {related.data.map((candidate) => (
        <button key={candidate.documentId} type="button" className="related-link body-medium ellipsis" onClick={() => run(commands.openDocumentFile(candidate.documentId))}>
          {candidate.name}
        </button>
      ))}
    </>
  );
}

/** One editor per custom field, saved together; a library without fields offers the standard ones (Kotlin `FieldsEditor`). */
export function FieldsSection({ look, busyReason }: { look: QuickLook_Serialize; busyReason: string | null }) {
  const fields = look.details.fields;
  const savedKey = `${look.documentId}\n${fields.map((field) => `${field.key}=${field.value}`).join("\n")}`;
  return (
    <>
      <SectionHeader title="Fields" />
      {fields.length === 0 ? (
        <>
          <span className="body-small muted">
            No custom fields yet. Add the standard ones (amount, due date, cancel by, contract end, warranty) or define your own in Settings.
          </span>
          <ActionButtons buttons={[{ label: "Add standard fields", icon: PLAYLIST_ADD, onClick: () => run(commands.addStandardFields()) }]} enabled={busyReason === null} busyReason={busyReason} />
        </>
      ) : (
        <FieldsEditor key={savedKey} look={look} busyReason={busyReason} />
      )}
    </>
  );
}

function FieldsEditor({ look, busyReason }: { look: QuickLook_Serialize; busyReason: string | null }) {
  const saved = look.details.fields;
  const [typed, setTyped] = useState(() => saved.map((field) => field.value));
  const changed = typed.some((value, index) => value !== saved[index]?.value);
  const save = () => run(commands.saveDocumentFields(look.documentId, saved.map((field, index) => ({ key: field.key, value: typed[index] ?? "" }))));
  return (
    <div className="column gap-s">
      {saved.map((field, index) => (
        <TextField
          key={field.key}
          label={field.label}
          value={typed[index] ?? ""}
          onChange={(value) => setTyped((current) => current.map((old, position) => (position === index ? value : old)))}
          labelBackground={SURFACE}
        />
      ))}
      <ActionButtons
        buttons={[{ label: "Save fields", icon: CHECK, style: "tonal", disabledReason: changed ? null : "Nothing changed", onClick: save }]}
        enabled={busyReason === null}
        busyReason={busyReason}
      />
    </div>
  );
}

export function RuleSection({ look, busyReason }: { look: QuickLook_Serialize; busyReason: string | null }) {
  return (
    <>
      <SectionHeader title="Rule" />
      <ActionButtons
        buttons={[{ label: "Create rule from this document", icon: RULE, disabledReason: look.details.ruleDisabledReason, onClick: () => run(commands.createRuleFromDocument(look.documentId)) }]}
        enabled={busyReason === null}
        busyReason={busyReason}
      />
    </>
  );
}

/** The opt-in retention: a keep-until date, or a period for the category (Kotlin `RetentionEditor`). */
export function RetentionSection({ look, busyReason }: { look: QuickLook_Serialize; busyReason: string | null }) {
  const retention = look.details.retention;
  return (
    <>
      <SectionHeader title="Retention" />
      <RetentionEditor key={`${look.documentId}\n${retention.keepUntil}\n${retention.category?.id}\n${retention.category?.years}`} look={look} busyReason={busyReason} />
    </>
  );
}

function RetentionEditor({ look, busyReason }: { look: QuickLook_Serialize; busyReason: string | null }) {
  const retention = look.details.retention;
  const category = retention.category;
  const [keepUntil, setKeepUntil] = useState(retention.keepUntil);
  const [years, setYears] = useState(category?.years ?? "");
  return (
    <div className="column gap-s">
      <span className="body-small muted">{retention.status}</span>
      <TextField label="Keep this document until (YYYY-MM-DD)" value={keepUntil} onChange={setKeepUntil} labelBackground={SURFACE} />
      {category && <TextField label={category.label} value={years} onChange={setYears} labelBackground={SURFACE} />}
      <ActionButtons
        buttons={[
          { label: "Save date", disabledReason: keepUntil === retention.keepUntil ? "Nothing changed" : null, onClick: () => run(commands.saveKeepUntil(look.documentId, keepUntil)) },
          ...(category
            ? [{ label: "Save period", disabledReason: years === category.years ? "Nothing changed" : null, onClick: () => run(commands.saveCategoryYears(category.id, years)) }]
            : []),
        ]}
        enabled={busyReason === null}
        busyReason={busyReason}
      />
    </div>
  );
}
