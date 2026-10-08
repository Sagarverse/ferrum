/** Ferrum — Iron-forged shield SVG logo */
export function Logo({ size = 32 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 48 48"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      {/* Shield shape — iron gradient */}
      <path
        d="M24 4L6 12V22C6 33.1 13.8 43.1 24 46C34.2 43.1 42 33.1 42 22V12L24 4Z"
        fill="url(#ferrumShieldGrad)"
        stroke="url(#ferrumShieldStroke)"
        strokeWidth="1.5"
      />
      {/* F letterform — iron brand */}
      <path
        d="M18 16H30M18 16V34M18 24H28"
        stroke="#FFF"
        strokeWidth="2.8"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
      {/* Anvil spark accent */}
      <circle cx="32" cy="14" r="1.5" fill="#FFD699" opacity="0.8" />
      <circle cx="34" cy="12" r="1" fill="#FFD699" opacity="0.5" />
      <defs>
        <linearGradient id="ferrumShieldGrad" x1="6" y1="4" x2="42" y2="46">
          <stop offset="0%" stopColor="#F0A54B" />
          <stop offset="50%" stopColor="#C06B1D" />
          <stop offset="100%" stopColor="#8B4513" />
        </linearGradient>
        <linearGradient id="ferrumShieldStroke" x1="6" y1="4" x2="42" y2="46">
          <stop offset="0%" stopColor="#FFD699" stopOpacity="0.6" />
          <stop offset="100%" stopColor="#8B4513" stopOpacity="0.3" />
        </linearGradient>
      </defs>
    </svg>
  );
}
