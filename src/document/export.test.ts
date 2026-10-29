import { describe, expect, it } from "vitest";
import { exportHtml, exportMarkdown } from ".";

const marked =
  "Hello <!--passage:a1-->brave <!--passage:b2-->new<!--/passage:a1--> world<!--/passage:b2-->.\n\n" +
  "<!--passage:c3-->First block\n\nsecond block<!--/passage:c3--> ends.\n";

describe("exportMarkdown", () => {
  it("removes every Passage marker and keeps the text", () => {
    expect(exportMarkdown("", marked)).toBe("Hello brave new world.\n\nFirst block\n\nsecond block ends.\n");
  });

  it("puts the title first as a heading", () => {
    expect(exportMarkdown("Patience", "It takes <!--passage:x-->time<!--/passage:x-->.\n")).toBe(
      "# Patience\n\nIt takes time.\n",
    );
  });

  it("keeps text that only looks like a marker", () => {
    const md = "Write \\<!--passage:x--> to mark.\n\n```\n<!--passage:y-->\n```\n";
    expect(exportMarkdown("", md)).toBe(md);
  });

  it("keeps marks around a Passage", () => {
    expect(exportMarkdown("", "**bold <!--passage:p-->and marked<!--/passage:p-->**\n")).toBe(
      "**bold and marked**\n",
    );
  });
});

describe("exportHtml", () => {
  it("renders the title and body with no Passage markers", () => {
    const html = exportHtml("Patience", "It takes <!--passage:x-->*time*<!--/passage:x-->.\n");
    expect(html).toBe("<h1>Patience</h1>\n<p>It takes <em>time</em>.</p>\n");
  });

  it("escapes HTML in the title and the text", () => {
    expect(exportHtml("<b>", "a <script>x</script>\n")).toBe(
      "<h1>&lt;b&gt;</h1>\n<p>a &lt;script&gt;x&lt;/script&gt;</p>\n",
    );
  });
});
