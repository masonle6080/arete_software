// Landing on a Passage: the range of the document it covers (ticket #16).
import { describe, expect, it } from "vitest";
import { fromMarkdown, passageRange } from ".";

function textIn(md: string, id: string): string | null {
  const doc = fromMarkdown(md);
  const range = passageRange(doc, id);
  return range && doc.textBetween(range.from, range.to, "\n");
}

describe("passageRange", () => {
  it("covers exactly the text of a Passage", () => {
    expect(textIn("Hello <!--passage:p-a-->brave new<!--/passage:p-a--> world.\n", "p-a")).toBe("brave new");
  });

  it("spans every block of a Passage that crosses blocks", () => {
    const md = "One <!--passage:p-a-->end of one.\n\nStart of two<!--/passage:p-a--> two.\n";
    expect(textIn(md, "p-a")).toBe("end of one.\nStart of two");
  });

  it("is null for a Passage that is not in the document", () => {
    expect(textIn("Hello <!--passage:p-a-->brave<!--/passage:p-a-->.\n", "p-gone")).toBeNull();
  });
});
