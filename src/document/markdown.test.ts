import { describe, expect, it } from "vitest";
import { fromMarkdown, toMarkdown } from ".";

/** Markdown in the form the editor writes it. Each must reload unchanged. */
const canonical: Record<string, string> = {
  "an empty document": "",
  "one paragraph": "It takes time.\n",
  "headings and paragraphs": "# Patience\n\nFirst paragraph.\n\n## Part two\n\nSecond paragraph.\n",
  "inline marks": "Some *italic*, **bold**, ~~struck~~, and `code` text.\n",
  "a link": "Read [the essay](https://example.com/essay) first.\n",
  "a bullet list": "- one\n- two\n- three\n",
  "an ordered list": "1. one\n2. two\n3. three\n",
  "an ordered list that starts later": "3. three\n4. four\n",
  "a nested list": "- one\n  - one point one\n- two\n",
  "a blockquote": "> A quote.\n>\n> Second paragraph.\n",
  "a loose list": "- one\n\n- two\n",
  "a fenced code block": "```ts\nconst x = 1;\n```\n",
  "a horizontal rule": "Above.\n\n---\n\nBelow.\n",
  "a hard break": "Line one\\\nline two\n",
  "escaped Markdown characters": "Not \\*italic\\* and 2 \\* 3.\n",
  "unicode text": "été — “quoted” ✓\n",
};

describe("Markdown without Passage markers", () => {
  for (const [name, md] of Object.entries(canonical)) {
    it(`round-trips ${name} without change`, () => {
      expect(toMarkdown(fromMarkdown(md))).toBe(md);
    });
  }

  it("settles hand-written Markdown into one stable form", () => {
    const handWritten = "Title\n=====\n\n* a\n* b\n\n\n\nText with _emphasis_\nover two lines.";

    const once = toMarkdown(fromMarkdown(handWritten));

    expect(once).toBe("# Title\n\n- a\n- b\n\nText with *emphasis* over two lines.\n");
    expect(toMarkdown(fromMarkdown(once))).toBe(once);
  });

  it("keeps a literal HTML comment as text", () => {
    const md = "A <!-- note --> here.\n";

    expect(fromMarkdown(md).textContent).toBe("A <!-- note --> here.");
  });
});
