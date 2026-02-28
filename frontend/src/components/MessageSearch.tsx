import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Search, X, Loader2, MessageCircle } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { messages } from '@/lib/api';
import { motion, AnimatePresence } from 'framer-motion';

interface MessageSearchProps {
  onSelectThread: (threadId: string, messageId: string) => void;
}

export function MessageSearch({ onSelectThread }: MessageSearchProps) {
  const [query, setQuery] = useState('');
  const [isOpen, setIsOpen] = useState(false);

  const { data: results = [], isLoading } = useQuery({
    queryKey: ['search', query],
    queryFn: () => messages.search(query),
    enabled: query.length >= 2,
  });

  const handleClear = () => {
    setQuery('');
    setIsOpen(false);
  };

  return (
    <div className="relative">
      <div className="relative">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          type="text"
          placeholder="Search messages..."
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setIsOpen(true);
          }}
          onFocus={() => setIsOpen(true)}
          className="pl-10 pr-10 bg-muted/50 border-border"
        />
        {query && (
          <button
            onClick={handleClear}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </div>

      <AnimatePresence>
        {isOpen && query.length >= 2 && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="absolute top-full mt-2 w-full glass rounded-xl shadow-lg z-50 max-h-96 overflow-y-auto"
          >
            {isLoading ? (
              <div className="flex items-center justify-center p-8">
                <Loader2 className="h-6 w-6 animate-spin text-primary" />
              </div>
            ) : results.length === 0 ? (
              <div className="p-8 text-center">
                <p className="text-sm text-muted-foreground font-mono">
                  No messages found for "{query}"
                </p>
              </div>
            ) : (
              <div className="p-2">
                {results.map((msg) => (
                  <button
                    key={msg.id}
                    onClick={() => {
                      onSelectThread(msg.thread_id, msg.id);
                      handleClear();
                    }}
                    className="w-full text-left p-3 rounded-lg hover:bg-muted/50 transition-colors"
                  >
                    <div className="flex items-start gap-3">
                      <MessageCircle className="h-4 w-4 text-primary mt-1 flex-shrink-0" />
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-mono text-foreground truncate">
                          {msg.content}
                        </p>
                        <p className="text-xs text-muted-foreground font-mono mt-1">
                          {new Date(msg.created_at).toLocaleString()}
                        </p>
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
