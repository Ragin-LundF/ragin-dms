import {useQuery} from "@tanstack/react-query";
import ADD from "@material-design-icons/svg/round/add.svg?raw";
import FORUM from "@material-design-icons/svg/round/forum.svg?raw";
import LINK from "@material-design-icons/svg/round/link.svg?raw";
import {commands, unwrap} from "../../../../ipc";
import {LIBRARY_QUERY, reportError} from "../../../../session/session";
import {Button, Chip, dialogs, Menu, MenuDivider, MenuItem, useMenuAnchor} from "../../../../shared/components";
import {openCase, startCase} from "../../../cases";
import type {OrganizeRowProps} from "./organize-block";

/** The cases of a document as chips, "Add to case…" and, for a linked document, a case from its links. */
export function CaseRow({ documentId, busyReason }: OrganizeRowProps) {
  const row = useQuery({
    queryKey: [LIBRARY_QUERY, "cases", "of-document", documentId],
    queryFn: () => unwrap(commands.casesOfDocument(documentId)),
  });
  const menu = useMenuAnchor();
  if (!row.data) return null;
  const busy = busyReason !== null;
  const addTo = (caseId: string) => {
    menu.close();
    unwrap(commands.casesAddDocument(caseId, documentId, "received")).catch(reportError);
  };
  const fromLinks = () =>
    unwrap(commands.casesCreateFromLinks(documentId)).then((caseId) => openCase(caseId), reportError);
  return (
    <div className="row gap-s quick-look-cases">
      <span className="label-medium muted">Cases</span>
      {row.data.cases.map((entry) => (
        <Chip key={entry.id} icon={FORUM} label={entry.title} onClick={() => openCase(entry.id, entry.status === "done")} />
      ))}
      <Button small variant="text" icon={ADD} disabled={busy} title={busyReason ?? undefined} onClick={menu.open}>Add to case…</Button>
      {row.data.hasLinks && (
        <Button small variant="text" icon={LINK} disabled={busy} title={busyReason ?? undefined} onClick={fromLinks}>Create case from linked documents</Button>
      )}
      {menu.anchor && (
        <Menu anchor={menu.anchor} onClose={menu.close}>
          {row.data.candidates.map((candidate) => (
            <MenuItem key={candidate.id} label={candidate.title} icon={FORUM} onClick={() => addTo(candidate.id)} />
          ))}
          {row.data.candidates.length > 0 && <MenuDivider />}
          <MenuItem label="New case…" icon={ADD} onClick={() => { menu.close(); void newCase(documentId); }} />
        </Menu>
      )}
    </div>
  );
}

async function newCase(documentId: string) {
  const title = await dialogs.askText({ title: "New case", label: "Title", initialValue: "", confirmLabel: "Start case" });
  if (title?.trim()) await startCase(title, null, [documentId]);
}
