import { Mic, MicOff } from 'lucide-react';
import { Button } from '../common/Button';
import { useState, useEffect } from 'react';
import { useSTT } from '../../contexts/STTContext';

interface BlankStateProps {
  onTranscript: (text: string) => void;
}

export const BlankState: React.FC<BlankStateProps> = ({
  onTranscript
}) => {
  const { isListening, waitingForPermission, partialTranscript, lastTranscript, startListening, stopListening, clearLastTranscript } = useSTT();

  const handleToggleListening = async () => {
    if (isListening || waitingForPermission) {
      await stopListening();
    } else {
      await startListening();
    }
  };

  // Handle last transcript (when speech recognition completes)
  useEffect(() => {
    if (lastTranscript) {
      // Call the onTranscript prop with the completed transcript
      onTranscript(lastTranscript);
      clearLastTranscript();
    }
  }, [lastTranscript, onTranscript]);

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#0A0A0A] to-[#1A1A1A] flex items-center justify-center">
      <div className="text-center space-y-8 max-w-2xl mx-auto px-6">
        {/* Title */}
        <div className="space-y-4">
          <h1 className="text-4xl font-light text-white/90">
            Welcome to Unicity
          </h1>
          <p className="text-lg text-white/60">
            I'm here to help. Just speak to me naturally.
          </p>
        </div>

        {/* Voice Indicator */}
        <div className="flex flex-col items-center space-y-6">
          <div className={`
            relative w-32 h-32 rounded-full flex items-center justify-center
            transition-all duration-300
            ${isListening 
              ? 'bg-gradient-to-r from-[#C5FC48] to-[#8ED818] animate-pulse' 
              : 'bg-[#2A2A2A]'
            }
          `}>
            {isListening ? (
              <Mic className="w-12 h-12 text-black" />
            ) : (
              <MicOff className="w-12 h-12 text-white/40" />
            )}
            
            {/* Animated rings when listening */}
            {isListening && (
              <>
                <div className="absolute inset-0 rounded-full border-2 border-[#C5FC48] animate-ping" />
                  <div className="absolute inset-0 rounded-full border-2 border-[#8ED818] animate-ping animation-delay-200" />
              </>
            )}
          </div>

          {/* Status Text */}
          <div className="space-y-2">
            <p className="text-white/80 font-medium">
              {isListening ? 'Listening...' : 'Click to start speaking'}
            </p>
            
            {/* Partial Transcript */}
            {partialTranscript && (
              <div className="bg-[#2A2A2A] rounded-lg px-4 py-2 max-w-md">
                <p className="text-white/90 text-sm italic">"{partialTranscript}"</p>
              </div>
            )}
          </div>
        </div>

        {/* Control Button */}
        <Button
          onClick={handleToggleListening}
          className={`
            px-8 py-4 text-lg font-medium
            ${isListening || waitingForPermission
              ? 'bg-red-500 hover:bg-red-600 text-white'
              : 'bg-gradient-to-r from-[#C5FC48] to-[#8ED818] text-black hover:from-[#8ED818] hover:to-[#C5FC48]'
            }
          `}
        >
          {isListening || waitingForPermission ? 'Stop Listening' : 'Start Speaking'}
        </Button>

        {/* Example Commands */}
        <div className="space-y-3 text-left">
          <p className="text-white/60 text-sm">Try saying:</p>
          <div className="space-y-2 text-white/40 text-sm">
            <p>• "I want to have a chat screen to write to you"</p>
            <p>• "Show me my crypto balances"</p>
            <p>• "I want to see a list of available agents"</p>
            <p>• "Swap 100 USDT to BTC"</p>
          </div>
        </div>
      </div>
    </div>
  );
};