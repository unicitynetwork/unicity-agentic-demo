import { type LucideIcon } from "lucide-react";

export interface IAgent {
  id: string;
  name: string;
  Icon: LucideIcon;
}

export interface IAsset {
  id: string;
  name: string;
  ticker: string;
  iconUrl: string;
  amount: number;
}

export interface UIComponentSuggestion {
  component_type: string;
  title: string;
  description: string;
  priority: number;
}

export interface QueryResult {
  success: boolean;
  response: string;
  steps: ExecutionStep[];
  ui_suggestions: UIComponentSuggestion[];
  error?: string;
}

export interface ExecutionStep {
  step: number;
  method_name: string;
  input: any;
  output: any;
  success: boolean;
  error?: string;
}

export interface BalanceInfo {
  asset_id: string;
  balance: string;
  raw_balance: number;
}

export interface AgentInfo {
  id: string;
  label: string;
  description: string;
  methods: string[];
}

export interface ChatMessage {
  id: string;
  type: 'user' | 'assistant';
  content: string;
  timestamp: Date;
  steps?: ExecutionStep[];
  error?: string;
}

export interface AppState {
  balances: BalanceInfo[];
  agents: AgentInfo[];
  messages: ChatMessage[];
  isLoading: boolean;
  error?: string;
}