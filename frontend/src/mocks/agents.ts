import { Pickaxe, ShieldCheck, Zap } from "lucide-react";
import type { IAgent } from "../types";

export const publicAgents: IAgent[] = [
  { id: '1', name: 'OTC Swap Deck', Icon: ShieldCheck },
  { id: '2', name: 'Mining', Icon: Pickaxe },
  { id: '3', name: 'Quake', Icon: Zap },
  { id: '4', name: 'P2P Marketplace', Icon: Zap },
  { id: '5', name: 'Curate', Icon: Zap },
]

export const privateAgents: IAgent[] = [
  { id: '6', name: 'DeepFake Detector', Icon: ShieldCheck },
  { id: '7', name: 'Background Check', Icon: Pickaxe },
]