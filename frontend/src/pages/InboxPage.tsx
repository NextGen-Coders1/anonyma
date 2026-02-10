import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { motion, AnimatePresence } from "framer-motion";
import { Inbox, EyeOff, ChevronDown, ChevronUp, Loader2 } from "lucide-react";
import { messages } from "@/lib/api";

const InboxPage = () => {
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const { data: messageList = [], isLoading } = useQuery({
    queryKey: ['inbox'],
    queryFn: messages.inbox,
  });

  const unreadCount = messageList.filter((m) => !m.is_read).length;

  return (
    <div>
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Inbox className="h-5 w-5 text-primary" />
          <h1 className="font-mono text-2xl font-bold text-foreground">Inbox</h1>
          {unreadCount > 0 && (
            <span className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-xs font-bold text-primary-foreground">
              {unreadCount}
            </span>
          )}
        </div>
        <p className="text-sm text-muted-foreground font-mono">
          Messages from unknown agents
        </p>
      </div>

      {isLoading ? (
        <div className="flex justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      ) : messageList.length === 0 ? (
        <div className="glass rounded-xl p-8 text-center">
          <p className="font-mono text-sm text-muted-foreground">Your inbox is empty. No messages yet.</p>
        </div>
      ) : (
        <div className="space-y-3">
          {messageList.map((msg, i) => (
            <motion.div
              key={msg.id}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: i * 0.05 }}
              className={`glass cursor-pointer rounded-xl p-5 transition-all duration-300 hover:neon-border-purple ${!msg.is_read ? "neon-border-cyan" : ""
                }`}
              onClick={() => setExpandedId(expandedId === msg.id ? null : msg.id)}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-muted">
                    <EyeOff className="h-4 w-4 text-primary" />
                  </div>
                  <div>
                    <h3 className="font-mono text-sm font-semibold text-foreground">
                      Anonymous Agent
                    </h3>
                    <span className="text-xs text-muted-foreground font-mono">
                      {new Date(msg.created_at).toLocaleString()}
                    </span>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {!msg.is_read && (
                    <div className="h-2 w-2 rounded-full bg-secondary animate-pulse-glow" />
                  )}
                  {expandedId === msg.id ? (
                    <ChevronUp className="h-4 w-4 text-muted-foreground" />
                  ) : (
                    <ChevronDown className="h-4 w-4 text-muted-foreground" />
                  )}
                </div>
              </div>

              <AnimatePresence>
                {expandedId === msg.id ? (
                  <motion.p
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: "auto", opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    className="mt-4 font-mono text-sm text-foreground/80 border-t border-border pt-4"
                  >
                    {msg.content}
                  </motion.p>
                ) : (
                  <p className="mt-3 truncate font-mono text-sm text-muted-foreground">
                    {msg.content}
                  </p>
                )}
              </AnimatePresence>
            </motion.div>
          ))}
        </div>
      )}
    </div>
  );
};

export default InboxPage;
