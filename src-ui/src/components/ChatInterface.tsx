import React, { useState, useRef, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { ChatMessage, ExecutionStep } from '../types';

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
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 120)}px`;
    }
  }, [inputValue]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (inputValue.trim() && !isLoading) {
      onSendMessage(inputValue);
      setInputValue('');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e as any);
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
            <div className="step-method">Step {step.step + 1}: {step.method_name}</div>
            <div className={`step-status ${step.success ? 'success' : 'error'}`}>
              {step.success ? '✓ Success' : `✗ Error: ${step.error || 'Unknown error'}`}
            </div>
          </div>
        ))}
      </div>
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
        <form onSubmit={handleSubmit} className="input-wrapper">
          <textarea
            ref={textareaRef}
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type your message here..."
            className="input-field"
            disabled={isLoading}
            rows={1}
          />
          <button
            type="submit"
            className="send-button"
            disabled={!inputValue.trim() || isLoading}
          >
            Send
          </button>
        </form>
      </div>
    </>
  );
};

export default ChatInterface;