import React from 'react';
import { Header } from './Header';
import { motion } from 'framer-motion';

interface LayoutProps {
  children: React.ReactNode;
  sidebar: React.ReactNode;
  showSidebar: boolean;
}

export const Layout: React.FC<LayoutProps> = ({ children, sidebar, showSidebar }) => {
  return (
    <div className="min-h-screen flex flex-col bg-linear-to-b py-7 from-[#101010] to-[#0B0B0B]">
      <div className="mx-auto w-full max-w-[1300px] px-6">
        <Header />

        <main
          className={`
            flex-1 
            grid grid-cols-1 
            gap-8 py-10
            ${showSidebar ? 'lg:grid-cols-[1fr_400px]' : 'lg:grid-cols-1'}
          `}
        >
          <motion.div layout transition={{ duration: 0.4, ease: 'easeInOut' }} className="flex min-w-0 flex-col gap-8 h-full">
            {children}
          </motion.div>

          {showSidebar && (
            <motion.aside 
              layout 
              transition={{ duration: 0.4, ease: 'easeInOut' }} 
              className="flex flex-col gap-6"
            >
              {sidebar}
            </motion.aside>
          )}
        </main>
      </div>
    </div>
  );
};