export interface QueryResult {
  success: boolean;
  response: string;
  steps: ExecutionStep[];
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