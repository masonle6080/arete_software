import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import type { View } from "../../shell/sidebar";
import { listenTreeChanges } from "../../shell/treeChanges";
import { Button, EmptyText, SectionHeader } from "../../ui";
import { CardFaceView } from "../style/DropStyles";
import { itemView, loadThesisContents, type ThesisContents } from "./contents";
import type { Thesis } from "./types";

interface Props {
  thesis: Thesis;
  /** Opens a tab of the current Thesis, or an item on top of its tab. */
  onGo(view: View): void;
}

/**
 * The body of the Thesis Overview, under the header that holds the statement,
 * the status Badge and moves, and the chips: a row of Drop Covers, the linked
 * Essays, and the Moodboards. Each section title opens its tab, and each item
 * opens that item.
 */
export function ThesisOverview({ thesis, onGo }: Props) {
  const [contents, setContents] = useState<ThesisContents>();
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    loadThesisContents(thesis.id).then(
      (c) => {
        setContents(c);
        setError(undefined);
      },
      (raw) => setError(toCoreError(raw)),
    );
  }, [thesis]);
  useEffect(() => {
    reload();
    return listenTreeChanges(reload);
  }, [reload]);

  if (error) return <p className="error">{error.message}</p>;
  if (!contents) return null;
  const { styles, essays, moodboards } = contents;

  const header = (tab: string, title: string, count: number) => (
    <SectionHeader title={title} count={count}>
      <Button size="small" onClick={() => onGo(itemView.tab(tab))}>
        Open {title}
      </Button>
    </SectionHeader>
  );

  return (
    <div className="thesis-overview">
      <section className="thesis-section">
        {header("drop", "Drop", styles.length)}
        {styles.length === 0 ? (
          <EmptyText>No Styles yet.</EmptyText>
        ) : (
          <ul className="overview-covers">
            {styles.map((s) => (
              <li key={s.id} className={s.status === "cut" ? "cut" : undefined}>
                <button type="button" className="style-card" onClick={() => onGo(itemView.style(s))}>
                  <CardFaceView style={s} />
                  <span className="style-card-name">{s.name}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
      <section className="thesis-section">
        {header("essays", "Essays", essays.length)}
        {essays.length === 0 ? (
          <EmptyText>No Essays yet.</EmptyText>
        ) : (
          <ul className="thesis-essays">
            {essays.map((e) => (
              <li key={e.id}>
                <button type="button" className="open" onClick={() => onGo(itemView.essay(e))}>
                  {e.title || "Untitled"}
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
      <section className="thesis-section">
        {header("moodboards", "Moodboards", moodboards.length)}
        {moodboards.length === 0 ? (
          <EmptyText>No Moodboards yet.</EmptyText>
        ) : (
          <ul className="thesis-essays">
            {moodboards.map((b) => (
              <li key={b.id}>
                <button type="button" className="open" onClick={() => onGo(itemView.moodboard(b))}>
                  {b.title || "Untitled"}
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
