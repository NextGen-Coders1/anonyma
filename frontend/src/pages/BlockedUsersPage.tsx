import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { motion } from 'framer-motion';
import { ShieldOff, Loader2, UserX } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { users } from '@/lib/api';
import { toast } from 'sonner';

const BlockedUsersPage = () => {
  const queryClient = useQueryClient();

  const { data: blockedUsers = [], isLoading } = useQuery({
    queryKey: ['blocked-users'],
    queryFn: users.getBlocked,
  });

  const unblockMutation = useMutation({
    mutationFn: (userId: string) => users.unblock(userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['blocked-users'] });
      toast.success('User unblocked');
    },
    onError: () => {
      toast.error('Failed to unblock user');
    },
  });

  return (
    <div>
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <ShieldOff className="h-5 w-5 text-destructive" />
          <h1 className="font-mono text-2xl font-bold text-foreground">Blocked Users</h1>
        </div>
        <p className="text-sm text-muted-foreground font-mono">
          Manage users you've blocked from sending you messages
        </p>
      </div>

      {isLoading ? (
        <div className="flex justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      ) : blockedUsers.length === 0 ? (
        <div className="glass rounded-xl p-12 text-center">
          <UserX className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <p className="font-mono text-sm text-muted-foreground">
            You haven't blocked any users yet.
          </p>
          <p className="font-mono text-xs text-muted-foreground mt-2">
            Blocked users cannot send you messages or see your activity.
          </p>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {blockedUsers.map((user, index) => (
            <motion.div
              key={user.id}
              initial={{ opacity: 0, y: 15 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.05 }}
              className="glass rounded-xl p-5 transition-all duration-300 hover:neon-border-red"
            >
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-destructive/10 font-mono text-sm font-bold text-destructive">
                    {user.username[0].toUpperCase()}
                  </div>
                  <div>
                    <h3 className="font-mono text-sm font-semibold text-foreground">
                      {user.username}
                    </h3>
                    {user.bio && (
                      <p className="font-mono text-xs text-muted-foreground line-clamp-1">
                        {user.bio}
                      </p>
                    )}
                  </div>
                </div>
              </div>

              <Button
                variant="outline"
                size="sm"
                className="w-full"
                onClick={() => {
                  if (window.confirm(`Unblock ${user.username}?`)) {
                    unblockMutation.mutate(user.id);
                  }
                }}
                disabled={unblockMutation.isPending}
              >
                {unblockMutation.isPending ? (
                  <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                ) : (
                  <ShieldOff className="mr-2 h-3.5 w-3.5" />
                )}
                Unblock User
              </Button>
            </motion.div>
          ))}
        </div>
      )}
    </div>
  );
};

export default BlockedUsersPage;
