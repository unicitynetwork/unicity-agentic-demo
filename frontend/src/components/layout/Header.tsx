// src/components/layout/Header.tsx
import { HelpCircle, BotMessageSquare } from 'lucide-react';

export const Header = () => {
  return (
    <header className="
      flex items-center justify-between
      py-4
      border-b-2 border-brand-bg-border
    ">
      <div className="flex items-center gap-2">
        <div className='flex justify-center items-center w-11 h-11 bg-linear-to-b rounded-xl from-[#C7FD4A] to-[#8CD616]'>
          <BotMessageSquare className='w-[30px] h-[30px] stroke-[#1D1D1D]'/>
        </div>
        <h1 className="text-2xl font-bold text-brand-text-light">
          Unicity AgentSphere
        </h1>
      </div>
      
      <a 
        href="#" 
        className="
          flex items-center gap-1  font-medium
          text-md text-brand-text-dim 
          hover:text-brand-text-light
          transition-colors
        "
      >
        <HelpCircle className="w-4 h-4 mt-0.5" />
        <span>Help</span>
      </a>
    </header>
  );
};