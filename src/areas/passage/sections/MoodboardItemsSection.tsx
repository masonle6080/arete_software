import { useEffect, useState } from "react";
import { toCoreError } from "../../../shell/coreError";
import { mediaApi, moodboardApi } from "../../moodboard/api";
import { mimeTypeOf } from "../../moodboard/mediaType";
import type { LinkedMoodboardItem } from "../../moodboard/types";
import type { PassageSection, PassageSectionProps } from "./index";

/** The canvas items linked to this Passage, from every Moodboard. */
function MoodboardItems({ passage }: PassageSectionProps) {
  const [items, setItems] = useState<LinkedMoodboardItem[]>();
  const [error, setError] = useState<string>();

  useEffect(() => {
    moodboardApi.itemsOfPassage(passage.id).then(setItems, (e) => setError(toCoreError(e).message));
  }, [passage.id]);

  if (error) return <p className="error">{error}</p>;
  if (!items) return null;
  if (items.length === 0) {
    return <p className="muted">No Moodboard items. Select an item on a Moodboard canvas to link it.</p>;
  }
  return (
    <ul className="passage-moodboard-items">
      {items.map((item) => (
        <li key={`${item.moodboard.id}:${item.elementId}`}>
          <ItemPreview item={item} />
          <span className="muted">
            {item.moodboard.owner.kind === "style" ? "Style" : "Thesis"} Moodboard “{item.moodboard.title}”
          </span>
        </li>
      ))}
    </ul>
  );
}

function ItemPreview({ item }: { item: LinkedMoodboardItem }) {
  if (item.media) return <MediaThumb media={item.media} />;
  if (item.text) return <span className="item-text">{item.text}</span>;
  if (item.color) return <span className="item-swatch" style={{ background: item.color }} title={item.color} />;
  return <span className="item-text">{item.elementType}</span>;
}

/** A thumbnail of an image read from the library. */
function MediaThumb({ media }: { media: string }) {
  const [url, setUrl] = useState<string>();
  useEffect(() => {
    let objectUrl: string | undefined;
    let live = true;
    mediaApi.read(media).then(
      (bytes) => {
        if (!live) return;
        objectUrl = URL.createObjectURL(new Blob([bytes], { type: mimeTypeOf(media) }));
        setUrl(objectUrl);
      },
      () => undefined, // A missing image shows as an empty frame.
    );
    return () => {
      live = false;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [media]);
  return <span className="item-thumb">{url && <img src={url} alt="" />}</span>;
}

export const moodboardItemsSection: PassageSection = {
  id: "moodboard-items",
  title: "Moodboard items",
  Component: MoodboardItems,
};
