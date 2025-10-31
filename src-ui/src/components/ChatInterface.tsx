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
  const [waitingForPermission, setWaitingForPermission] = useState(false);
  const sttPartialsRef = useRef<string>('');
  const unlistenPartRef = useRef<UnlistenFn | null>(null);
  const unlistenFinalRef = useRef<UnlistenFn | null>(null);
  const unlistenDebugRef = useRef<UnlistenFn | null>(null);
  const unlistenMicPermRef = useRef<UnlistenFn | null>(null);
  const unlistenSpeechAuthRef = useRef<UnlistenFn | null>(null);
  const startingRef = useRef(false);
  const shouldRetryRef = useRef(false);

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 120)}px`;
    }
  }, [inputValue]);

  // Setup event listeners once on mount
  useEffect(() => {
    const setupListeners = async () => {
      try {
        // Partial results
        if (!unlistenPartRef.current) {
          unlistenPartRef.current = await listen<string>('stt://partial', (e) => {
            const txt = (e.payload || '').trim();
            console.log('📝 Partial:', txt);

            // Accumulate text
            const current = sttPartialsRef.current;
            const newText = current ? `${current} ${txt}` : txt;

            sttPartialsRef.current = newText;
            setInputValue(newText);
            if (!isListening) setIsListening(true);
          });
        }

        // Final results
        if (!unlistenFinalRef.current) {
          unlistenFinalRef.current = await listen<string>('stt://final', (e) => {
            const txt = (e.payload || '').trim();
            console.log('✅ Final:', txt);

            // Handle error messages
            if (txt.startsWith('[error]')) {
              const errorMsg = txt.replace('[error]', '').trim();
              console.error('Speech recognition error:', errorMsg);

              let userMessage = errorMsg;
              if (errorMsg.includes('not authorized')) {
                userMessage = 'Speech recognition permission needed. Please grant permission in System Settings > Privacy & Security > Speech Recognition, then try again.';
              } else if (errorMsg.includes('authorization requested')) {
                userMessage = 'Please grant speech recognition permission when prompted, then click the microphone button again.';
              }

              alert(userMessage);
              setIsListening(false);
              setIsSpeechMode(false);
              return;
            }

            sttPartialsRef.current = '';
            setInputValue(txt);
          });
        }

        // Debug messages
        if (!unlistenDebugRef.current) {
          unlistenDebugRef.current = await listen<string>('stt://debug', (e) => {
            const msg = (e.payload || '').toString();
            console.debug('[stt debug]', msg);
          });
        }

        // NEW: Listen for microphone permission changes
        if (!unlistenMicPermRef.current) {
          unlistenMicPermRef.current = await listen<string>('stt://mic-permission', (e) => {
            const status = (e.payload || '').toString();
            console.log('🎙️ Mic permission:', status);

            if (status === 'granted' && shouldRetryRef.current) {
              console.log('🔄 Auto-retrying after mic permission granted...');
              shouldRetryRef.current = false;
              setWaitingForPermission(false);
              // Retry after a short delay
              setTimeout(() => {
                toggleListening();
              }, 500);
            } else if (status === 'denied') {
              setWaitingForPermission(false);
              shouldRetryRef.current = false;
              alert('Microphone access was denied. Please enable it in System Settings > Privacy & Security > Microphone.');
            }
          });
        }

        // NEW: Listen for speech authorization changes
        if (!unlistenSpeechAuthRef.current) {
          unlistenSpeechAuthRef.current = await listen<string>('stt://speech-auth', (e) => {
            const status = (e.payload || '').toString();
            console.log('🗣️ Speech auth:', status);

            if (status === 'Authorized' && shouldRetryRef.current) {
              console.log('🔄 Auto-retrying after speech auth granted...');
              shouldRetryRef.current = false;
              setWaitingForPermission(false);
              // Retry after a short delay
              setTimeout(() => {
                toggleListening();
              }, 500);
            } else if (status === 'Denied') {
              setWaitingForPermission(false);
              shouldRetryRef.current = false;
              alert('Speech recognition was denied. Please enable it in System Settings > Privacy & Security > Speech Recognition.');
            }
          });
        }
      } catch (e) {
        console.error('Failed to set up STT event listeners:', e);
      }
    };

    setupListeners();

    // Cleanup on unmount
    return () => {
      // Always try to stop, even if we think we're not listening
      invoke('stt_stop').catch(() => {});
      try { unlistenPartRef.current?.(); } catch {}
      try { unlistenFinalRef.current?.(); } catch {}
      try { unlistenDebugRef.current?.(); } catch {}
      try { unlistenMicPermRef.current?.(); } catch {}
      try { unlistenSpeechAuthRef.current?.(); } catch {}
    };
  }, []); // Only run once on mount

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (inputValue.trim() && !isLoading) {
      onSendMessage(inputValue);
      setInputValue('');
      setIsSpeechMode(false);
      if (isListening || waitingForPermission) {
        await invoke('stt_stop').catch(() => {});
        setIsListening(false);
        setWaitingForPermission(false);
        shouldRetryRef.current = false;
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
    // Prevent multiple simultaneous calls
    if (startingRef.current) {
      console.log('⏸️ Already starting/stopping, ignoring click');
      return;
    }

    startingRef.current = true;
    console.log('🎤 Toggle listening, current state:', isListening);

    try {
      if (isListening || waitingForPermission) {
        // Stopping
        console.log('🛑 Stopping speech recognition...');
        await invoke('stt_stop');
        setIsListening(false);
        setIsSpeechMode(false);
        setWaitingForPermission(false);
        shouldRetryRef.current = false;
        console.log('✅ Stopped successfully');
      } else {
        // Starting
        console.log('▶️ Starting speech recognition...');
        sttPartialsRef.current = '';
        setInputValue('');
        setIsSpeechMode(true);

        try {
          await invoke('stt_start');
          // Success!
          setIsListening(true);
          setWaitingForPermission(false);
          shouldRetryRef.current = false;

          // Re-focus the textarea so user can see text appearing
          textareaRef.current?.focus();

          console.log('✅ Started successfully');
        } catch (e) {
          const errorMessage = (e as Error).toString();
          console.error('❌ stt_start failed:', errorMessage);

          // Check if this is a permission request (not a real error)
          if (errorMessage.includes('permission requested') ||
            errorMessage.includes('authorization requested') ||
            errorMessage.includes('Please try again after granting permission')) {
            // This is expected - permission dialog is showing
            console.log('ℹ️ Permission requested, waiting for grant...');
            setWaitingForPermission(true); // Keep button pressed!
            shouldRetryRef.current = true; // Enable auto-retry
            // Don't reset isListening or isSpeechMode - keep them as they are
            // Don't show alert - just wait for permission
          } else {
            // Real error
            let userMessage = 'Unable to start speech recognition: ' + errorMessage;

            if (errorMessage.includes('not authorized')) {
              userMessage = 'Speech recognition permission required. Please enable in System Settings > Privacy & Security > Speech Recognition.';
            } else if (errorMessage.includes('denied')) {
              userMessage = 'Microphone or speech recognition access was denied. Please enable in System Settings > Privacy & Security.';
            } else if (errorMessage.includes('unavailable')) {
              userMessage = 'Speech recognition is not available on this device.';
            }

            alert(userMessage);
            setWaitingForPermission(false);
            setIsListening(false);
            setIsSpeechMode(false);
            shouldRetryRef.current = false;
          }
        }
      }
    } finally {
      // CRITICAL: Always reset startingRef
      startingRef.current = false;
      console.log('🔓 Button unlocked');
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
    const isPressed = isListening || waitingForPermission;
    const buttonTitle = isPressed
      ? (waitingForPermission ? 'Waiting for permission...' : 'Stop recording')
      : 'Start voice input';

    return (
      <button
        type="button"
        className={`mic-button ${isPressed ? 'recording' : ''}`}
        onClick={toggleListening}
        disabled={isLoading || startingRef.current}
        aria-pressed={isPressed}
        title={buttonTitle}
      >
        {isPressed ? (
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
        {(isListening || waitingForPermission) && (
          <div className="speech-indicator">
            <span className="recording-dot"></span>
            {waitingForPermission ? 'Waiting for permission...' : 'Listening... Speak now'}
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