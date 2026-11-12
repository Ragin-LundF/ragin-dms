import {useEffect, useState} from "react";
import {useQuery} from "@tanstack/react-query";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import OPEN from "@material-design-icons/svg/round/open_in_new.svg?raw";
import {type CasesReviewDto, commands, unwrap} from "../../ipc";
import {LIBRARY_QUERY, useSession} from "../../session/session";
import {Icon, IconButton} from "../../shared/components";
import {openDocument} from "../library/navigation";
import {reviewCaseChoice, type ReviewCaseState, reviewChoiceOf} from "./state";

/** The case part of a review: follow-ups for what the letter asks, or "Add to case" for the sender's open case. */
export function ReviewCases({ documentId }: { documentId: string }) {
  const review = useQuery({
    queryKey: [LIBRARY_QUERY, "cases", "review", documentId],
    queryFn: () => unwrap(commands.casesReview(documentId)),
  });
  useEffect(() => () => reviewCaseChoice.set(null), []);
  return review.data ? <ReviewCaseChoice review={review.data} /> : null;
}

function ReviewCaseChoice({ review }: { review: CasesReviewDto }) {
  const { aiEnabled } = useSession();
  const [state, setState] = useState<ReviewCaseState>(() => ({
    checked: review.requested.map(() => true),
    target: review.openCase ? "existing" : "new",
    join: review.preChecked,
  }));
  useEffect(() => reviewCaseChoice.set(reviewChoiceOf(review, aiEnabled, state)), [review, aiEnabled, state]);
  const open = review.openCase;
  if (aiEnabled && review.requested.length > 0) {
    return (
      <div className="review-cases">
        <span className="label-large row gap-s"><Icon svg={AUTO_AWESOME} /> Create a follow-up</span>
        {review.requested.map((action, index) => (
          <label key={`${action.text}-${action.date}`} className="row gap-s body-medium">
            <input type="checkbox" className="checkbox" checked={state.checked[index] ?? true}
              onChange={(event) => setState({ ...state, checked: state.checked.map((value, at) => (at === index ? event.target.checked : value)) })} />
            <span className="grow">{action.text}</span>
            <span className="body-small muted nowrap">by {action.date}</span>
          </label>
        ))}
        <div className="row gap-m body-medium">
          <label className="row gap-s"><input type="radio" checked={state.target === "new"} onChange={() => setState({ ...state, target: "new" })} /> New case “{review.newTitle}”</label>
          {open && <label className="row gap-s"><input type="radio" checked={state.target === "existing"} onChange={() => setState({ ...state, target: "existing" })} /> Add to “{open.title}”</label>}
        </div>
        {review.matches.length > 0 && (
          <div className="column">
            <span className="body-small muted">Matching documents in your library</span>
            {review.matches.map((match) => (
              <span key={match.documentId} className="row gap-s">
                <span className="body-medium grow ellipsis">{match.name}</span>
                <IconButton icon={OPEN} label={`Open ${match.name}`} small tooltip onClick={() => openDocument(match.documentId)} />
              </span>
            ))}
          </div>
        )}
      </div>
    );
  }
  if (!open) return null;
  return (
    <label className="review-cases row gap-s body-medium">
      <input type="checkbox" className="checkbox" checked={state.join} onChange={(event) => setState({ ...state, join: event.target.checked })} />
      <span className="grow">Add to case “{open.title}”{open.status === "waiting" ? " (waiting for this answer)" : ""}</span>
    </label>
  );
}
