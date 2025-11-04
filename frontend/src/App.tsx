import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Layout } from './components/layout/Layout';
import { ConsoleWindow } from './components/console/ConsoleWindow';
import { ActiveAgentsBar } from './components/console/ActiveAgentsBar';
import { publicAgents, privateAgents } from './mocks/agents';
import { AgentSection } from './components/agents/AgentSection';

import { AppState, ChatMessage, BalanceInfo, AgentInfo } from './types';
import SideBar from './components/sidebar/SideBar';

function App() {
  const [appState, setAppState] = useState<AppState>({
    balances: [],
    agents: [],
    messages: [
      {
        id: '1',
        type: 'assistant',
        content: 'Welcome to Unicity AgentSphere. Connecting to backend...',
        timestamp: new Date(),
      }
    ],
    isLoading: false,
    error: undefined,
  });

  useEffect(() => {
    const initializeApp = async () => {
      try {
        setAppState(prev => ({ ...prev, isLoading: true }));

        const [balances, agents] = await Promise.all([
          invoke<BalanceInfo[]>('get_all_balances'),
          invoke<AgentInfo[]>('get_agents'),
        ]);

        setAppState(prev => ({
          ...prev,
          balances,
          agents,
          isLoading: false,
          messages: [
            ...prev.messages, 
            { id: '2', type: 'assistant', content: 'Connection successful. Agents loaded.', timestamp: new Date() }
          ]
        }));
      } catch (error) {
        console.error('Failed to initialize app:', error);
        const errorMsg = error instanceof Error ? error.message : 'Failed to initialize app';
        setAppState(prev => ({
          ...prev,
          isLoading: false,
          error: errorMsg,
        }));
      }
    };

    initializeApp();
  }, []);

  const handleSendMessage = async (message: string) => {
    if (!message.trim() || appState.isLoading) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      type: 'user',
      content: message,
      timestamp: new Date(),
    };
    setAppState(prev => ({ ...prev, messages: [...prev.messages, userMessage], isLoading: true, error: undefined }));

    try {
      const response = await invoke<any>('process_query', { query: message });

      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: response.response,
        timestamp: new Date(),
        steps: response.steps,
        error: response.error,
      };

      const balances = await invoke<BalanceInfo[]>('get_all_balances');

      setAppState(prev => ({
        ...prev,
        messages: [...prev.messages, assistantMessage],
        balances,
        isLoading: false,
      }));

    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      const errorMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: 'Sorry, something went wrong processing your request.',
        timestamp: new Date(),
        error: errorMsg,
      };
      setAppState(prev => ({
        ...prev,
        messages: [...prev.messages, errorMessage],
        isLoading: false,
        error: errorMsg,
      }));
    }
  };
  return (
    <Layout
      sidebar={<SideBar balances={appState.balances}/>}
    >
      <div className="flex flex-col gap-5 min-h-0 h-full">
        <AgentSection 
          title="Public Agents"
          subTitle='Ready-to-use agents from the community' 
          agents={publicAgents}
          showAddButton={true} 
        />
        <AgentSection 
          title="Private Agents" 
          subTitle='Ready-to-use private agents'
          agents={privateAgents} 
          showAddButton={true} 
        />
        
        <div className="flex-1 min-h-0 space-y-2">
          <ConsoleWindow 
            messages={appState.messages}
            isLoading={appState.isLoading}
            error={appState.error}
            onSendMessage={handleSendMessage}
          />
          <ActiveAgentsBar />
        </div>
      </div>
    </Layout>
  );
}

export default App;
