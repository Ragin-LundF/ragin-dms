import {commands, type LibraryExportOptionsDto, type LibraryExportScopeDto_Serialize, unwrap} from "../../../ipc";
import {reportError} from "../../../session/session";
import {chooseFolder} from "../../../shared/platform";
import {createStore} from "../../../shared/store";

/** The export the user is setting up (Kotlin `ExportDraftModel`): which documents, its name and layout. */
export type ExportDraft = {
  scope: LibraryExportScopeDto_Serialize;
  scopeLabel: string;
  name: string;
  options: LibraryExportOptionsDto;
  correspondent: string;
};

const DEFAULT_OPTIONS: LibraryExportOptionsDto = { flatten: false, renameToTitle: false, index: true, zip: false, taxGrouping: false };

export const exportStore = createStore<ExportDraft | null>(null);

/** Opens the export of `scope`, named and labelled `label` (the view title or the saved view's name). */
export function openExport(scope: LibraryExportScopeDto_Serialize, label: string) {
  exportStore.set({ scope, scopeLabel: label, name: label, options: DEFAULT_OPTIONS, correspondent: "" });
}

/** "Send to tax adviser": the tax year as one ZIP with the index, grouped by tax area, missing items first. */
export function openTaxAdviserExport(year: number) {
  exportStore.set({
    scope: { kind: "taxYear", year },
    scopeLabel: `Tax year ${year}`,
    name: `Tax ${year}`,
    options: { ...DEFAULT_OPTIONS, zip: true, index: true, taxGrouping: true },
    correspondent: "",
  });
}

/** The tax years offered as presets: this year and the three before. */
export function taxYears(today = new Date()): number[] {
  const year = today.getFullYear();
  return [year, year - 1, year - 2, year - 3];
}

function change(patch: (draft: ExportDraft) => Partial<ExportDraft>) {
  exportStore.set((draft) => draft && { ...draft, ...patch(draft) });
}

export const exporting = {
  presetTaxYear(year: number) {
    change((draft) => ({ scope: { kind: "taxYear", year }, scopeLabel: `Tax year ${year}`, name: `Tax ${year}`, options: { ...draft.options, taxGrouping: true } }));
  },
  setCorrespondent(correspondent: string) {
    change(() => ({ correspondent }));
  },
  presetCorrespondent() {
    const name = exportStore.get()?.correspondent.trim();
    if (!name) return;
    change(() => ({ scope: { kind: "correspondent", name }, scopeLabel: `Everything from ${name}`, name }));
  },
  setName(name: string) {
    change(() => ({ name }));
  },
  setOptions(options: LibraryExportOptionsDto) {
    change(() => ({ options }));
  },
  cancel() {
    exportStore.set(null);
  },
  /** Asks for the target folder and exports there; only a successful export closes the dialog. */
  async run() {
    const draft = exportStore.get();
    if (!draft || !draft.name.trim()) return;
    const directory = await chooseFolder(`Choose where to export "${draft.name}"`);
    if (directory === null) return;
    try {
      if (await unwrap(commands.libraryExport(draft.scope, draft.scopeLabel, directory, draft.name.trim(), draft.options))) exportStore.set(null);
    } catch (error) {
      reportError(error);
    }
  },
};
