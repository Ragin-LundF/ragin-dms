import HISTORY from "@material-design-icons/svg/round/history.svg?raw";
import type {RecentDocumentDto} from "../../../ipc";
import {AppCard, CardTitle, FullViewButton, KindBadge, ReasonTooltip} from "../../../shared/components";
import {RowThumbnail} from "../tasks/buttons";

/** The documents added last, with a thumbnail from the primary and when they were added (Kotlin `RecentDocumentsCard`). */
export function RecentDocumentsCard({ documents, primaryFolderId, onOpen, onShowAll }: {
  documents: RecentDocumentDto[];
  primaryFolderId: string | null;
  onOpen: (documentId: string) => void;
  onShowAll: () => void;
}) {
  return (
    <AppCard className="recent-card">
      <CardTitle title="Recently added" icon={HISTORY} tint="var(--primary)" trailing={<FullViewButton label="Show all recently added documents" onClick={onShowAll} />} />
      {documents.length === 0 && <span className="body-medium muted">Drop files anywhere on this page to add them</span>}
      {documents.map((document) => (
        <div key={document.id} role="button" tabIndex={0} className="recent-row" onClick={() => onOpen(document.id)} onKeyDown={(event) => event.key === "Enter" && onOpen(document.id)}>
          <RowThumbnail
            file={primaryFolderId ? { folderId: primaryFolderId, path: document.path } : null}
            onClick={() => onOpen(document.id)}
            fallback={<KindBadge kind={document.kind} size={32} />}
          />
          <div className="column grow">
            <span className="body-medium ellipsis">{document.title}</span>
            <span className="body-small muted ellipsis">{document.subtitle}</span>
          </div>
          <ReasonTooltip reason={document.addedOn}>
            <span className="label-medium muted nowrap">{document.added}</span>
          </ReasonTooltip>
        </div>
      ))}
    </AppCard>
  );
}
