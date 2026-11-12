import ADD from "@material-design-icons/svg/round/add.svg?raw";
import AUTO from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import CATEGORY from "@material-design-icons/svg/round/category.svg?raw";
import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import MOVE from "@material-design-icons/svg/round/drive_file_move.svg?raw";
import OFFER from "@material-design-icons/svg/round/local_offer.svg?raw";
import RECEIPT from "@material-design-icons/svg/round/receipt_long.svg?raw";
import {useState} from "react";
import {TaxAssignDialog} from "../../tax";
import {Icon, IconButton, Menu, MenuDivider, MenuItem, ReasonTooltip, useMenuAnchor} from "../../../shared/components";
import {bulk, chosenIds} from "../state/actions";
import type {LibraryState} from "../state/queries";
import {selection} from "../state/selection";

/**
 * The floating bar while documents are ticked (Kotlin `SelectionBar`): "N selected · Category · Tag · Move · Tax year ·
 * Tag with AI · Trash… · ×". The library's actions wait for a busy library, "Tag with AI" for a busy AI.
 */
export function SelectionBar({ state, count, busyReason, aiBusyReason, aiEnabled }: {
  state: LibraryState;
  count: number;
  busyReason: string | null;
  aiBusyReason: string | null;
  aiEnabled: boolean;
}) {
  const categories = useMenuAnchor();
  const tags = useMenuAnchor();
  const tagOptions = state.facets.find((entry) => entry.facet === "tag")?.options ?? [];
  const [taxIds, setTaxIds] = useState<string[] | null>(null);
  return (
    <div className="lib-bulk">
      <span className="nowrap" style={{ fontWeight: 600, paddingRight: 8 }}>{count} selected</span>
      <BarButton label="Category" icon={CATEGORY} disabledReason={busyReason} onClick={categories.open} />
      {categories.anchor && (
        <Menu anchor={categories.anchor} onClose={categories.close}>
          {state.categories.map((entry) => (
            <MenuChoice key={entry.id} text={entry.name} depth={entry.depth} onClick={() => { categories.close(); bulk.assignCategory(entry.id); }} />
          ))}
          {state.categories.length > 0 && <MenuDivider />}
          <MenuChoice text="Without a category" depth={0} italic onClick={() => { categories.close(); bulk.assignCategory(null); }} />
        </Menu>
      )}
      <BarButton label="Tag" icon={OFFER} disabledReason={busyReason} onClick={tags.open} />
      {tags.anchor && (
        <Menu anchor={tags.anchor} onClose={tags.close}>
          {tagOptions.map((option) =>
            option.filter.kind === "tag" ? <MenuChoice key={option.label} text={option.label} depth={0} onClick={() => { tags.close(); bulk.addTag(option.label); }} /> : null,
          )}
          {tagOptions.length > 0 && <MenuDivider />}
          <MenuItem label="New tag…" icon={ADD} onClick={() => { tags.close(); void bulk.newTag(); }} />
        </Menu>
      )}
      <BarButton label="Move" icon={MOVE} disabledReason={busyReason} onClick={bulk.move} />
      <BarButton label="Tax year" icon={RECEIPT} disabledReason={busyReason} onClick={() => setTaxIds(chosenIds())} />
      {taxIds && <TaxAssignDialog documentIds={taxIds} onClose={() => setTaxIds(null)} />}
      {aiEnabled && <BarButton label="Tag with AI" icon={AUTO} disabledReason={aiBusyReason} ai onClick={bulk.tagWithAi} />}
      <BarButton label="Trash…" icon={DELETE} disabledReason={busyReason} onClick={() => void bulk.trash()} />
      <IconButton icon={CLOSE} label="Clear the selection" onClick={selection.clear} />
    </div>
  );
}

function BarButton({ label, icon, disabledReason, ai = false, onClick }: { label: string; icon: string; disabledReason: string | null; ai?: boolean; onClick: (event: React.MouseEvent) => void }) {
  return (
    <ReasonTooltip reason={disabledReason}>
      <button type="button" className={ai ? "lib-bar-button ai" : "lib-bar-button"} disabled={disabledReason !== null} onClick={onClick}>
        <Icon svg={icon} />
        <span>{label}</span>
      </button>
    </ReasonTooltip>
  );
}

/** One entry of a bulk bar menu, indented by `depth` levels (Kotlin `MenuChoice`). */
function MenuChoice({ text, depth, italic = false, onClick }: { text: string; depth: number; italic?: boolean; onClick: () => void }) {
  return (
    <button type="button" role="menuitem" className="menu-item" style={{ paddingLeft: 12 + 16 * depth, paddingRight: 12 }} onClick={onClick}>
      <span className="ellipsis" style={italic ? { fontStyle: "italic" } : undefined}>{text}</span>
    </button>
  );
}
