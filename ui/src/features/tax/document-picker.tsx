import {useState} from "react";
import {useQuery} from "@tanstack/react-query";
import {commands} from "../../ipc";
import {LIBRARY_QUERY} from "../../session/session";
import {Button, Dialog, SearchField} from "../../shared/components";

/** "I have it": picks the library document for an item, the item's sender first; picking links and assigns it. */
export function TaxDocumentPicker({ year, itemId, itemLabel, onPick, onDismiss }: {
  year: number;
  itemId: string;
  itemLabel: string;
  onPick: (documentId: string) => void;
  onDismiss: () => void;
}) {
  const [query, setQuery] = useState("");
  const choices = useQuery({ queryKey: [LIBRARY_QUERY, "tax-choices", year, itemId], queryFn: () => commands.taxDocumentChoices(year, itemId) });
  const wanted = query.trim().toLowerCase();
  const list = choices.data?.status === "ok" ? choices.data.data : [];
  const shown = list.filter((choice) => [choice.name, choice.correspondent ?? "", choice.date ?? ""].some((text) => text.toLowerCase().includes(wanted)));
  return (
    <Dialog title={`Which document is "${itemLabel}"?`} width={560} onDismiss={onDismiss} buttons={<Button variant="text" onClick={onDismiss}>Cancel</Button>}>
      <div className="column gap-s">
        <span className="body-small muted">The document is ticked off and assigned to the tax year {year} and this area.</span>
        <SearchField query={query} onQueryChange={setQuery} placeholder="Search by name, sender or date" />
        <div className="tax-choices">
          {shown.length === 0 && <span className="body-small muted">No document matches</span>}
          {shown.map((choice) => (
            <button key={choice.id} type="button" className="tax-choice" onClick={() => onPick(choice.id)}>
              <span className="body-medium tax-ellipsis">{choice.name}</span>
              <span className="body-small muted">{[choice.correspondent, choice.date].filter(Boolean).join(" · ")}</span>
            </button>
          ))}
        </div>
      </div>
    </Dialog>
  );
}
