import { useState, useEffect } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';

import { Layout } from './components/layout/Layout';
import { BlankState } from './components/blank/BlankState';
import { ConsoleWindow } from './components/console/ConsoleWindow';
import { ActiveAgentsBar } from './components/console/ActiveAgentsBar';
import { AgentSection } from './components/agents/AgentSection';
import SideBar from './components/sidebar/SideBar';

import { AppState, ChatMessage, BalanceInfo, AgentInfo } from './types';
import { publicAgents, privateAgents } from './mocks/agents';
import { Button } from './components/common/Button';

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

  const [showSplash, setShowSplash] = useState(true);
  const [showConsole, setShowConsole] = useState(false);
  const [showSidebar, setShowSidebar] = useState(false);
  const [showAgents, setShowAgents] = useState(false);

  useEffect(() => {
    const initializeApp = async () => {
      try {


        // if (!window.__TAURI__) {
        //   console.warn("Mock Mode: Загружаем фейковые балансы");
        //   setAppState(prev => ({ ...prev, balances: [{asset_id: 'USDT', balance: '1000.00', raw_balance: 0}] }));
        //   return;
        // }
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

  const handleUiCommand = (command: string) => {
    if (command.includes('console')) {
      setShowSplash(false);
      setShowConsole(true);
    }
    if (command.includes('balances')) {
      setShowSidebar(true);
    }
    if (command.includes('agents')) {
      setShowAgents(true);
    }
  };

  const handleSendMessage = async (message: string) => {
    if (!message.trim() || appState.isLoading) return;

    handleUiCommand(message.toLowerCase());

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
  const SimButtons = () => (
    <div className="fixed bottom-4 left-4 z-50 flex gap-2 p-2 bg-gray-800 rounded-lg opacity-80 hover:opacity-100">
      <Button onClick={() => {
        setShowSplash(false); // Выходим со сплэша
        setShowConsole(prev => !prev); // Переключаем консоль
      }}>
        Toggle Console
      </Button>
      <Button onClick={() => setShowSidebar(prev => !prev)}>
        Toggle Balances
      </Button>
      <Button onClick={() => setShowAgents(prev => !prev)}>
        Toggle Agents
      </Button>
      <Button variant="secondary" onClick={() => {
        setShowSplash(true); setShowConsole(false); setShowSidebar(false); setShowAgents(false);
      }}>
        Reset (Splash)
      </Button>
    </div>
  );


  return (
    <div className='bg-linear-to-b from-[#101010] to-[#0B0B0B]'>
      <SimButtons />

      <AnimatePresence>
        {showSplash && (
          <motion.div
            key="splash"
            exit={{ opacity: 0, scale: 0.95 }}
            transition={{ duration: 0.3 }}
          >
            <BlankState
              onStartListening={() => invoke('stt_start')}
              onStopListening={() => invoke('stt_stop')}
              onTranscript={(text) => handleSendMessage(text)}
            />
          </motion.div>
        )}
      </AnimatePresence>
      {!showSplash && (
        <Layout
          showSidebar={showSidebar}
          sidebar={<AnimatePresence>
            {showSidebar && (
              <motion.div
                key="sidebar"
                initial={{ opacity: 0, x: 50 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 50 }}
                transition={{ duration: 0.4, ease: 'easeInOut' }}
                className="flex flex-col gap-6"
              >
                <SideBar balances={appState.balances} />
              </motion.div>
            )}
          </AnimatePresence>}
        >
          <AnimatePresence>
            {showAgents && (
              <motion.div
                key="agents"
                layout
                initial={{ opacity: 0, y: -30 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -30 }}
                transition={{ duration: 0.4, ease: 'easeInOut' }}
                className="flex flex-col gap-5 min-h-0"
              >
                <AgentSection title="Public Agents" subTitle="Agents from the community" agents={publicAgents} showAddButton/>
                <AgentSection title="Private Agents" subTitle="Your private agents" agents={privateAgents} showAddButton />
              </motion.div>
            )}
          </AnimatePresence>
          <AnimatePresence>
            {showConsole && (
              <motion.div 
                key="console" 
                layout
                initial={{ opacity: 0, y: 30 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.4, ease: 'easeInOut' }}
                className="flex flex-col flex-1 h-full min-h-[300px]"
              >
                <ConsoleWindow
                  messages={appState.messages}
                  isLoading={appState.isLoading}
                  error={appState.error}
                  onSendMessage={handleSendMessage}
                />
              </motion.div>
            )}
          </AnimatePresence>

          <AnimatePresence>
            {showConsole && ( 
              <motion.div
                key="active-agents"
                layout
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
              >
                <ActiveAgentsBar />
              </motion.div>
            )}
          </AnimatePresence>
        </Layout>
      )}
    </div>
  );
}

export default App;
