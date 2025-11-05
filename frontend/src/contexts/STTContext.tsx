import React, { createContext, useContext, useState, useEffect, useRef, ReactNode } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

interface STTContextType {
  isListening: boolean;
  waitingForPermission: boolean;
  partialTranscript: string;
  lastTranscript: string;
  startListening: () => Promise<void>;
  stopListening: () => Promise<void>;
  clearLastTranscript: () => void;
}

const STTContext = createContext<STTContextType | undefined>(undefined);

export const useSTT = () => {
  const context = useContext(STTContext);
  if (!context) {
    throw new Error('useSTT must be used within an STTProvider');
  }
  return context;
};

interface STTProviderProps {
  children: ReactNode;
}

export const STTProvider: React.FC<STTProviderProps> = ({ children }) => {
  const [isListening, setIsListening] = useState(false);
  const [waitingForPermission, setWaitingForPermission] = useState(false);
  const [partialTranscript, setPartialTranscript] = useState('');
  const [lastTranscript, setLastTranscript] = useState('');
  
  const unlistenPartRef = useRef<UnlistenFn | null>(null);
  const unlistenMicPermRef = useRef<UnlistenFn | null>(null);
  const unlistenSpeechAuthRef = useRef<UnlistenFn | null>(null);
  const startingRef = useRef(false);
  const shouldRetryRef = useRef(false);
  const currentTranscriptRef = useRef('');
  const silenceTimerRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    const setupListeners = async () => {
      try {
        // Listen for STT partial transcripts
        unlistenPartRef.current = await listen<string>('stt://partial', (e) => {
          const txt = (e.payload || '').trim();
          console.log('📝 STT Partial:', txt);
          const current = currentTranscriptRef.current;
          
          let newText = current;
          if (!current.includes(txt)) {
            newText = current ? `${current} ${txt}` : txt;
          }
          
          currentTranscriptRef.current = newText;
          setPartialTranscript(newText);
          setIsListening(true);
          
          // Clear any existing silence timer
          if (silenceTimerRef.current) {
            clearTimeout(silenceTimerRef.current);
          }
          
          // Check for sentence end markers or natural pause indicators
          const hasEndMarker = newText.includes('.') || newText.includes('?') || newText.includes('!') ||
                             newText.includes(' I want ') || newText.includes(' Show me ') ||
                             newText.includes(' I need ') || newText.includes(' Can you ');
          
          if (hasEndMarker) {
            setLastTranscript(newText);
            setPartialTranscript('');
            currentTranscriptRef.current = '';
          } else {
            // Set a timer to detect when speech has stopped (no more partials for 2 seconds)
            silenceTimerRef.current = setTimeout(() => {
              if (currentTranscriptRef.current.trim()) {
                setLastTranscript(currentTranscriptRef.current);
                setPartialTranscript('');
                currentTranscriptRef.current = '';
              }
            }, 2000);
          }
        });

        unlistenMicPermRef.current = await listen<string>('stt://mic-permission', (e) => {
          const status = (e.payload || '').toString();
          console.log('🎙️ Mic permission:', status);
          if (status === 'granted' && shouldRetryRef.current) {
            setWaitingForPermission(false);
            shouldRetryRef.current = false;
            setTimeout(() => startListening(), 500);
          } else if (status === 'denied') {
            setWaitingForPermission(false);
            shouldRetryRef.current = false;
          }
        });

        unlistenSpeechAuthRef.current = await listen<string>('stt://speech-auth', (e) => {
          // Demo
        });

      } catch (e) {
        console.error('Failed to set up STT event listeners:', e);
      }
    };

    setupListeners();

    // Cleanup function
    return () => {
      invoke('stt_stop').catch(() => { });
      unlistenPartRef.current?.();
      unlistenMicPermRef.current?.();
      unlistenSpeechAuthRef.current?.();
      if (silenceTimerRef.current) {
        clearTimeout(silenceTimerRef.current);
      }
    };
  }, []);

  const startListening = async () => {
    if (startingRef.current) return;
    startingRef.current = true;

    try {
      currentTranscriptRef.current = '';
      setPartialTranscript('');
      setLastTranscript('');
      
      try {
        await invoke('stt_start');
        setIsListening(true);
        setWaitingForPermission(false);
        shouldRetryRef.current = false;
      } catch (e) {
        const errorMsg = (e as Error).toString();
        console.error('stt_start failed:', errorMsg);
        if (errorMsg.includes('permission requested') || errorMsg.includes('authorization requested')) {
          setWaitingForPermission(true);
          shouldRetryRef.current = true;
        } else {
          setIsListening(false);
          setWaitingForPermission(false);
        }
      }
    } finally {
      startingRef.current = false;
    }
  };

  const stopListening = async () => {
    if (startingRef.current) return;
    startingRef.current = true;

    try {
      await invoke('stt_stop');
      setIsListening(false);
      setWaitingForPermission(false);
      shouldRetryRef.current = false;
    } finally {
      startingRef.current = false;
    }
  };

  const clearLastTranscript = () => {
    setLastTranscript('');
  };

  const value: STTContextType = {
    isListening,
    waitingForPermission,
    partialTranscript,
    lastTranscript,
    startListening,
    stopListening,
    clearLastTranscript,
  };

  return (
    <STTContext.Provider value={value}>
      {children}
    </STTContext.Provider>
  );
};