import type {HealthScheduleDto, HealthSnapshotDiffDto, HealthStateDto_Serialize} from "../../ipc";
import {FIXTURE_DOCUMENTS, FIXTURE_FOLDERS} from "../fixture";
import type {Handler} from "./index";
import {mockOperation} from "./shell";

let schedule: HealthScheduleDto = "weekly";
const damaged = FIXTURE_DOCUMENTS[2];

/** Mocked commands of features::health: one damaged copy in the cloud folder and three snapshots. */
export const healthHandlers: Record<string, Handler> = {
  health_state: (): HealthStateDto_Serialize => ({
    headline: "1 problem found",
    tone: "error",
    filesChecked: FIXTURE_DOCUMENTS.length * 2 - 3,
    done: 1,
    lastCheckAt: "2026-11-11T03:12:00Z",
    unfinished: false,
    schedule,
    folders: [
      { name: "Documents", location: "/Synthetic/Documents", primary: true, summary: `${FIXTURE_DOCUMENTS.length} files checked`, pill: "Intact", tone: "good" },
      { name: "Cloud", location: "/Synthetic/Cloud/Documents", primary: false, summary: `${FIXTURE_DOCUMENTS.length - 3} files checked · 3 not downloaded, skipped`, pill: "1 damaged", tone: "error" },
    ],
    findings: damaged
      ? [{
          key: `damaged:${damaged.id}:${FIXTURE_FOLDERS.sync}`,
          title: `Damaged copy in Cloud: ${damaged.path}`,
          detail: "Documents has an intact copy",
          buttons: [
            {
              label: "Repair from Documents…", icon: "healing", style: "tonal", disabledReason: null, lane: "library",
              action: { kind: "repairCopy", taskKey: `damaged:${damaged.id}:${FIXTURE_FOLDERS.sync}` },
              confirm: { title: "Repair from Documents", message: "The damaged copy goes to the trash of Cloud and the intact copy of Documents takes its place.", affected: [`Cloud: ${damaged.path}`], confirmLabel: "Repair" },
            },
            { label: "Dismiss", icon: null, style: "text", disabledReason: null, lane: "library", action: { kind: "dismiss", taskKey: `damaged:${damaged.id}:${FIXTURE_FOLDERS.sync}` }, confirm: null },
          ],
          opens: [],
          preview: null,
          copies: [],
        }]
      : [],
    snapshots: [
      { id: "20261111T091400Z-r42", takenAt: "2026-11-11T09:14:00Z", label: "Revision 42", current: true },
      { id: "20261110T214000Z-r41", takenAt: "2026-11-10T21:40:00Z", label: "Revision 41", current: false },
      { id: "20261103T080000Z-r35", takenAt: "2026-11-03T08:00:00Z", label: "Revision 35", current: false },
    ],
  }),
  health_check_now: () => mockOperation("Checking the archive", "library", 6, "Archive checked: every copy is intact", true),
  health_set_schedule: (args) => {
    schedule = args.schedule as HealthScheduleDto;
    return null;
  },
  health_compare_snapshot: (): HealthSnapshotDiffDto => ({ lines: ["1 document leaves the catalog", "2 tags come back"] }),
  health_restore_snapshot: () => mockOperation("Restoring a catalog snapshot", "library", 2, "Restored the catalog snapshot"),
};
