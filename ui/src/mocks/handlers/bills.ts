import type {
    BillDocumentDto,
    BillPaymentCodeDto,
    BillRowDto_Serialize,
    BillsFilterDto,
    BillsViewDto_Serialize,
    ExpectedDocumentDto,
} from "../../ipc";
import type {Handler} from "./index";

/** Invented bills; `paidOn` set means paid. */
const BILLS: BillRowDto_Serialize[] = [
  { documentId: "bill-water", title: "Water bill Q3", payee: "Sample Waterworks", due: "2026-11-14", dueInDays: 3, status: "open", paidOn: null, amount: "86.40 EUR", source: "ai", qrRefusal: null },
  { documentId: "bill-dentist", title: "Dentist invoice", payee: "Example Dental Practice", due: "2026-11-08", dueInDays: -3, status: "overdue", paidOn: null, amount: "212.75 EUR", source: "ai", qrRefusal: null },
  { documentId: "bill-insurance", title: "Car insurance 2027", payee: "Sample Insurance", due: "2026-12-01", dueInDays: 20, status: "open", paidOn: null, amount: "487.20 EUR", source: "eInvoice", qrRefusal: null },
  { documentId: "bill-shop", title: "Online shop order", payee: "Example Shop", due: "2026-11-20", dueInDays: 9, status: "open", paidOn: null, amount: "49.00 CHF", source: "typed", qrRefusal: "Payment codes only work for euro amounts, not CHF" },
  { documentId: "bill-plumber", title: "Plumber, kitchen tap", payee: "Sample Plumbing", due: "2026-11-05", dueInDays: -6, status: "paid", paidOn: "2026-11-04", amount: "164.50 EUR", source: "eInvoice", qrRefusal: null },
];

const OLDER: BillRowDto_Serialize[] = [
  { documentId: "bill-old", title: "Electricity advance", payee: "Sample Energy", due: "2026-06-01", dueInDays: -163, status: "overdue", paidOn: null, amount: "94.00 EUR", source: "ai", qrRefusal: null },
];

const EXPECTED: ExpectedDocumentDto[] = [
  { taskKey: "expected:sample building society:-:2026-01-15", title: "Annual statement", correspondent: "Sample Building Society", rhythm: "Arrived every January since 2021", expectedOn: "2026-01-15", documentIds: [] },
  { taskKey: "expected:employer ltd:salary:2026-10-03", title: "Salary", correspondent: "Employer Ltd", rhythm: "Arrived every month since 2024", expectedOn: "2026-10-03", documentIds: [] },
];

let tracking = false;

/** The amounts of `rows` summed per currency, like "835.35 EUR + 49.00 CHF". */
function total(rows: BillRowDto_Serialize[]) {
  const sums = new Map<string, number>();
  for (const row of rows) {
    const [value = "0", currency = ""] = row.amount.split(" ");
    sums.set(currency, (sums.get(currency) ?? 0) + Number(value));
  }
  const amount = [...sums].map(([currency, sum]) => `${sum.toFixed(2)} ${currency}`).join(" + ");
  return { amount, count: rows.length };
}

function setPaid(documentId: string, paidOn: string | null) {
  const bill = BILLS.find((row) => row.documentId === documentId);
  if (!bill) return;
  bill.paidOn = paidOn;
  bill.status = paidOn ? "paid" : bill.dueInDays < 0 ? "overdue" : "open";
}

/** Mocked commands of features::bills. Totals are summed per currency like the Rust presenter does. */
export const billHandlers: Record<string, Handler> = {
  bills_view: (args): BillsViewDto_Serialize => {
    const filter = args.filter as BillsFilterDto;
    const unpaid = BILLS.filter((row) => row.status !== "paid");
    const rows = BILLS.filter((row) => filter === "all" || (filter === "paid" ? row.status === "paid" : row.status !== "paid")).sort((a, b) => a.due.localeCompare(b.due));
    return {
      open: total(unpaid),
      overdue: total(unpaid.filter((row) => row.status === "overdue")),
      paidThisMonth: total(BILLS.filter((row) => row.status === "paid")),
      dueThisMonth: total(unpaid.filter((row) => row.due <= "2026-11-30")),
      month: "November",
      rows,
      older: OLDER,
      trackingStarted: tracking,
    };
  },
  bills_start_tracking: () => {
    tracking = true;
    return null;
  },
  bills_mark_paid: (args) => {
    setPaid(args.documentId as string, "2026-11-11");
    return null;
  },
  bills_mark_unpaid: (args) => {
    setPaid(args.documentId as string, null);
    return null;
  },
  bills_settle_older: () => {
    OLDER.splice(0);
    return null;
  },
  bills_payment_code: (args): BillPaymentCodeDto => {
    const bill = BILLS.find((row) => row.documentId === args.documentId);
    if (!bill) throw { code: "invalid_input", message: "The document is no longer in the catalog" };
    return {
      documentId: bill.documentId,
      title: bill.title,
      svg: bill.qrRefusal ? null : '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 21 21"><rect width="21" height="21" fill="#fff"/><rect x="1" y="1" width="7" height="7"/><rect x="13" y="1" width="7" height="7"/><rect x="1" y="13" width="7" height="7"/></svg>',
      refusal: bill.qrRefusal,
      payee: bill.payee,
      iban: "DE89 3704 0044 0532 0130 00",
      amount: bill.amount,
      reference: "KD 4711",
      due: bill.due,
      source: bill.source,
      paid: bill.status === "paid",
    };
  },
  bills_document: (args): BillDocumentDto => {
    const bill = BILLS.find((row) => row.documentId === args.documentId);
    return bill
      ? { isBill: true, status: bill.status, paidOn: bill.paidOn, canReadEInvoice: true }
      : { isBill: false, status: null, paidOn: null, canReadEInvoice: true };
  },
  bills_read_e_invoice: () => {
    throw { code: "invalid_input", message: "The mock documents hold no e-invoice" };
  },
  bills_expected_missing: (): ExpectedDocumentDto[] => EXPECTED,
  bills_expected_requested: (args) => {
    EXPECTED.splice(EXPECTED.findIndex((item) => item.taskKey === args.taskKey), 1);
    return null;
  },
  bills_expected_stop: (args) => {
    EXPECTED.splice(EXPECTED.findIndex((item) => item.taskKey === args.taskKey), 1);
    return null;
  },
};
