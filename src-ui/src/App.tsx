import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './styles.css';
import { BalanceInfo, AgentInfo, ChatMessage, AppState } from './types';
import BalanceDisplay from './components/BalanceDisplay';
import AgentStatus from './components/AgentStatus';
import ChatInterface from './components/ChatInterface';

function App() {
  const [appState, setAppState] = useState<AppState>({
    balances: [],
    agents: [],
    messages: [],
    isLoading: false,
    error: undefined,
  });

  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [appState.messages]);

  useEffect(() => {
    // Initialize app data
    const initializeApp = async () => {
      try {
        const [balances, agents] = await Promise.all([
          invoke<BalanceInfo[]>('get_all_balances'),
          invoke<AgentInfo[]>('get_agents'),
        ]);

        setAppState((prev: any) => ({
          ...prev,
          balances,
          agents,
        }));
      } catch (error) {
        console.error('Failed to initialize app:', error);
        setAppState((prev: any) => ({
          ...prev,
          error: error instanceof Error ? error.message : 'Failed to initialize app',
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

    setAppState((prev: AppState) => ({
      ...prev,
      messages: [...prev.messages, userMessage],
      isLoading: true,
      error: undefined,
    }));

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

      // Refresh balances after processing
      const balances = await invoke<BalanceInfo[]>('get_all_balances');

      setAppState((prev: AppState) => ({
        ...prev,
        messages: [...prev.messages, assistantMessage],
        balances,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to process query:', error);
      
      const errorMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        type: 'assistant',
        content: 'Sorry, something went wrong while processing your request.',
        timestamp: new Date(),
        error: error instanceof Error ? error.message : 'Unknown error',
      };

      setAppState((prev: AppState) => ({
        ...prev,
        messages: [...prev.messages, errorMessage],
        isLoading: false,
        error: error instanceof Error ? error.message : 'Failed to process query',
      }));
    }
  };

  return (
    <div className="app">
      <div className="sidebar">
        <BalanceDisplay balances={appState.balances} />
        <AgentStatus agents={appState.agents} />
      </div>
      
      <div className="main-content">
        <div className="header">
          <h1>Unicity Agentic Demo</h1>
          <p>Neurosymbolic Flow Based Programming System</p>
        </div>
        
        <div className="chat-container">
          <ChatInterface
            messages={appState.messages}
            isLoading={appState.isLoading}
            onSendMessage={handleSendMessage}
            error={appState.error}
          />
          <div ref={messagesEndRef} />
        </div>
      </div>
    </div>
  );
}

export default App;