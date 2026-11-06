import type { Icon } from "../ui";

/** One sidebar row: a Thesis tab, a Shared area, or Settings. */
export interface SidebarRow {
  /** The row key, as `openRow` gives it: `thesis:<tab>` or `area:<id>`. */
  key: string;
  label: string;
  icon: Icon;
  onOpen(): void;
}

interface Props {
  rows: SidebarRow[];
  /** The key of the open row (see `openRow`). */
  openKey: string;
}

/** A flat list of sidebar rows, each with its icon. The open row shows as selected. */
export function SidebarRows({ rows, openKey }: Props) {
  return (
    <ul className="sidebar-rows">
      {rows.map(({ key, label, icon: RowIcon, onOpen }) => (
        <li key={key}>
          <button
            type="button"
            className={key === openKey ? "sidebar-row active" : "sidebar-row"}
            aria-current={key === openKey ? "page" : undefined}
            onClick={onOpen}
          >
            <RowIcon className="sidebar-icon" size={15} strokeWidth={1.75} aria-hidden />
            <span className="sidebar-label">{label}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}
