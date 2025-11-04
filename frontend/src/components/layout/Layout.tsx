// src/components/layout/Layout.tsx
import React from 'react';
import { Header } from './Header';

interface LayoutProps {
  children: React.ReactNode;
  sidebar: React.ReactNode;
}

export const Layout: React.FC<LayoutProps> = ({ children, sidebar }) => {
  return (
    <div className="min-h-screen flex flex-col bg-linear-to-b py-7 from-[#101010] to-[#0B0B0B]">
      <div className="mx-auto w-full max-w-[1300px] px-6">
        <Header />

        <main
          className="
            flex-1 
            grid grid-cols-1 lg:grid-cols-[1fr_400px] 
            gap-8 py-10
          "
        >
          <div className="flex min-w-0 flex-col gap-8">
            {children}
          </div>

          <aside className="flex flex-col gap-6">
            {sidebar}
          </aside>
        </main>
      </div>
    </div>
  );
};