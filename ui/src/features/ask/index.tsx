import {useEffect, useState} from "react";
import {useQuery, useQueryClient} from "@tanstack/react-query";
import ARROW_DROP_DOWN from "@material-design-icons/svg/round/arrow_drop_down.svg?raw";
import DESCRIPTION from "@material-design-icons/svg/round/description.svg?raw";
import QUESTION_ANSWER from "@material-design-icons/svg/round/question_answer.svg?raw";
import SEND from "@material-design-icons/svg/round/send.svg?raw";
import SYNC from "@material-design-icons/svg/round/sync.svg?raw";
import {type AskEntryDto_Serialize, commands, unwrap} from "../../ipc";
import {busyIn, LIBRARY_QUERY, reportError, useSession} from "../../session/session";
import {
    ActionButtons,
    AppCard,
    Chip,
    EmptyState,
    Icon,
    IconButton,
    Menu,
    MenuItem,
    ScreenHeader,
    TextField,
    useMenuAnchor,
    waitFor
} from "../../shared/components";
import {createStore, useStore} from "../../shared/store";
import {takeRequest, useNavigation} from "../../shell/screens";
import {openDocument} from "../library/navigation";
import "./ask.css";

export { askLibrary } from "./navigation";

const ASK_QUERY = [LIBRARY_QUERY, "ask"];
/** The chosen scope outlives the screen, as in Kotlin's view model. */
const scopeIndex = createStore(0);

/**
 * Ask your library: a question, the answer with the documents it comes from as chips that open them, and the scope
 * to ask in. The index behind it is updated on demand.
 */
export function AskScreen() {
  const session = useSession();
  const nav = useNavigation();
  const queries = useQueryClient();
  const state = useQuery({ queryKey: ASK_QUERY, queryFn: () => commands.askState() });
  const scope = useStore(scopeIndex);
  const [question, setQuestion] = useState("");
  const aiBusy = busyIn(session, "ai");
  const aiBusyReason = aiBusy === null ? null : waitFor(aiBusy);

  const ask = (text: string) => {
    commands
      .askLibrary(text, scope)
      .then(() => queries.invalidateQueries({ queryKey: ASK_QUERY }))
      .catch(reportError);
  };

  useEffect(() => {
    const request = takeRequest<{ question: string }>("ask");
    if (request) ask(request.question);
  }, [nav.request]); // only a new request matters, not a new ask function

  const scopes = state.data?.scopes ?? ["Whole library"];
  const entries = state.data?.entries ?? [];
  return (
    <div className="screen ask-screen">
      <ScreenHeader
        title="Ask"
        subtitle="Questions answered from your documents, with their sources"
        actions={
          <ActionButtons
            enabled={aiBusyReason === null}
            busyReason={aiBusyReason}
            buttons={[{ label: "Update index", icon: SYNC, onClick: () => void unwrap(commands.updateAskIndex()).catch(reportError) }]}
          />
        }
      />
      <div className="ask-entries scroll">
        {entries.length === 0 ? (
          <EmptyState
            icon={QUESTION_ANSWER}
            title="Ask about your documents"
            message={'For example: "When does my car insurance end?" or "How much did I pay for electricity in 2024?" Update the index once, so questions find documents by meaning, not only by words.'}
          />
        ) : (
          entries.map((entry, index) => <EntryCard key={index} entry={entry} />)
        )}
      </div>
      <div className="row gap-s">
        <ScopeMenu scopes={scopes} index={Math.min(scope, scopes.length - 1)} onSelect={(index) => scopeIndex.set(index)} />
        <TextField
          label="Your question"
          value={question}
          onChange={setQuestion}
          labelBackground="var(--background)"
          style={{ flex: 1 }}
          onEnter={() => {
            if (question.trim() === "" || aiBusy !== null) return;
            ask(question);
            setQuestion("");
          }}
        />
        <IconButton
          icon={SEND}
          label="Ask"
          disabled={question.trim() === "" || aiBusy !== null}
          onClick={() => {
            ask(question);
            setQuestion("");
          }}
        />
      </div>
    </div>
  );
}

function ScopeMenu({ scopes, index, onSelect }: { scopes: string[]; index: number; onSelect: (index: number) => void }) {
  const menu = useMenuAnchor();
  return (
    <>
      <button type="button" className="btn btn-outlined ask-scope" onClick={menu.open}>
        <span className="nowrap">{scopes[index] ?? "Whole library"}</span>
        <Icon svg={ARROW_DROP_DOWN} />
      </button>
      {menu.anchor && (
        <Menu anchor={menu.anchor} onClose={menu.close}>
          {scopes.map((scope, position) => (
            <MenuItem
              key={scope}
              label={scope}
              onClick={() => {
                menu.close();
                onSelect(position);
              }}
            />
          ))}
        </Menu>
      )}
    </>
  );
}

function EntryCard({ entry }: { entry: AskEntryDto_Serialize }) {
  return (
    <AppCard className="ask-entry">
      <span className="title-small">{entry.question}</span>
      {entry.asking ? <span className="body-medium">Searching your documents…</span> : entry.answer !== null && <span className="body-medium selectable ask-answer">{entry.answer}</span>}
      {entry.note && <span className="body-small muted">{entry.note}</span>}
      {entry.citations.length > 0 && (
        <div className="row wrap gap-s">
          {entry.citations.map((citation) => (
            <Chip key={citation.number} icon={DESCRIPTION} label={`[${citation.number}] ${citation.title}`} onClick={() => openDocument(citation.documentId)} />
          ))}
        </div>
      )}
    </AppCard>
  );
}
