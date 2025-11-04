import React from 'react';
import type { IAgent } from '../../types';

interface AgentCardProps {
  agent: IAgent;
}

export const AgentCard: React.FC<AgentCardProps> = ({ agent }) => {
  const { name, Icon } = agent

  return (
    <div className='bg-linear-to-r from-[#A6FF00] to-[#8AD515] rounded-xl p-px w-52 h-32'>
      <div className="
      flex flex-col items-center justify-center w-full h-full
      p-4 
      bg-linear-to-r from-[#333E1A] to-[#273610]
      rounded-xl 
      cursor-pointer
      transition-all duration-200
      hover:border-brand-green
      hover:shadow-lg hover:shadow-brand-green/10
    ">
        <div className="
        flex items-center justify-center 
        w-13 h-13 
        bg-[#3A4D13]
        rounded-full
      ">
          <Icon className="w-6 h-6 text-brand-green" strokeWidth={1.5} />
        </div>

        <p className="mt-1 font-bold text-white truncate">
          {name}
        </p>
        <p className="text-xs font-semibold text-white truncate opacity-25">
          Click to interact
        </p>
      </div>
    </div>
  )
}