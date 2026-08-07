import type { SVGProps } from "react";

export type IconName =
  | "add"
  | "apps"
  | "back"
  | "battery"
  | "check"
  | "cloud"
  | "community"
  | "disc"
  | "discover"
  | "folder"
  | "gamepad"
  | "home"
  | "library"
  | "menu"
  | "profile"
  | "search"
  | "settings"
  | "shield"
  | "spark"
  | "wifi";

interface IconProps extends SVGProps<SVGSVGElement> {
  name: IconName;
}

export function Icon({ name, ...props }: IconProps) {
  const common = {
    fill: "none",
    stroke: "currentColor",
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    strokeWidth: 1.8,
  };

  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" {...props}>
      <g {...common}>{paths[name]}</g>
    </svg>
  );
}

const paths: Record<IconName, React.ReactNode> = {
  home: (
    <>
      <path d="m3.5 10.4 8.5-7 8.5 7" />
      <path d="M5.5 9.2V21h13V9.2M9.5 21v-6h5v6" />
    </>
  ),
  library: (
    <>
      <rect x="3" y="4" width="5" height="16" rx="1" />
      <rect x="9.5" y="4" width="5" height="16" rx="1" />
      <path d="m16 5 3.1-.8L22 18.8l-3.2.7z" />
    </>
  ),
  discover: (
    <>
      <circle cx="12" cy="12" r="8.5" />
      <path d="m15.7 8.3-2.1 5.3-5.3 2.1 2.1-5.3z" />
    </>
  ),
  community: (
    <>
      <path d="M8.5 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7ZM15.8 10a3 3 0 1 0 0-6" />
      <path d="M2.5 20c.3-4.1 2.3-6.2 6-6.2s5.7 2.1 6 6.2M15.2 13.5c3.9 0 6 2 6.3 5.7" />
    </>
  ),
  apps: (
    <>
      <rect x="3" y="3" width="7" height="7" rx="1.5" />
      <rect x="14" y="3" width="7" height="7" rx="1.5" />
      <rect x="3" y="14" width="7" height="7" rx="1.5" />
      <rect x="14" y="14" width="7" height="7" rx="1.5" />
    </>
  ),
  add: <path d="M12 4v16M4 12h16" />,
  settings: (
    <>
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1a1.7 1.7 0 0 0 1.9.3A1.7 1.7 0 0 0 10 3V2.8h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1Z" />
    </>
  ),
  gamepad: (
    <>
      <path d="M8.1 7.5h7.8c2.4 0 3.8 1.4 4.5 4.2l.8 3.2c.6 2.4-.7 4.1-2.4 4.1-1.4 0-2.2-1.1-3.2-2.4H8.4C7.4 17.9 6.6 19 5.2 19c-1.7 0-3-1.7-2.4-4.1l.8-3.2c.7-2.8 2.1-4.2 4.5-4.2Z" />
      <path d="M8 10.5v4M6 12.5h4M16.8 11.2h.1M18.6 13h.1" />
    </>
  ),
  wifi: (
    <>
      <path d="M3 9a14.5 14.5 0 0 1 18 0M6 12.5a9.8 9.8 0 0 1 12 0M9.2 16a4.6 4.6 0 0 1 5.6 0" />
      <circle cx="12" cy="19" r=".7" fill="currentColor" stroke="none" />
    </>
  ),
  battery: (
    <>
      <rect x="2.5" y="7" width="17" height="10" rx="2" />
      <path d="M22 10v4M5.5 10h10.5v4H5.5z" fill="currentColor" stroke="none" />
    </>
  ),
  profile: (
    <>
      <circle cx="12" cy="8" r="3.5" />
      <path d="M4.5 21c.4-4.7 2.9-7 7.5-7s7.1 2.3 7.5 7" />
    </>
  ),
  search: (
    <>
      <circle cx="10.5" cy="10.5" r="6.5" />
      <path d="m15.5 15.5 5 5" />
    </>
  ),
  menu: <path d="M4 7h16M4 12h16M4 17h16" />,
  back: <path d="m14.5 5-7 7 7 7" />,
  check: <path d="m5 12.5 4.2 4.2L19 7" />,
  shield: (
    <>
      <path d="M12 3 20 6v5c0 5.1-3 8.5-8 10-5-1.5-8-4.9-8-10V6z" />
      <path d="m8.5 12 2.2 2.2 4.8-5" />
    </>
  ),
  spark: <path d="m12 2 1.6 6.4L20 10l-6.4 1.6L12 18l-1.6-6.4L4 10l6.4-1.6z" />,
  disc: (
    <>
      <circle cx="12" cy="12" r="9" />
      <circle cx="12" cy="12" r="2.5" />
    </>
  ),
  folder: <path d="M3 6.5h7l2 2h9v10.5H3z" />,
  cloud: (
    <path d="M7 18h10a4 4 0 0 0 .6-8 6 6 0 0 0-11.5 1.2A3.5 3.5 0 0 0 7 18Z" />
  ),
};
