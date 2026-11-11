import {useEffect} from "react";
import {events} from "../../ipc";
import {ChatDialog} from "./assistant/chat-dialog";
import {SummaryDialog} from "./assistant/summary-dialog";
import {LinkPickerDialog} from "./details/link-picker-dialog";
import {ReviewDialog} from "./review/review-dialog";
import {openReview} from "./state";
import "./document.css";

export { QuickLookPanel } from "./quicklook/quick-look-panel";
export { closeQuickLook, openQuickLook, openReview, quickLook, quickLookEditing, stepQuickLook } from "./state";

/** Every dialog the document feature opens (review, chat, summary, link picker); mounted once by the shell. */
export function DocumentDialogs() {
  useEffect(() => {
    const listening = events.openReviewEvent.listen(({ payload }) => openReview(payload.documentId));
    return () => void listening.then((stop) => stop());
  }, []);
  return (
    <>
      <ReviewDialog />
      <SummaryDialog />
      <ChatDialog />
      <LinkPickerDialog />
    </>
  );
}

