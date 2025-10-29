import React from 'react';
import { AgentInfo } from '../types';

interface AgentStatusProps {
  agents: AgentInfo[];
}

const AgentStatus: React.FC<AgentStatusProps> = ({ agents }) => {
  return (
    <div className="agents-card">
      <div className="agents-title">Available Agents</div>
      {agents.map((agent) => (
        <div key={agent.id} className="agent-item">
          <div className="agent-name">{agent.label}</div>
          <div className="agent-description">{agent.description}</div>
        </div>
      ))}
    </div>
  );
};

export default AgentStatus;