import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Users, Loader2, Send, ShieldOff } from "lucide-react";
import { users } from "@/lib/api";
import { Button } from "@/components/ui/button";
import SendMessageModal from "@/components/SendMessageModal";
import { motion } from "framer-motion";
import { toast } from "sonner";

const UsersPage = () => {
  const [selectedUserId, setSelectedUserId] = useState<string | null>(null);
  const [selectedUsername, setSelectedUsername] = useState<string>("");
  const queryClient = useQueryClient();

  const { data: usersList = [], isLoading } = useQuery({
    queryKey: ['users'],
    queryFn: users.list,
  });

  const blockMutation = useMutation({
    mutationFn: (userId: string) => users.block(userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
      toast.success('User blocked');
    },
    onError: () => {
      toast.error('Failed to block user');
    },
  });

  const handleSendMessage = (userId: string, username: string) => {
    setSelectedUserId(userId);
    setSelectedUsername(username);
  };

  return (
    <div>
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Users className="h-5 w-5 text-primary" />
          <h1 className="font-mono text-2xl font-bold text-foreground">Active Agents</h1>
        </div>
        <p className="text-sm text-muted-foreground font-mono">
          <span className="text-secondary">{usersList.length}</span> agents currently in the shadows
        </p>
      </div>

      {isLoading ? (
        <div className="flex justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      ) : usersList.length === 0 ? (
        <div className="glass rounded-xl p-8 text-center">
          <p className="font-mono text-sm text-muted-foreground">No agents found.</p>
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {usersList.map((user, i) => (
            <motion.div
              key={user.id}
              initial={{ opacity: 0, y: 15 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: i * 0.05 }}
              className="glass rounded-xl p-5 transition-all duration-300 hover:neon-border-purple"
            >
              <div className="mb-4 flex items-center gap-3">
                <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10 font-mono text-lg font-bold text-primary">
                  {user.username[0].toUpperCase()}
                </div>
                <div>
                  <h3 className="font-mono text-sm font-bold text-foreground">{user.username}</h3>
                  <p className="text-xs text-muted-foreground font-mono">{user.provider}</p>
                </div>
              </div>
              <Button
                variant="outline"
                size="sm"
                className="w-full mb-2"
                onClick={() => handleSendMessage(user.id, user.username)}
              >
                <Send className="mr-2 h-3.5 w-3.5" />
                Send Message
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="w-full text-destructive hover:text-destructive"
                onClick={() => {
                  if (window.confirm(`Block ${user.username}? They won't be able to send you messages.`)) {
                    blockMutation.mutate(user.id);
                  }
                }}
                disabled={blockMutation.isPending}
              >
                <ShieldOff className="mr-2 h-3.5 w-3.5" />
                Block User
              </Button>
            </motion.div>
          ))}
        </div>
      )}

      <SendMessageModal
        isOpen={!!selectedUserId}
        onClose={() => {
          setSelectedUserId(null);
          setSelectedUsername("");
        }}
        recipientId={selectedUserId || ""}
        recipientUsername={selectedUsername}
      />
    </div>
  );
};

export default UsersPage;
