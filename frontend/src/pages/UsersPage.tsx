import { useState } from "react";
import { Users } from "lucide-react";
import { mockAgents } from "@/lib/mock-data";
import AgentCard from "@/components/AgentCard";
import SendMessageModal from "@/components/SendMessageModal";
import type { Agent } from "@/lib/mock-data";

const UsersPage = () => {
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);

  const onlineCount = mockAgents.filter((a) => a.status === "online").length;

  return (
    <div>
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Users className="h-5 w-5 text-primary" />
          <h1 className="font-mono text-2xl font-bold text-foreground">Active Agents</h1>
        </div>
        <p className="text-sm text-muted-foreground font-mono">
          <span className="text-secondary">{onlineCount}</span> agents currently in the shadows
        </p>
      </div>

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {mockAgents.map((agent, i) => (
          <AgentCard
            key={agent.id}
            agent={agent}
            onSendMessage={setSelectedAgent}
            index={i}
          />
        ))}
      </div>

      <SendMessageModal
        isOpen={!!selectedAgent}
        onClose={() => setSelectedAgent(null)}
        recipientUsername={selectedAgent?.username ?? ""}
      />
    </div>
  );
};

export default UsersPage;
