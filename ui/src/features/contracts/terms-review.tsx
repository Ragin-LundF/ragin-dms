import { useEffect, useState } from "react";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import { commands, describeError, unwrap, type ContractTermsReviewDto_Serialize } from "../../ipc";
import { reportError } from "../../session/session";
import { Button, Dialog, Icon, Spinner } from "../../shared/components";

/**
 * What the AI read from a contract's documents. Nothing is saved until the user accepts a price point (each on its
 * own) or the terms.
 */
export function TermsReview({ contractId, name, onClose }: { contractId: string; name: string; onClose: () => void }) {
  const [review, setReview] = useState<ContractTermsReviewDto_Serialize | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [accepted, setAccepted] = useState<Set<string>>(new Set());

  useEffect(() => {
    let current = true;
    unwrap(commands.contractsExtractTerms(contractId))
      .then((read) => current && setReview(read), (error: unknown) => current && setProblem(describeError(error)));
    return () => {
      current = false;
    };
  }, [contractId]);

  const terms = review?.terms ?? null;
  const accept = (key: string, run: () => Promise<unknown>) =>
    run().then(() => setAccepted((done) => new Set(done).add(key)), reportError);

  return (
    <Dialog title={`Prices and terms of ${name}`} onDismiss={onClose} width={560} buttons={<Button variant="text" onClick={onClose}>Close</Button>}>
      {!review && !problem && (
        <div className="row gap-m"><Spinner small /><span className="body-medium muted">The AI reads the contract's documents…</span></div>
      )}
      {problem && <p className="body-medium error-text" role="alert">{problem}</p>}
      {review && (
        <div className="column gap-m">
          <span className="row gap-xs label-medium muted"><Icon svg={AUTO_AWESOME} />{review.origin}</span>
          {review.prices.length === 0 && !terms && <p className="body-medium muted">The AI found no prices or terms that are not recorded yet.</p>}
          {review.prices.map((price) => {
            const key = `${price.point.validFrom}:${price.point.price}`;
            return (
              <div key={key} className="row gap-m contract-review-row">
                <div className="column grow">
                  <span className="body-medium">{price.label}{price.rose && <span className="contract-pill warning">Price ↑</span>}</span>
                  {price.source && <span className="body-small muted ellipsis">From {price.source}</span>}
                </div>
                <Button small variant="tonal" disabled={accepted.has(key)} onClick={() => accept(key, () => unwrap(commands.contractsAcceptPrice(contractId, price.point)))}>
                  {accepted.has(key) ? "Accepted" : "Accept"}
                </Button>
              </div>
            );
          })}
          {terms && (
            <div className="row gap-m contract-review-row">
              <span className="body-medium grow">{review.termsLabel}</span>
              <Button small variant="tonal" disabled={accepted.has("terms")} onClick={() => accept("terms", () => unwrap(commands.contractsAcceptTerms(contractId, terms)))}>
                {accepted.has("terms") ? "Accepted" : "Accept terms"}
              </Button>
            </div>
          )}
        </div>
      )}
    </Dialog>
  );
}
