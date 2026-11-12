import { useQuery } from "@tanstack/react-query";
import EMERGENCY from "@material-design-icons/svg/round/emergency.svg?raw";
import { commands, unwrap } from "../../ipc";
import { LIBRARY_QUERY, reportError } from "../../session/session";
import { Button, Menu, MenuItem, useMenuAnchor } from "../../shared/components";
import { GROUPS } from "./groups";

export type AddToEmergencyButtonProps = { documentId?: string; contractId?: string; busyReason?: string | null };

/**
 * "Add to emergency folder" for a document or a contract, with the group chosen from a menu; once added, it says
 * so and offers to take it out again.
 */
export function AddToEmergencyButton({ documentId, contractId, busyReason = null }: AddToEmergencyButtonProps) {
  const menu = useMenuAnchor();
  const membership = useQuery({
    queryKey: [LIBRARY_QUERY, "emergency", "membership", documentId, contractId],
    queryFn: () => unwrap(commands.emergencyMembership(documentId ?? null, contractId ?? null)),
  });
  const member = membership.data;
  if (!member) return null;
  if (member.itemId) {
    const itemId = member.itemId;
    return (
      <div className="row gap-s emergency-member">
        <span className="body-small muted ellipsis">In the emergency folder · {member.group}</span>
        <Button variant="text" small disabled={busyReason !== null} title={busyReason ?? undefined}
          onClick={() => void unwrap(commands.emergencyEdit({ kind: "remove", itemId })).catch(reportError)}>
          Remove
        </Button>
      </div>
    );
  }
  const add = (group: (typeof GROUPS)[number]["group"]) => {
    menu.close();
    const edit = contractId
      ? { kind: "addContract" as const, group, contractId }
      : { kind: "addDocument" as const, group, documentId: documentId ?? "" };
    unwrap(commands.emergencyEdit(edit)).catch(reportError);
  };
  return (
    <>
      <Button variant="text" small icon={EMERGENCY} disabled={busyReason !== null} title={busyReason ?? undefined} onClick={menu.open}>
        Add to emergency folder
      </Button>
      {menu.anchor && (
        <Menu anchor={menu.anchor} onClose={menu.close}>
          {GROUPS.map(({ group, label }) => <MenuItem key={group} label={label} onClick={() => add(group)} />)}
        </Menu>
      )}
    </>
  );
}
