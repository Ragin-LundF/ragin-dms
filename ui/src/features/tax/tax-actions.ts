import {commands, type TaxAreaDto, type TaxItemChangeDto_Serialize, unwrap} from "../../ipc";
import {reportError} from "../../session/session";
import {dialogs} from "../../shared/components";

/** The changes of the tax screen and the tax dialogs; each reports its own error. */
export const taxActions = {
  open(year: number) {
    unwrap(commands.taxOpenYear(year)).catch(reportError);
  },
  change(year: number, change: TaxItemChangeDto_Serialize) {
    unwrap(commands.taxUpdateItem(year, change)).catch(reportError);
  },
  async rename(year: number, itemId: string, current: string) {
    const label = await dialogs.askText({ title: "Rename item", label: "Name", initialValue: current, confirmLabel: "Rename" });
    if (label?.trim()) taxActions.change(year, { kind: "rename", itemId, label });
  },
  async add(year: number, area: TaxAreaDto, areaLabel: string) {
    const label = await dialogs.askText({ title: `New item in ${areaLabel}`, label: "What is needed", initialValue: "", confirmLabel: "Add" });
    if (label?.trim()) taxActions.change(year, { kind: "add", area, label, correspondent: null });
  },
  /** Assigns documents to a year and area, or with both null removes the assignment; resolves to whether it was done. */
  assign(documentIds: string[], year: number | null, area: TaxAreaDto | null, deductible: string | null, keepDeductible: boolean): Promise<boolean> {
    return unwrap(commands.taxAssign({ documentIds, year, area, deductible, keepDeductible })).catch((error: unknown) => {
      reportError(error);
      return false;
    });
  },
};
