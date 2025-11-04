import React from 'react';
import type { IAgent } from '../../types';
import { AgentCard } from './AgentCard';
import { Plus } from 'lucide-react';

interface AgentSectionProps {
  title: string;
  subTitle: string;
  agents: IAgent[];
  showAddButton?: boolean;
}

export const AgentSection: React.FC<AgentSectionProps> = ({ title, subTitle, agents, showAddButton = false }) => {
  return (
    <div>
      <h2 className="text-xl font-semibold text-brand-text-dim h-6">
        {title}
      </h2>
      <h3 className='text-sm font-medium text-[#262525] mb-3'>
        {subTitle}
      </h3>
      <div className="
        grid grid-flow-col auto-cols-[13rem] gap-2 
        overflow-x-auto overflow-y-hidden
        pb-2
        snap-x snap-mandatory
        [-webkit-overflow-scrolling:touch]
      ">
        {agents.map((agent) => (
          <div key={agent.id} className="snap-start">
            <AgentCard agent={agent} />
          </div>
        ))}

        {showAddButton && (
          <div className="
            flex flex-col items-center justify-center
            p-4 w-52 h-32
            border-2 border-dashed border-[#868686]
            rounded-xl 
            cursor-pointer
            transition-all duration-200
            hover:border-white
            snap-start
          ">
            <div className="
              flex items-center justify-center 
              w-13 h-13 
              bg-brand-bg-dark
              rounded-full
            ">
              <Plus className="w-6 h-6 text-white" />
            </div>
            
            <p className="mt-3 font-medium text-white">
              Add new
            </p>
          </div>
        )}
      </div>
    </div>
  );
};