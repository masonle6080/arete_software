import { describe, expect, it } from "vitest";
import type { DesignDecision } from "../designDecision/api";
import type { Passage } from "../passage/api";
import { styleTraceHtml, styleTraceMarkdown } from "./trace";

const style = { name: "Patience Tee", color: "Black" };

function passage(id: string, text: string, documentTitle: string, orphaned = false): Passage {
  return {
    id,
    documentId: 1,
    documentKind: "essay",
    documentTitle,
    lastKnownText: text,
    orphaned,
    createdAt: "",
    updatedAt: "",
  };
}

function decision(id: number, aspect: DesignDecision["aspect"], rationale: string, passages: Passage[] = []) {
  return { id, styleId: 1, aspect, rationale, decorationId: null, passages } satisfies DesignDecision;
}

describe("styleTraceMarkdown", () => {
  it("lists each Design Decision with its rationale and Passage text", () => {
    const md = styleTraceMarkdown(style, [
      decision(1, "fabric", "Heavy cotton, because patience is weight.", [
        passage("a", "Time is a cloth that thickens.", "On Patience"),
      ]),
      decision(2, "color", "Black absorbs the hours."),
    ]);
    expect(md).toBe(
      [
        "# Patience Tee (Black)",
        "",
        "## Fabric",
        "",
        "Heavy cotton, because patience is weight.",
        "",
        "> Time is a cloth that thickens.",
        ">",
        "> — On Patience",
        "",
        "## Color",
        "",
        "Black absorbs the hours.",
        "",
      ].join("\n"),
    );
  });

  it("exports an orphaned Passage with its last known text", () => {
    const md = styleTraceMarkdown(style, [
      decision(1, "silhouette", "Boxy.", [passage("a", "What was cut away.", "Drafts", true)]),
    ]);
    expect(md).toContain("> What was cut away.\n>\n> — Drafts\n");
  });

  it("keeps multi-line text inside its quote and paragraph", () => {
    const md = styleTraceMarkdown(style, [
      decision(1, "other", "First line.\nSecond line.", [passage("a", "One.\n\nTwo.", "")]),
    ]);
    expect(md).toContain("## Other\n\nFirst line.\nSecond line.\n\n> One.\n>\n> Two.\n>\n> — Untitled\n");
  });

  it("keeps a Style with no Design Decisions to its title", () => {
    expect(styleTraceMarkdown(style, [])).toBe("# Patience Tee (Black)\n");
  });

  it("does not let user text start a Markdown heading or list", () => {
    const md = styleTraceMarkdown(style, [
      decision(1, "decoration", "# not a heading\n- not a list", [passage("a", "> not nested", "E")]),
    ]);
    expect(md).toContain("\\# not a heading\n\\- not a list\n");
    expect(md).toContain("> \\> not nested\n");
  });
});

describe("styleTraceHtml", () => {
  it("shows each Design Decision with its rationale and Passage text", () => {
    const html = styleTraceHtml(style, [
      decision(1, "fabric", "Heavy cotton.\nSoft hand.", [passage("a", "Time thickens.", "On Patience", true)]),
    ]);
    expect(html).toBe(
      "<h1>Patience Tee (Black)</h1>\n" +
        "<h2>Fabric</h2>\n" +
        "<p>Heavy cotton.<br>Soft hand.</p>\n" +
        '<blockquote><p>Time thickens.</p><p class="trace-source">— On Patience</p></blockquote>\n',
    );
  });

  it("escapes HTML in user text", () => {
    const html = styleTraceHtml({ name: "<b>", color: "" }, [decision(1, "other", "a & <i>", [])]);
    expect(html).toBe("<h1>&lt;b&gt;</h1>\n<h2>Other</h2>\n<p>a &amp; &lt;i&gt;</p>\n");
  });
});
