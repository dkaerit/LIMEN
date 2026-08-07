import type { ButtonHTMLAttributes, ReactNode } from "react";

type FocusButtonProps = Omit<
  ButtonHTMLAttributes<HTMLButtonElement>,
  "tabIndex"
> & {
  focusId: string;
  focused: boolean;
};

export function FocusButton({
  focusId,
  focused,
  className = "",
  children,
  ...props
}: FocusButtonProps) {
  return (
    <button
      {...props}
      className={`focus-surface ${focused ? "is-focused" : ""} ${className}`.trim()}
      data-focus-id={focusId}
      tabIndex={focused ? 0 : -1}
    >
      {children}
    </button>
  );
}

export function PlatformBadge({ children }: { children: ReactNode }) {
  return <span className="platform-badge">{children}</span>;
}

export function ControllerHint({
  glyph,
  children,
}: {
  glyph: ReactNode;
  children: ReactNode;
}) {
  return (
    <span className="controller-hint">
      <span className="controller-glyph" aria-hidden="true">
        {glyph}
      </span>
      {children}
    </span>
  );
}
