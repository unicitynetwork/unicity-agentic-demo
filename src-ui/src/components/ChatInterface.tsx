import React, { useState, useRef, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { ChatMessage, ExecutionStep } from '../types';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

interface ChatInterfaceProps {
  messages: ChatMessage[];
  isLoading: boolean;
  onSendMessage: (message: string) => void;
  error?: string;
}

const ChatInterface: React.FC<ChatInterfaceProps> = ({
  messages,
  isLoading,
  onSendMessage,
  error,
}) => {
  const [inputValue, setInputValue] = useState('');
  const [isSpeechMode, setIsSpeechMode] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const [isListening, setIsListening] = useState(false);
  const sttPartialsRef = useRef<string>('');
  const unlistenPartRef = useRef<UnlistenFn | null>(null);
  const unlistenFinalRef = useRef<UnlistenFn | null>(null);

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 120)}px`;
    }
  }, [inputValue]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (inputValue.trim() && !isLoading) {
      onSendMessage(inputValue);
      setInputValue('');
      setIsSpeechMode(false);
      if (isListening) {
        await invoke('stt_stop').catch(() => {});
        try { unlistenPartRef.current && unlistenPartRef.current(); } catch {}
        try { unlistenFinalRef.current && unlistenFinalRef.current(); } catch {}
        unlistenPartRef.current = null;
        unlistenFinalRef.current = null;
        setIsListening(false);
      }
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e as any);
    }
  };

  const toggleListening = async () => {
    if (isListening) {
      try {
        await invoke('stt_stop');
      } catch (e) {
        console.error('stt_stop failed', e);
      }
      // cleanup event listeners
      try { unlistenPartRef.current && unlistenPartRef.current(); } catch {}
      try { unlistenFinalRef.current && unlistenFinalRef.current(); } catch {}
      unlistenPartRef.current = null;
      unlistenFinalRef.current = null;
      setIsListening(false);
      return;
    }

    // starting
    sttPartialsRef.current = '';
    setInputValue('');

    // subscribe to partial and final transcripts
    try {
      unlistenPartRef.current = await listen<string>('stt://partial', (e) => {
        const txt = (e.payload || '').trim();
        sttPartialsRef.current = txt;
        setInputValue(txt);
      });
      unlistenFinalRef.current = await listen<string>('stt://final', (e) => {
        const txt = (e.payload || '').trim();
        sttPartialsRef.current = '';
        setInputValue(txt);
      });
    } catch (e) {
      console.error('failed to subscribe to STT events', e);
    }

    try {
      await invoke('stt_start');
      setIsListening(true);
    } catch (e) {
      console.error('stt_start failed', e);
      setIsListening(false);
      alert('Unable to start speech recognition: ' + (e as Error).message);
    }
  };

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  const ExecutionSteps: React.FC<{ steps: ExecutionStep[] }> = ({ steps }) => {
    if (steps.length === 0) return null;

    return (
      <div className="execution-steps">
        <div style={{ fontWeight: 600, marginBottom: '8px' }}>Execution Steps:</div>
        {steps.map((step, index) => (
          <div key={index} className="step-item">
            <div className="step-method">Step {step.step}: {step.method_name}</div>
            <div className={`step-status ${step.success ? 'success' : 'error'}`}>
              {step.success ? '✓ Success' : `✗ Error: ${step.error || 'Unknown error'}`}
            </div>
          </div>
        ))}
      </div>
    );
  };

  const renderMicrophoneButton = () => {
    return (
      <button
        type="button"
        className={`mic-button ${isListening ? 'recording' : ''}`}
        onClick={toggleListening}
        disabled={isLoading}
        title={isListening ? 'Stop recording' : 'Start voice input'}
      >
        {isListening ? (
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="6" y="6" width="12" height="12" rx="2" ry="2"></rect>
          </svg>
        ) : (
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path>
            <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
            <line x1="12" y1="19" x2="12" y2="23"></line>
            <line x1="8" y1="23" x2="16" y2="23"></line>
          </svg>
        )}
      </button>
    );
  };

  return (
    <>
      <div className="messages">
        {error && (
          <div className="error-message">
            Error: {error}
          </div>
        )}

        {messages.map((message) => (
          <div key={message.id} className={`message ${message.type}`}>
            <div className="message-avatar">
              {message.type === 'user' ? 'U' : 'A'}
            </div>
            <div className="message-content">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {message.content}
              </ReactMarkdown>

              {message.steps && <ExecutionSteps steps={message.steps} />}

              {message.error && (
                <div className="step-status error">
                  Error: {message.error}
                </div>
              )}

              <div className="message-time">
                {formatTime(message.timestamp)}
              </div>
            </div>
          </div>
        ))}

        {isLoading && (
          <div className="message assistant">
            <div className="message-avatar">A</div>
            <div className="message-content">
              <div className="loading">
                <div className="loading-spinner"></div>
                Processing your request...
              </div>
            </div>
          </div>
        )}
      </div>

      <div className="input-container">
        {isListening && (
          <div className="speech-indicator">
            <span className="recording-dot"></span>
            Listening... Speak now
          </div>
        )}
        
        <form onSubmit={handleSubmit} className="input-wrapper">
          <textarea
            ref={textareaRef}
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={isSpeechMode ? "Listening... Speak into your microphone" : "Type your message here..."}
            className={`input-field ${isSpeechMode ? 'speech-mode' : ''}`}
            disabled={isLoading}
            rows={1}
          />
          
          <div className="input-buttons">
            {renderMicrophoneButton()}
            
            <button
              type="submit"
              className="send-button"
              disabled={!inputValue.trim() || isLoading}
            >
              Send
            </button>
          </div>
        </form>
      </div>
    </>
  );
};

export default ChatInterface;