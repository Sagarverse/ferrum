/** Metallic Mini enterprise SVG logo */
export function Logo({ size = 32 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 48 48"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      {/* Shield shape */}
      <path
        d="M24 4L6 12V22C6 33.1 13.8 43.1 24 46C34.2 43.1 42 33.1 42 22V12L24 4Z"
        fill="url(#shieldGrad)"
        stroke="url(#shieldStroke)"
        strokeWidth="1.5"
      />
      {/* M letterform */}
      <path
        d="M15 32V18L24 26L33 18V32"
        stroke="#fff"
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
      {/* Center diamond accent */}
      <path
        d="M24 20L27 24L24 28L21 24Z"
        fill="#fff"
        opacity="0.9"
      />
      <defs>
        <linearGradient id="shieldGrad" x1="6" y1="4" x2="42" y2="46">
          <stop offset="0%" stopColor="#00B4D8" />
          <stop offset="100%" stopColor="#0077B6" />
        </linearGradient>
        <linearGradient id="shieldStroke" x1="6" y1="4" x2="42" y2="46">
          <stop offset="0%" stopColor="#00D4FF" stopOpacity="0.6" />
          <stop offset="100%" stopColor="#0077B6" stopOpacity="0.3" />
        </linearGradient>
      </defs>
    </svg>
  );
}
