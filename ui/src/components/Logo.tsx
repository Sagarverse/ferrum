import { useBrand } from '../context/BrandContext';

export function Logo({ size = 32 }: { size?: number }) {
  const { brandName } = useBrand();
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 48 48"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      style={{ filter: brandName === 'Ferrum' ? 'drop-shadow(0 0 8px rgba(255, 110, 0, 0.4))' : 'drop-shadow(0 0 8px rgba(0, 180, 216, 0.4))' }}
    >
      {/* Shield shape — dynamic gradient */}
      <path
        d="M24 4L6 12V22C6 33.1 13.8 43.1 24 46C34.2 43.1 42 33.1 42 22V12L24 4Z"
        fill={brandName === 'Ferrum' ? "url(#ferrumOrangeShieldGrad)" : "url(#metallicBlueShieldGrad)"}
        stroke={brandName === 'Ferrum' ? "url(#ferrumOrangeShieldStroke)" : "url(#metallicBlueShieldStroke)"}
        strokeWidth="1.5"
      />
      {brandName === 'Ferrum' ? (
        <path
          d="M18 16H30M18 16V34M18 24H28"
          stroke="#FFFFFF"
          strokeWidth="2.8"
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
        />
      ) : (
        <path
          d="M16 34V16L24 24L32 16V34"
          stroke="#FFFFFF"
          strokeWidth="2.8"
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
        />
      )}
      {/* Spark accents */}
      <circle cx="32" cy="14" r="1.5" fill={brandName === 'Ferrum' ? "#FFD166" : "#90E0EF"} opacity="0.9" />
      <circle cx="34" cy="12" r="1" fill={brandName === 'Ferrum' ? "#FFD166" : "#90E0EF"} opacity="0.7" />
      <defs>
        <linearGradient id="ferrumOrangeShieldGrad" x1="6" y1="4" x2="42" y2="46">
          <stop offset="0%" stopColor="#FF8C00" />
          <stop offset="50%" stopColor="#FF4500" />
          <stop offset="100%" stopColor="#B32400" />
        </linearGradient>
        <linearGradient id="ferrumOrangeShieldStroke" x1="6" y1="4" x2="42" y2="46">
          <stop offset="0%" stopColor="#FFE066" stopOpacity="0.9" />
          <stop offset="100%" stopColor="#FF4500" stopOpacity="0.6" />
        </linearGradient>
        <linearGradient id="metallicBlueShieldGrad" x1="6" y1="4" x2="42" y2="46">
          <stop offset="0%" stopColor="#00B4D8" />
          <stop offset="50%" stopColor="#0077B6" />
          <stop offset="100%" stopColor="#023E8A" />
        </linearGradient>
        <linearGradient id="metallicBlueShieldStroke" x1="6" y1="4" x2="42" y2="46">
          <stop offset="0%" stopColor="#90E0EF" stopOpacity="0.9" />
          <stop offset="100%" stopColor="#0077B6" stopOpacity="0.6" />
        </linearGradient>
      </defs>
    </svg>
  );
}
