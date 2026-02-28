import { useState } from 'react';
import { MoreVertical, Edit, Trash2, Pin, Copy } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

interface MessageActionsProps {
  messageId: string;
  content: string;
  isMine: boolean;
  isPinned?: boolean;
  onEdit?: (messageId: string, content: string) => void;
  onDelete?: (messageId: string) => void;
  onPin?: (messageId: string) => void;
}

export function MessageActions({
  messageId,
  content,
  isMine,
  isPinned = false,
  onEdit,
  onDelete,
  onPin,
}: MessageActionsProps) {
  const [isOpen, setIsOpen] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(content);
    setIsOpen(false);
  };

  const handleEdit = () => {
    if (onEdit) {
      onEdit(messageId, content);
    }
    setIsOpen(false);
  };

  const handleDelete = () => {
    if (onDelete && window.confirm('Delete this message?')) {
      onDelete(messageId);
    }
    setIsOpen(false);
  };

  const handlePin = () => {
    if (onPin) {
      onPin(messageId);
    }
    setIsOpen(false);
  };

  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="h-6 w-6 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
        >
          <MoreVertical className="h-3.5 w-3.5" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-48">
        <DropdownMenuItem onClick={handleCopy}>
          <Copy className="mr-2 h-4 w-4" />
          Copy text
        </DropdownMenuItem>
        
        {onPin && (
          <DropdownMenuItem onClick={handlePin}>
            <Pin className="mr-2 h-4 w-4" />
            {isPinned ? 'Unpin message' : 'Pin message'}
          </DropdownMenuItem>
        )}
        
        {isMine && onEdit && (
          <DropdownMenuItem onClick={handleEdit}>
            <Edit className="mr-2 h-4 w-4" />
            Edit message
          </DropdownMenuItem>
        )}
        
        {isMine && onDelete && (
          <DropdownMenuItem onClick={handleDelete} className="text-destructive">
            <Trash2 className="mr-2 h-4 w-4" />
            Delete message
          </DropdownMenuItem>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
