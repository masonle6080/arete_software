import type { Area, AreaProps } from "../../shell/area";

function LibraryScreen({ library }: AreaProps) {
  return (
    <section>
      <h1>Library</h1>
      <dl className="facts">
        <dt>Folder</dt>
        <dd>{library.path}</dd>
        <dt>Schema version</dt>
        <dd>{library.schemaVersion}</dd>
      </dl>
    </section>
  );
}

export const libraryArea: Area = { id: "library", label: "Library", Screen: LibraryScreen };
