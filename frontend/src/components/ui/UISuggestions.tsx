import { UIComponentSuggestion } from '../../types';
import { Button } from '../common/Button';
import { MessageSquare, Wallet, Users, History, Mic, Settings } from 'lucide-react';

interface UISuggestionsProps {
  suggestions: UIComponentSuggestion[];
  onSuggestionClick: (suggestion: UIComponentSuggestion) => void;
}

const getIcon = (componentType: string) => {
  switch (componentType) {
    case 'chat_screen':
      return MessageSquare;
    case 'crypto_balances':
      return Wallet;
    case 'agent_list':
      return Users;
    case 'transaction_history':
      return History;
    case 'voice_interface':
      return Mic;
    case 'settings_panel':
      return Settings;
    default:
      return MessageSquare;
  }
};

const getPriorityColor = (priority: number) => {
  if (priority >= 8) return 'from-[#C5FC48] to-[#8ED818]'; // High priority - green
  if (priority >= 5) return 'from-[#FFD700] to-[#FFA500]'; // Medium priority - gold
  return 'from-[#87CEEB] to-[#4682B4]'; // Low priority - blue
};

export const UISuggestions: React.FC<UISuggestionsProps> = ({
  suggestions,
  onSuggestionClick
}) => {
  if (suggestions.length === 0) return null;

  // Sort by priority (highest first)
  const sortedSuggestions = [...suggestions].sort((a, b) => b.priority - a.priority);

  return (
    <div className="p-4 bg-[#1A1A1A] border border-[#2A2A2A] rounded-xl">
      <h3 className="text-lg font-semibold text-white mb-3">Suggested Components</h3>
      <div className="space-y-2">
        {sortedSuggestions.map((suggestion, index) => {
          const Icon = getIcon(suggestion.component_type);
          const gradientClass = getPriorityColor(suggestion.priority);
          
          return (
            <div
              key={index}
              className="flex items-center gap-3 p-3 bg-[#0F0F0F] border border-[#2A2A2A] rounded-lg hover:border-[#3A3A3A] transition-colors"
            >
              <div className={`
                p-2 rounded-lg bg-gradient-to-r ${gradientClass}
              `}>
                <Icon className="w-4 h-4 text-black" />
              </div>
              
              <div className="flex-1">
                <h4 className="text-white font-medium">{suggestion.title}</h4>
                <p className="text-gray-400 text-sm">{suggestion.description}</p>
              </div>
              
              <Button
                variant="icon"
                size="sm"
                onClick={() => onSuggestionClick(suggestion)}
                className="bg-gradient-to-r from-[#C5FC48] to-[#8ED818] hover:from-[#8ED818] hover:to-[#C5FC48]"
              >
                Add
              </Button>
            </div>
          );
        })}
      </div>
    </div>
  );
};