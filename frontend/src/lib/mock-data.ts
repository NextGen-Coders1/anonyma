export interface Agent {
  id: string;
  username: string;
  status: "online" | "offline" | "away";
  lastSeen?: string;
}

export interface Message {
  id: string;
  content: string;
  timestamp: string;
  read: boolean;
}

export interface Broadcast {
  id: string;
  content: string;
  timestamp: string;
  reactions: number;
}

export const mockAgents: Agent[] = [
  { id: "1", username: "ShadowWalker", status: "online" },
  { id: "2", username: "NightOwl_99", status: "online" },
  { id: "3", username: "CipherX", status: "away", lastSeen: "2 min ago" },
  { id: "4", username: "GhostNode", status: "online" },
  { id: "5", username: "DarkMatter", status: "offline", lastSeen: "1h ago" },
  { id: "6", username: "QuantumFlux", status: "online" },
  { id: "7", username: "BytePhantom", status: "away", lastSeen: "5 min ago" },
  { id: "8", username: "NeonDrift", status: "online" },
];

export const mockMessages: Message[] = [
  { id: "1", content: "I know what you did at the last hackathon...", timestamp: "2 min ago", read: false },
  { id: "2", content: "Check the encrypted file I left in the shared drive.", timestamp: "15 min ago", read: false },
  { id: "3", content: "The code review is done. No traces left.", timestamp: "1h ago", read: true },
  { id: "4", content: "Meet at the usual channel. Midnight.", timestamp: "3h ago", read: true },
  { id: "5", content: "Your last commit had a vulnerability. Fixed it for you.", timestamp: "5h ago", read: true },
];

export const mockBroadcasts: Broadcast[] = [
  { id: "1", content: "System maintenance tonight at 02:00 UTC. All shadows will be temporarily visible. 🌑", timestamp: "10 min ago", reactions: 12 },
  { id: "2", content: "New encryption protocol deployed. Your secrets are safer than ever.", timestamp: "1h ago", reactions: 24 },
  { id: "3", content: "Who else noticed the anomaly in sector 7? Something's not right...", timestamp: "2h ago", reactions: 8 },
  { id: "4", content: "Reminder: The first rule of Anonyma is you don't talk about Anonyma. 🤫", timestamp: "4h ago", reactions: 42 },
  { id: "5", content: "Just deployed a zero-day patch. You're welcome.", timestamp: "6h ago", reactions: 31 },
];
