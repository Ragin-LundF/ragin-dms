import ADD from "@material-design-icons/svg/round/add.svg?raw";
import CALENDAR from "@material-design-icons/svg/round/calendar_today.svg?raw";
import CATEGORY from "@material-design-icons/svg/round/category.svg?raw";
import CHECK from "@material-design-icons/svg/round/check.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import DESCRIPTION from "@material-design-icons/svg/round/description.svg?raw";
import DROPDOWN from "@material-design-icons/svg/round/arrow_drop_down.svg?raw";
import EDIT from "@material-design-icons/svg/round/edit.svg?raw";
import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import MORE from "@material-design-icons/svg/round/more_horiz.svg?raw";
import OFFER from "@material-design-icons/svg/round/local_offer.svg?raw";
import PERSON from "@material-design-icons/svg/round/person.svg?raw";
import {useState} from "react";
import type {LibraryFacetDto, LibraryFacetOptionDto_Serialize} from "../../../ipc";
import {
    type ActionButton,
    Icon,
    IconButton,
    Menu,
    MenuDivider,
    MenuItem,
    ReasonTooltip,
    useMenuAnchor
} from "../../../shared/components";
import {taxonomy} from "../state/actions";
import type {LibraryState} from "../state/queries";
import {type FacetFilter, sameFilter} from "../state/view-state";

type Option = LibraryFacetOptionDto_Serialize;

const FACETS: Record<LibraryFacetDto, { label: string; icon: string; note: string | null }> = {
  category: { label: "Category", icon: CATEGORY, note: "Subcategories stay inside the list; a category includes them." },
  tag: { label: "Tag", icon: OFFER, note: "Sorted by how often a tag is used." },
  year: { label: "Year", icon: CALENDAR, note: "The year of the document date, or of when a document without one was added." },
  correspondent: { label: "Correspondent", icon: PERSON, note: null },
  type: { label: "Type", icon: DESCRIPTION, note: null },
  folder: { label: "Folder", icon: FOLDER, note: "Shows what is inside a folder. To move, rename or create folders, open Folders in the sidebar." },
};

export const FACET_ORDER: LibraryFacetDto[] = ["category", "tag", "year", "correspondent", "type", "folder"];

/**
 * One facet as a chip (Kotlin `FacetChip`); while its filter is active it is tinted and shows its token's value. A
 * click opens the popover; choosing a value applies it, choosing the active one removes it. Categories and tags have
 * a ⋯ menu each, the category popover "New category…" at its end.
 */
export function FacetChip({ facet, state, filters, busyReason, onPick, onRemove }: {
  facet: LibraryFacetDto;
  state: LibraryState | undefined;
  filters: FacetFilter[];
  busyReason: string | null;
  onPick: (filter: FacetFilter) => void;
  onRemove: (filter: FacetFilter) => void;
}) {
  const popover = useMenuAnchor();
  const [optionMenu, setOptionMenu] = useState<{ anchor: DOMRect; actions: ActionButton[]; label: string } | null>(null);
  const style = FACETS[facet];
  const active = filters.find((filter) => filter.kind === facet) ?? null;
  const value = active && state?.tokens.find((token) => token.filter && sameFilter(token.filter, active))?.value;
  const options = state?.facets.find((entry) => entry.facet === facet)?.options ?? [];
  const pick = (filter: FacetFilter) => {
    popover.close();
    if (active && sameFilter(active, filter)) onRemove(filter);
    else onPick(filter);
  };
  const openOptionMenu = (anchor: DOMRect, option: Option) => {
    const actions = actionsOf(option, state);
    popover.close();
    setOptionMenu({ anchor, actions, label: option.label });
  };
  return (
    <>
      <button type="button" className={active ? "chip lib-chip selected" : "chip lib-chip"} aria-pressed={active !== null} onClick={popover.open}>
        <Icon svg={style.icon} />
        <span className="lib-chip-label">{value ? `${style.label}: ${value}` : style.label}</span>
        <Icon svg={DROPDOWN} />
      </button>
      {popover.anchor && (
        <Menu anchor={popover.anchor} onClose={popover.close}>
          <FacetPopover
            facet={facet}
            options={options}
            active={active}
            busyReason={busyReason}
            onPick={pick}
            onOptionMenu={openOptionMenu}
            onNewCategory={() => {
              popover.close();
              void taxonomy.newCategory(null);
            }}
          />
        </Menu>
      )}
      {optionMenu && (
        <Menu anchor={optionMenu.anchor} onClose={() => setOptionMenu(null)}>
          {optionMenu.actions.map((action) => (
            <MenuItem
              key={action.label}
              label={action.label}
              icon={action.icon}
              disabledReason={action.disabledReason ?? busyReason}
              onClick={() => {
                setOptionMenu(null);
                action.onClick();
              }}
            />
          ))}
        </Menu>
      )}
    </>
  );
}

function FacetPopover({ facet, options, active, busyReason, onPick, onOptionMenu, onNewCategory }: {
  facet: LibraryFacetDto;
  options: Option[];
  active: FacetFilter | null;
  busyReason: string | null;
  onPick: (filter: FacetFilter) => void;
  onOptionMenu: (anchor: DOMRect, option: Option) => void;
  onNewCategory: () => void;
}) {
  const [narrowed, setNarrowed] = useState("");
  const style = FACETS[facet];
  const wanted = narrowed.trim().toLowerCase();
  const shown = options.filter((option) => option.label.toLowerCase().includes(wanted));
  const placeholder = `Filter ${style.label.toLowerCase()}…`;
  return (
    <div className="lib-popover">
      <div className="lib-narrow">
        <input className="body-medium" value={narrowed} placeholder={placeholder} aria-label={placeholder} autoFocus onChange={(event) => setNarrowed(event.target.value)} />
      </div>
      {shown.length === 0 && <span className="lib-popover-note body-small">Nothing to choose</span>}
      {shown.map((option) => {
        const checked = active !== null && sameFilter(option.filter, active);
        const hasMenu = option.filter.kind === "tag" || (option.filter.kind === "category" && option.filter.categoryId !== null);
        return (
          <div
            key={JSON.stringify(option.filter)}
            tabIndex={-1}
            role="menuitem"
            className={["lib-option", checked ? "checked" : "", option.missing ? "missing" : ""].filter(Boolean).join(" ")}
            style={{ paddingLeft: 12 + 16 * option.depth }}
            onClick={() => onPick(option.filter)}
          >
            {checked && <Icon svg={CHECK} label="Active" />}
            <span className="lib-option-label">{option.label}</span>
            <span className="lib-option-trailing">
              <span className="label-small">{option.count}</span>
              {hasMenu && (
                <span onClick={(event) => event.stopPropagation()}>
                  <ReasonTooltip reason={busyReason}>
                    <IconButton
                      icon={MORE}
                      small
                      label={`Actions for ${option.label}`}
                      disabled={busyReason !== null}
                      onClick={(event) => onOptionMenu(event.currentTarget.getBoundingClientRect(), option)}
                    />
                  </ReasonTooltip>
                </span>
              )}
            </span>
          </div>
        );
      })}
      {facet === "category" && <MenuItem label="New category…" icon={ADD} disabledReason={busyReason} onClick={onNewCategory} />}
      {style.note && (
        <>
          <MenuDivider />
          <span className="lib-popover-note body-small">{style.note}</span>
        </>
      )}
    </div>
  );
}

/** The ⋯ actions of a category or tag value (Kotlin `TaxonomyMenu`). */
function actionsOf(option: Option, state: LibraryState | undefined): ActionButton[] {
  const filter = option.filter;
  if (filter.kind === "tag") {
    return [
      { label: "Rename", icon: EDIT, onClick: () => void taxonomy.renameTag(filter.tag) },
      { label: "Remove", icon: DELETE, onClick: () => void taxonomy.removeTag(filter.tag) },
    ];
  }
  if (filter.kind !== "category" || filter.categoryId === null) return [];
  const id = filter.categoryId;
  const entry = state?.categories.find((category) => category.id === id);
  if (!entry) return [];
  return [
    { label: "Add subcategory", icon: ADD, onClick: () => void taxonomy.newCategory(id) },
    { label: "Rename", icon: EDIT, onClick: () => void taxonomy.renameCategory(id, entry.name) },
    {
      label: "Delete",
      icon: DELETE,
      disabledReason: entry.documentCount > 0 ? "Documents still use this category" : null,
      onClick: () => void taxonomy.removeCategory(id, entry.name),
    },
  ];
}
