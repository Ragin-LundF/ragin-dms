import type {
    LetterDraftDto_Serialize,
    LetterDraftInputDto_Serialize,
    LetterKindDto,
    LetterSavedDto,
    LetterSaveInputDto
} from "../../ipc";
import {FIXTURE_DOCUMENTS} from "../fixture";
import type {Handler} from "./index";

const SUBJECTS: Record<LetterKindDto, string> = {
  cancellation: "Cancellation",
  objection: "Objection",
  reminder: "Reminder",
  request: "Request",
  reply: "Reply",
};

function parentOf(path: string): string {
  return path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "";
}

/** Mocked commands of features::letters: the template draft, and saving as a PDF named after the subject. */
export const letterHandlers: Record<string, Handler> = {
  letter_draft: (args): LetterDraftDto_Serialize => {
    const input = args.input as LetterDraftInputDto_Serialize;
    const sources = FIXTURE_DOCUMENTS.filter((document) => input.documentIds.includes(document.id));
    if (sources.length < input.documentIds.length) throw { code: "invalid_input", message: "The document or file no longer exists" };
    const newest = sources[sources.length - 1];
    const fact = input.subjectFacts[0];
    return {
      subject: fact ? `${SUBJECTS[input.kind]}: ${fact}` : SUBJECTS[input.kind],
      body: ["Dear Sir or Madam,", "I hereby cancel the contract at the next possible date. Please confirm the cancellation and the end date in writing.", ...input.subjectFacts, "Kind regards\n\nAlex Sample"].join("\n\n"),
      sender: { name: "Alex Sample", addressLines: ["1 Harbour Road", "12345 Sampletown"] },
      recipient: { name: input.recipient ?? newest?.correspondent ?? "", addressLines: [] },
      language: "en",
      draftedByAi: false,
      origin: "Template",
      folder: newest ? parentOf(newest.path) : "",
      folders: ["", ...new Set(FIXTURE_DOCUMENTS.map((document) => parentOf(document.path)).filter((folder) => folder !== ""))].sort(),
    };
  },
  letter_save: (args): LetterSavedDto => {
    const input = args.input as LetterSaveInputDto;
    const name = `2026-11-11 ${input.subject.replace(/[<>:"/\\|?*]/g, "-")}.pdf`;
    return { documentId: "mock-letter", path: input.folder ? `${input.folder}/${name}` : name };
  },
};
