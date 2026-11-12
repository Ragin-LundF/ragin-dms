import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import {commands, type EmergencyGroupListDto_Serialize, unwrap} from "../../ipc";
import {reportError} from "../../session/session";
import {IconButton, Switch} from "../../shared/components";
import {openQuickLook} from "../document";

/** The items by group: a switch for "goes into the overview and the ZIP", a "changed" pill, and remove. */
export function EmergencyList({ groups, busyReason }: { groups: EmergencyGroupListDto_Serialize[]; busyReason: string | null }) {
  const edit = (promise: ReturnType<typeof commands.emergencyEdit>) => void unwrap(promise).catch(reportError);
  return (
    <div className="column emergency-list">
      {groups.filter((group) => group.items.length > 0).map((group) => (
        <section key={group.group} className="column gap-s">
          <h2 className="label-medium muted emergency-group">{group.label}</h2>
          {group.items.map((item) => (
            <div key={item.id} className="row gap-m emergency-item">
              <Switch label={`Include ${item.text}`} checked={item.included} disabled={busyReason !== null}
                onChange={(included) => edit(commands.emergencyEdit({ kind: "setIncluded", itemId: item.id, included }))} />
              {item.documentId ? (
                <button type="button" className="link-button body-medium grow ellipsis" onClick={() => openQuickLook(item.documentId ?? "", [item.documentId ?? ""])}>
                  {item.text}
                </button>
              ) : (
                <span className="body-medium grow ellipsis">{item.text}</span>
              )}
              {item.changed && <span className="emergency-changed label-small">changed since last refresh</span>}
              <IconButton icon={CLOSE} label="Remove from the emergency folder" small tooltip disabled={busyReason !== null}
                onClick={() => edit(commands.emergencyEdit({ kind: "remove", itemId: item.id }))} />
            </div>
          ))}
        </section>
      ))}
    </div>
  );
}
