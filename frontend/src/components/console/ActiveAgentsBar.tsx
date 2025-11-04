// src/components/console/ActiveAgentsBar.tsx
import { X } from 'lucide-react';

const activeAgents = [
  { id: 1, name: 'P2P Trading' },
  { id: 2, name: 'Friendly Miners' },
  { id: 3, name: 'P2P Games' },
];

export const ActiveAgentsBar = () => {
  return (
    <div className="
      bg-brand-bg-light 
      rounded-xl 
      py-2 px-4 
      flex items-center gap-4
    ">
      <span className="text-sm font-medium text-brand-text-light whitespace-nowrap">
        Active agents
      </span>
      
      <div className="flex items-center gap-2 overflow-x-auto">
        {activeAgents.map(agent => (
          <div 
            key={agent.id}
            className="
              flex items-center gap-1.5
              bg-brand-green-dark text-brand-green
              px-3 py-1 rounded-full
              text-xs font-medium
              cursor-pointer
            "
          >
            <span>{agent.name}</span>
            <X className="w-3.5 h-3.5" />
          </div>
        ))}
      </div>
    </div>
  );
};