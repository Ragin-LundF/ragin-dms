import CHEVRON_RIGHT from "@material-design-icons/svg/round/chevron_right.svg?raw";
import EXPAND_MORE from "@material-design-icons/svg/round/expand_more.svg?raw";
import type {QuickLook_Serialize} from "../../../ipc";
import {Icon} from "../../../shared/components";
import {PdfSection} from "../../pdf";
import {
    FieldsSection,
    LinksSection,
    NoteSection,
    RelatedSection,
    RetentionSection,
    RuleSection
} from "./details-sections";

/**
 * The rarely used parts, folded so the first view stays short (Kotlin `MoreSection`): note and links, fields, rule
 * and retention, and the PDF tools. It stays open or folded while the quick look moves on.
 */
export function MoreSection({ look, open, onToggle, busyReason }: { look: QuickLook_Serialize; open: boolean; onToggle: () => void; busyReason: string | null }) {
  return (
    <div className="column gap-m">
      <button type="button" className="more-toggle" aria-expanded={open} onClick={onToggle}>
        <span className="muted-icon"><Icon svg={open ? EXPAND_MORE : CHEVRON_RIGHT} size={18} /></span>
        <span className="title-small">Notes, links, retention, pages</span>
      </button>
      {open && (
        <div className="column gap-m">
          <NoteSection look={look} busyReason={busyReason} />
          <LinksSection look={look} busyReason={busyReason} />
          <RelatedSection documentId={look.documentId} />
          <FieldsSection look={look} busyReason={busyReason} />
          <RuleSection look={look} busyReason={busyReason} />
          <RetentionSection look={look} busyReason={busyReason} />
          <PdfSection documentId={look.documentId} />
        </div>
      )}
    </div>
  );
}
