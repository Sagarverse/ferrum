import React, { createContext, useContext, useState } from 'react';

export type BrandName = 'Ferrum' | 'Metallic Mini';

interface BrandContextType {
  brandName: BrandName;
  setBrandName: (name: BrandName) => void;
}

const BrandContext = createContext<BrandContextType>({
  brandName: 'Ferrum',
  setBrandName: () => {},
});

export function BrandProvider({ children }: { children: React.ReactNode }) {
  const [brandName, setBrandNameState] = useState<BrandName>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('ferrum_brand_name');
      if (saved === 'Metallic Mini' || saved === 'Ferrum') {
        return saved as BrandName;
      }
      if (saved === 'Metallic') {
        return 'Metallic Mini';
      }
    }
    return 'Ferrum';
  });

  const setBrandName = (name: BrandName) => {
    setBrandNameState(name);
    if (typeof window !== 'undefined') {
      localStorage.setItem('ferrum_brand_name', name);
    }
  };

  return (
    <BrandContext.Provider value={{ brandName, setBrandName }}>
      {children}
    </BrandContext.Provider>
  );
}

export function useBrand() {
  return useContext(BrandContext);
}
