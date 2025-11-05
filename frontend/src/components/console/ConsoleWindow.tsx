import { ChevronRight, ArrowUp, Loader2, StopCircle, Mic } from 'lucide-react';
import { Button } from '../common/Button';
import { useEffect, useRef, useState } from 'react';
import { ChatMessage } from '../../types';
import { useSTT } from '../../contexts/STTContext';

interface ConsoleWindowProps {
  messages: ChatMessage[];
  isLoading: boolean;
  error?: string;
  onSendMessage: (message: string) => void;
}

export const ConsoleWindow: React.FC<ConsoleWindowProps> = ({
  messages,
  isLoading,
  error,
  onSendMessage
}) => {

  const [inputValue, setInputValue] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const {
    isListening,
    waitingForPermission,
    partialTranscript,
    lastTranscript,
    startListening,
    stopListening,
    clearLastTranscript
  } = useSTT();

  // Update input value when partial transcript changes
  useEffect(() => {
    if (partialTranscript) {
      setInputValue(partialTranscript);
    }
  }, [partialTranscript]);

  // Handle last transcript (when speech recognition completes)
  useEffect(() => {
    if (lastTranscript && !isLoading) {
      // Automatically send the completed transcript to the LLM
      onSendMessage(lastTranscript);
      // Clear the input field after sending
      setInputValue('');
      // Stop listening after sending the message
      stopListening();
      // Clear the last transcript to prevent re-sending
      clearLastTranscript();
    }
  }, [lastTranscript, onSendMessage, isLoading, stopListening, setInputValue, clearLastTranscript]);

  const toggleListening = async () => {
    if (isListening || waitingForPermission) {
      await stopListening();
    } else {
      setInputValue('');
      await startListening();
    }
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (inputValue.trim() && !isLoading) {
      onSendMessage(inputValue);
      setInputValue(''); // Clear the input field
      if (isListening || waitingForPermission) {
        await stopListening();
      }
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

  return (
    <div className="p-3 bg-[#101010] border-[1.5px] border-[#1D1D1D] rounded-xl flex flex-col overflow-hidden">
      <div className="p-3 flex-1 space-y-2 overflow-y-auto h-40">
        {messages.map(msg => (
          <div
            key={msg.id}
            className={`flex gap-3 ${msg.type === 'user' ? 'justify-end' : ''}`}
          >
            {msg.type === 'assistant' && (
              <div className="
                w-8 h-8 rounded-full bg-brand-green-dark 
                text-brand-green font-bold text-sm
                shrink-0 flex items-center justify-center
              ">
                A
              </div>
            )}
            <div className={`
              max-w-[80%] p-3 rounded-lg
              ${msg.type === 'user'
                ? 'bg-brand-green text-brand-text-dark'
                : 'bg-brand-bg-dark text-brand-text-light'
              }
            `}>
              {/* TODO: render Markdown */}
              <p className="text-sm whitespace-pre-wrap">{msg.content}</p>

              {/* TODO: Render ExecutionSteps */}

              {msg.error && (
                <p className="mt-2 text-xs text-red-400">Error: {msg.error}</p>
              )}

              <p className={`mt-2 text-xs ${msg.type === 'user' ? 'text-brand-text-dark/70' : 'text-brand-text-dim'}`}>
                {formatTime(msg.timestamp)}
              </p>
            </div>
            {msg.type === 'user' && (
              <div className="
                w-8 h-8 rounded-full bg-brand-bg-dark 
                text-brand-text-light font-bold text-sm
                shrink-0 flex items-center justify-center
              ">
                U
              </div>
            )}
          </div>
        ))}
        {isLoading && (
          <div className="flex gap-3">
            <div className="w-8 h-8 rounded-full bg-brand-green-dark text-brand-green font-bold text-sm shrink-0 flex items-center justify-center">A</div>
            <div className="max-w-[80%] p-3 rounded-lg bg-brand-bg-dark text-brand-text-light">
              <Loader2 className="w-5 h-5 animate-spin text-brand-text-dim" />
            </div>
          </div>
        )}

        {error && (
          <p className="text-sm text-red-500">App Error: {error}</p>
        )}
        <div ref={messagesEndRef} />
      </div>

      <form
        onSubmit={handleSubmit}
        className="flex items-center gap-3 bg-[#0B0B0B] rounded-xl pl-4 py-1 pr-1"
      >
        <ChevronRight className="w-5 h-5 text-brand-text-dim" />
        <input
          type="text"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={isListening ? "Listening..." : "Enter a prompt here"}
          className="
            flex-1 bg-transparent
            text-brand-text-light
            placeholder:text-brand-text-dim
            focus:outline-none
          "
          disabled={isLoading}
        />
        <button
          type="button"
          onClick={toggleListening}
          disabled={isLoading}
          className={`
            p-2 rounded-lg transition-colors cursor-pointer
            ${isListening || waitingForPermission
              ? 'text-red-500 bg-red-500/10'
              : 'text-brand-text-dim hover:bg-brand-bg-dark hover:text-brand-text-light'
            }
          `}
        >
          {isListening || waitingForPermission ? (
            <StopCircle className="w-5 h-5" />
          ) : (
            <Mic className="w-5 h-5" />
          )}
        </button>
        <Button variant="icon" type="submit" className=" cursor-pointer bg-linear-to-r from-[#C5FC48] to-[#8ED818]" disabled={!inputValue.trim() || isLoading}>
          <ArrowUp className='w-4 h-4 text-[#121212] stroke-3' />
        </Button>
      </form>
    </div>
  );
};