import type { BillSourceDto } from "../../ipc";

const SOURCES: Record<BillSourceDto, { label: string; title: string; tone: string }> = {
  eInvoice: { label: "E-invoice", title: "Exact data from the e-invoice XML inside the PDF", tone: "blue" },
  ai: { label: "AI, approved", title: "Read by the AI and approved by you", tone: "violet" },
  rule: { label: "Rule", title: "Set by one of your matching rules", tone: "grey" },
  typed: { label: "Typed", title: "Entered by hand", tone: "grey" },
};

/** Where a bill's values came from. */
export function SourcePill({ source }: { source: BillSourceDto }) {
  const style = SOURCES[source];
  return <span className={`bill-pill tone-${style.tone}`} title={style.title}>{style.label}</span>;
}
