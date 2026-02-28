import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { motion, AnimatePresence } from "framer-motion";
import { Inbox, EyeOff, Loader2, Smile, MessageCircle, Trash2, Pin } from "lucide-react";
import { messages, conversations } from "@/lib/api";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import ConversationView from "@/components/ConversationView";
import { MessageSearch } from "@/components/MessageSearch";

const EMOJIS = ["🔥", "❤️", "😂", "😮", "😢", "👏"];

const InboxPage = () => {
  const [activeThread, setActiveThread] = useState<{ messageId: string; threadId: string; toUsername?: string } | null>(null);
  const queryClient = useQueryClient();

  // Inbox: messages received by the current user (where they are recipient)
  const { data: receivedMessages = [], isLoading: inboxLoading } = useQuery({
    queryKey: ['inbox'],
    queryFn: messages.inbox,
  });

  // Conversations: all threads where user participates (both sent and received)
  const { data: allConversations = [], isLoading: convLoading } = useQuery({
    queryKey: ['conversations'],
    queryFn: conversations.list,
  });

  const isLoading = inboxLoading || convLoading;

  // Use conversations list as the source of truth (it already deduplicates by thread)
  const allThreads = allConversations.sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );

  const reactMutation = useMutation({
    mutationFn: ({ messageId, emoji }: { messageId: string; emoji: string }) =>
      messages.react(messageId, emoji),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['inbox'] });
      queryClient.invalidateQueries({ queryKey: ['conversations'] });
    },
    onError: () => {
      toast.error("Failed to add reaction");
    },
  });

  const deleteThreadMutation = useMutation({
    mutationFn: (threadId: string) => conversations.deleteThread(threadId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['inbox'] });
      queryClient.invalidateQueries({ queryKey: ['conversations'] });
      toast.success("Thread deleted");
    },
    onError: () => {
      toast.error("Failed to delete thread");
    },
  });

  const pinThreadMutation = useMutation({
    mutationFn: (threadId: string) => conversations.pinThread(threadId),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['conversations'] });
      toast.success(data.pinned ? "Thread pinned" : "Thread unpinned");
    },
    onError: () => {
      toast.error("Failed to pin thread");
    },
  });

  // Calculate total unread count from received messages only
  const unreadCount = receivedMessages.filter((m) => !m.is_read).length;

  const openThread = (msg: typeof allThreads[0]) => {
    setActiveThread({ messageId: msg.id, threadId: msg.thread_id, toUsername: msg.to_username });
  };

  const handleDeleteThread = (e: React.MouseEvent, threadId: string) => {
    e.stopPropagation();
    if (window.confirm("Delete this entire conversation? This cannot be undone.")) {
      deleteThreadMutation.mutate(threadId);
    }
  };

  const handlePinThread = (e: React.MouseEvent, threadId: string) => {
    e.stopPropagation();
    pinThreadMutation.mutate(threadId);
  };

  const handleSearchSelect = (threadId: string, messageId: string) => {
    setActiveThread({ messageId, threadId, toUsername: undefined });
  };

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
        <p className="text-sm text-muted-foreground font-mono mb-4">
          Anonymous messages & conversations
        </p>
        
        {/* Search Bar */}
        <MessageSearch onSelectThread={handleSearchSelect} />
      </div>

      {isLoading ? (
        <div className="flex justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      ) : allThreads.length === 0 ? (
        <div className="glass rounded-xl p-8 text-center">
          <p className="font-mono text-sm text-muted-foreground">Your inbox is empty. No messages yet.</p>
        </div>
      ) : (
        <div className="space-y-3">
          {allThreads.map((msg, i) => (
            <motion.div
              key={msg.thread_id}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: i * 0.05 }}
              className={`glass cursor-pointer rounded-xl p-5 transition-all duration-300 hover:neon-border-purple ${!msg.is_read && !msg.is_mine ? "neon-border-cyan" : ""
                }`}
              onClick={() => openThread(msg)}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-muted">
                    <EyeOff className="h-4 w-4 text-primary" />
                  </div>
                  <div>
                    <h3 className="font-mono text-sm font-semibold text-foreground">
                      {msg.is_mine && msg.to_username ? `Chat with ${msg.to_username}` : "Anonymous Agent"}
                    </h3>
                    <span className="text-xs text-muted-foreground font-mono">
                      {new Date(msg.created_at).toLocaleString()}
                    </span>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {/* Pin button */}
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 w-7 p-0"
                    onClick={(e) => handlePinThread(e, msg.thread_id)}
                    title="Pin thread"
                  >
                    <Pin className="h-3.5 w-3.5" />
                  </Button>
                  
                  {/* Delete button */}
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 w-7 p-0 text-destructive hover:text-destructive"
                    onClick={(e) => handleDeleteThread(e, msg.thread_id)}
                    title="Delete thread"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                  
                  {/* Unread badge */}
                  {(msg.unread_count ?? 0) > 0 && (
                    <span className="flex h-5 min-w-5 items-center justify-center rounded-full bg-secondary px-1 text-[10px] font-bold text-background animate-pulse-glow">
                      +{msg.unread_count}
                    </span>
                  )}
                  {!msg.is_read && !msg.is_mine && (
                    <div className="h-2 w-2 rounded-full bg-secondary animate-pulse-glow" />
                  )}
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 gap-1 px-2 text-xs text-muted-foreground hover:text-primary"
                    onClick={(e) => {
                      e.stopPropagation();
                      openThread(msg);
                    }}
                  >
                    <MessageCircle className="h-3.5 w-3.5" />
                    Chat
                  </Button>
                </div>
              </div>

              {/* Message preview */}
              <p className="mt-3 truncate font-mono text-sm text-muted-foreground">
                {msg.content}
              </p>

              {/* Reactions (on the most recent seen message) */}
              <AnimatePresence>
                {msg.reactions && Object.keys(msg.reactions).length > 0 && (
                  <motion.div
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: "auto" }}
                    className="mt-3 flex flex-wrap items-center gap-2"
                    onClick={(e) => e.stopPropagation()}
                  >
                    {Object.entries(msg.reactions).map(([emoji, count]) => (
                      <div key={emoji} className="flex items-center gap-1 rounded-full bg-muted/50 px-2 py-0.5 text-xs">
                        <span>{emoji}</span>
                        <span className="font-bold text-primary">{count as number}</span>
                      </div>
                    ))}
                  </motion.div>
                )}
              </AnimatePresence>

              {/* Add Reaction row */}
              <div
                className="mt-2 flex items-center gap-1"
                onClick={(e) => e.stopPropagation()}
              >
                <Smile className="h-3.5 w-3.5 text-muted-foreground mr-1" />
                {EMOJIS.map((emoji) => (
                  <button
                    key={emoji}
                    onClick={(e) => {
                      e.stopPropagation();
                      reactMutation.mutate({ messageId: msg.id, emoji });
                    }}
                    className="hover:scale-125 transition-transform text-sm p-0.5"
                  >
                    {emoji}
                  </button>
                ))}
              </div>
            </motion.div>
          ))}
        </div>
      )}

      {/* Conversation / Chat overlay */}
      {activeThread && (
        <ConversationView
          messageId={activeThread.messageId}
          threadId={activeThread.threadId}
          toUsername={activeThread.toUsername}
          isOpen={!!activeThread}
          onClose={() => setActiveThread(null)}
        />
      )}
    </div>
  );
};

export default InboxPage;
