import AUTORENEW from "@material-design-icons/svg/round/autorenew.svg?raw";
import {Button, Chip, Menu, MenuDivider, MenuItem, useMenuAnchor} from "../../../../shared/components";
import {attachToContract, createContract, openContract, useContractsOfDocument} from "../../../contracts";
import type {OrganizeRowProps} from "./organize-block";

/** The contracts a document belongs to as chips, and "Add to contract…" with the running contracts or a new one. */
export function ContractRow({ documentId, busyReason }: OrganizeRowProps) {
  const { data } = useContractsOfDocument(documentId);
  const menu = useMenuAnchor();
  if (!data) return null;
  return (
    <div className="contract-row">
      {data.contracts.map((contract) => (
        <Chip key={contract.id} icon={AUTORENEW} label={`Contract: ${contract.name}`} onClick={() => openContract(contract.id)} title="Show the contract" />
      ))}
      <Button small variant="text" onClick={menu.open} disabled={busyReason !== null} title={busyReason ?? undefined}>
        Add to contract…
      </Button>
      {menu.anchor && (
        <Menu anchor={menu.anchor} onClose={menu.close}>
          {data.others.map((contract) => (
            <MenuItem key={contract.id} label={contract.name} onClick={() => { menu.close(); void attachToContract(contract.id, [documentId]); }} />
          ))}
          {data.others.length > 0 && <MenuDivider />}
          <MenuItem label="New contract…" onClick={() => { menu.close(); createContract(data.correspondent ?? "", [documentId]); }} />
        </Menu>
      )}
    </div>
  );
}
