import { motion } from "framer-motion";
import { Send, Wifi, WifiOff, Clock } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { Agent } from "@/lib/mock-data";

interface AgentCardProps {
  agent: Agent;
  onSendMessage: (agent: Agent) => void;
  index: number;
}

const statusConfig = {
  online: { icon: Wifi, label: "Online", className: "text-secondary" },
  away: { icon: Clock, label: "Away", className: "text-primary" },
  offline: { icon: WifiOff, label: "Offline", className: "text-muted-foreground" },
};

const AgentCard = ({ agent, onSendMessage, index }: AgentCardProps) => {
  const status = statusConfig[agent.status];
  const StatusIcon = status.icon;

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.05, duration: 0.3 }}
      className="glass group rounded-xl p-5 transition-all duration-300 hover:neon-border-purple"
    >
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted font-mono text-sm font-bold text-primary">
            {agent.username.slice(0, 2).toUpperCase()}
          </div>
          <div>
            <h3 className="font-mono text-sm font-semibold text-foreground">
              {agent.username}
            </h3>
            <div className={`flex items-center gap-1.5 text-xs ${status.className}`}>
              <StatusIcon className="h-3 w-3" />
              <span className="font-mono">{status.label}</span>
              {agent.lastSeen && (
                <span className="text-muted-foreground">· {agent.lastSeen}</span>
              )}
            </div>
          </div>
        </div>
      </div>

      <Button
        variant="glass"
        size="sm"
        className="w-full opacity-70 transition-opacity group-hover:opacity-100"
        onClick={() => onSendMessage(agent)}
        disabled={agent.status === "offline"}
      >
        <Send className="mr-2 h-3.5 w-3.5" />
        Send Anonymously
      </Button>
    </motion.div>
  );
};

export default AgentCard;
