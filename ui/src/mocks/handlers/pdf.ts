import type {
    MergeCandidateDto,
    PageItemDto,
    PdfToolsDto,
    RebuildChoiceDto,
    RebuildCompareDto_Serialize
} from "../../ipc";
import {fileName, FIXTURE_DOCUMENTS, FIXTURE_PAGE_COUNTS, type FixtureDocument} from "../fixture";
import type {Handler} from "./index";
import {mockRun} from "./restructure";

let comparison: RebuildCompareDto_Serialize | null = null;

function documentOf(id: unknown): FixtureDocument {
  const document = FIXTURE_DOCUMENTS.find((candidate) => candidate.id === id);
  if (!document) throw { code: "invalid_input", message: "The document or file no longer exists" };
  return document;
}

function candidate(document: FixtureDocument): MergeCandidateDto {
  return { documentId: document.id, name: document.title ?? fileName(document.path), path: document.path };
}

function pagesOf(documentId: string): number {
  return FIXTURE_PAGE_COUNTS[documentId] ?? 3;
}

function rebuilt(document: FixtureDocument): RebuildCompareDto_Serialize {
  const pages = pagesOf(document.id);
  return {
    documentId: document.id,
    rebuildId: `rebuild-${document.id}`,
    name: fileName(document.path),
    path: document.path,
    originalSize: document.size,
    rebuiltSize: document.size + 401_000,
    pages: Array.from({ length: pages }, (_, page) => ({
      linesAsText: 30 - page * 3,
      linesTotal: 32,
      kept: page === 0 ? "Kept as images: 1 × covered by a stamp, signature or handwriting; 1 × the AI was not sure of every character" : null,
      regions: [
        { kind: "image" as const, left: 80, top: 60, right: 300, bottom: 140 },
        ...Array.from({ length: 12 }, (_, line) => ({ kind: line === 5 ? ("keptLine" as const) : ("text" as const), left: 80, top: 200 + line * 40, right: 900 - (line % 3) * 120, bottom: 225 + line * 40 })),
      ],
    })),
  };
}

const CHOICES: Record<RebuildChoiceDto, { label: (name: string) => string; done: (name: string) => string }> = {
  keepOriginal: { label: (name) => `Keeping the original of ${name}`, done: (name) => `Kept the original of ${name}` },
  replaceWithRebuild: { label: (name) => `Replacing ${name} with its rebuild`, done: (name) => `${name} is a real PDF now; the scan is in the trash` },
  keepBoth: { label: (name) => `Adding the rebuild of ${name}`, done: (name) => `Added the rebuild of ${name} next to it` },
};

/** Mocked commands of features::pdf. */
export const pdfHandlers: Record<string, Handler> = {
  pdf_tools: (args): PdfToolsDto => {
    const document = documentOf(args.documentId);
    return { pdf: document.kind === "pdf", name: fileName(document.path), ocrAvailable: true, rebuildable: /scan/i.test(document.path) };
  },
  make_searchable: (args) => {
    const name = fileName(documentOf(args.documentId).path);
    mockRun(`Making ${name} searchable`, "library", 3, false, () => ({ text: `${name} is searchable now`, isError: false }));
    return null;
  },
  make_all_searchable: () => {
    mockRun("Making scans searchable", "library", 4, true, () => ({ text: "Made 2 scans searchable; scans without a transcription were left out", isError: false }));
    return null;
  },
  merge_start: (args) => {
    const document = documentOf(args.documentId);
    const name = fileName(document.path);
    return { first: candidate(document), name: `${name.includes(".") ? name.slice(0, name.lastIndexOf(".")) : name} (merged)` };
  },
  merge_matches: (args) => {
    const wanted = (args.query as string).trim().toLowerCase();
    const chosen = args.chosen as string[];
    return FIXTURE_DOCUMENTS.filter((document) => document.kind === "pdf" && !chosen.includes(document.id))
      .filter((document) => wanted === "" || `${document.path} ${document.title ?? ""}`.toLowerCase().includes(wanted))
      .sort((first, second) => second.addedAt.localeCompare(first.addedAt))
      .slice(0, 30)
      .map(candidate);
  },
  merge_documents: (args) => {
    const ids = args.documentIds as string[];
    const name = (args.name as string).trim();
    mockRun(`Merging ${ids.length} PDFs`, "library", 3, false, () => ({ text: `Merged into ${name}`, isError: false }));
    return null;
  },
  pdf_page_count: (args) => pagesOf(args.documentId as string),
  edit_pdf_pages: (args) => {
    const name = fileName(documentOf(args.documentId).path);
    const kept = (args.pages as PageItemDto[]).filter((page) => !page.deleted);
    const parts = kept.slice(0, -1).filter((page) => page.splitAfter).length + 1;
    mockRun(`Editing the pages of ${name}`, "library", 3, false, () => ({ text: parts > 1 ? `Split ${name} into ${parts} documents` : `Saved the pages of ${name}`, isError: false }));
    return null;
  },
  start_rebuild: (args) => {
    if (comparison) throw { code: "busy", message: "Choose what to keep in the open comparison first" };
    const document = documentOf(args.documentId);
    const name = fileName(document.path);
    mockRun(`Converting ${name} to a real PDF`, "ai", pagesOf(document.id) * 2, true, () => {
      comparison = rebuilt(document);
      return { text: `${name} is rebuilt: compare and choose`, isError: false };
    });
    return null;
  },
  rebuild_compare: () => comparison,
  choose_rebuild: (args) => {
    const open = comparison;
    if (!open) return null;
    const choice = CHOICES[args.choice as RebuildChoiceDto];
    mockRun(choice.label(open.name), "library", 2, false, () => {
      comparison = null;
      return { text: choice.done(open.name), isError: false };
    });
    return null;
  },
};
