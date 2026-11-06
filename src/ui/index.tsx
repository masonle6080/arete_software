/**
 * The shared UI base: the components every screen uses in place of its own
 * buttons, headers, status text, tags, and empty lines. Their styles and the
 * design tokens are in `ui.css`.
 */
import type { ButtonHTMLAttributes, ReactNode } from "react";
import { badgeGroup, badgeLabel, type Lifecycle, type StatusOf } from "./badge";

export { badgeGroup, badgeLabel, type BadgeGroup, type Lifecycle, type StatusOf } from "./badge";
export { countLabel } from "./count";
export { icons, type Icon } from "./icons";

const cx = (...names: (string | false | undefined)[]) => names.filter(Boolean).join(" ");

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /**
   * - default: a bordered push button
   * - primary: filled with the accent color, for the main action of a form
   * - plain: no border, for quiet actions such as "Remove" in a row
   * - destructive: a plain button in the error color
   */
  variant?: "default" | "primary" | "plain" | "destructive";
  size?: "regular" | "small";
}

/**
 * A push button in one of the shared styles. Like a plain `<button>`, it
 * submits its form unless `type="button"` is given.
 */
export function Button({ variant = "default", size = "regular", className, ...rest }: ButtonProps) {
  return (
    <button
      className={cx("btn", variant !== "default" && `btn-${variant}`, size === "small" && "btn-small", className)}
      {...rest}
    />
  );
}

interface SectionHeaderProps {
  title: ReactNode;
  /** The heading level. 2 for a section of a screen, 3 for a part of a section. */
  level?: 2 | 3;
  /** A number shown quietly after the title, such as how many items the section holds. */
  count?: number;
  /** Actions shown at the end of the header, such as an "Add" button. */
  children?: ReactNode;
  className?: string;
}

/** The title line of a section or a part of a section, with optional actions at its end. */
export function SectionHeader({ title, level = 2, count, children, className }: SectionHeaderProps) {
  const Heading = level === 2 ? "h2" : "h3";
  return (
    <div className={cx("section-header", `section-header-${level}`, className)}>
      <Heading>
        {title}
        {count !== undefined && <span className="section-count">{count}</span>}
      </Heading>
      {children && <div className="section-actions">{children}</div>}
    </div>
  );
}

interface SegmentedControlProps<T extends string> {
  /** What the control picks, for screen readers. */
  label: string;
  options: readonly { value: T; label: ReactNode }[];
  value: T;
  onChange(value: T): void;
}

/** A row of joined buttons that picks one of a few options, like a macOS segmented control. */
export function SegmentedControl<T extends string>({ label, options, value, onChange }: SegmentedControlProps<T>) {
  return (
    <div className="segmented" role="radiogroup" aria-label={label}>
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          role="radio"
          aria-checked={o.value === value}
          className={cx("segment", o.value === value && "selected")}
          onClick={() => onChange(o.value)}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

interface BadgeProps<L extends Lifecycle> {
  lifecycle: L;
  status: StatusOf[L];
}

/** A status as a small colored label. The color group comes from `badgeGroup`. */
export function Badge<L extends Lifecycle>({ lifecycle, status }: BadgeProps<L>) {
  return <span className={`badge badge-${badgeGroup(lifecycle, status)}`}>{badgeLabel(lifecycle, status)}</span>;
}

interface ChipProps {
  children: ReactNode;
  /** Makes the chip a button, for example to open what it names. */
  onClick?(): void;
  /** Shows a "×" button that removes the chip's link. */
  onRemove?(): void;
  /** The accessible name of the remove button. Defaults to "Remove". */
  removeLabel?: string;
  title?: string;
}

/** A small rounded tag for a link, a tag, or a count. */
export function Chip({ children, onClick, onRemove, removeLabel = "Remove", title }: ChipProps) {
  return (
    <span className={cx("chip", onClick && "chip-link")} title={title}>
      {onClick ? (
        <button type="button" className="chip-label" onClick={onClick}>
          {children}
        </button>
      ) : (
        <span className="chip-label">{children}</span>
      )}
      {onRemove && (
        <button type="button" className="chip-remove" aria-label={removeLabel} title={removeLabel} onClick={onRemove}>
          ×
        </button>
      )}
    </span>
  );
}

interface BreadcrumbProps {
  /** The labels from the root to the open place. */
  items: readonly string[];
  /** Opens one level; 0 is the root. The last item is the open place and is not a button. */
  onOpen(depth: number): void;
}

/** Where the open screen sits, such as `Impermanence › Styles › Hooded Coat (Ash)`. Each part opens its level. */
export function Breadcrumb({ items, onOpen }: BreadcrumbProps) {
  return (
    <nav className="breadcrumb" aria-label="Breadcrumb">
      <ol>
        {items.map((label, i) => (
          <li key={i}>
            {i === items.length - 1 ? (
              <span className="crumb current" aria-current="page">
                {label}
              </span>
            ) : (
              <button type="button" className="crumb" onClick={() => onOpen(i)}>
                {label}
              </button>
            )}
          </li>
        ))}
      </ol>
    </nav>
  );
}

/** The short line an empty section shows, such as "No design files yet." */
export function EmptyText({ children }: { children: ReactNode }) {
  return <p className="empty-text">{children}</p>;
}
