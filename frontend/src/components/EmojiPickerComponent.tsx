import { useState } from 'react';
import EmojiPicker, { EmojiClickData, Theme } from 'emoji-picker-react';
import { Smile } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { useTheme } from '@/hooks/useTheme';

interface EmojiPickerComponentProps {
  onEmojiSelect: (emoji: string) => void;
  size?: 'sm' | 'md' | 'lg';
}

export function EmojiPickerComponent({ onEmojiSelect, size = 'md' }: EmojiPickerComponentProps) {
  const [isOpen, setIsOpen] = useState(false);
  const { theme } = useTheme();

  const handleEmojiClick = (emojiData: EmojiClickData) => {
    onEmojiSelect(emojiData.emoji);
    setIsOpen(false);
  };

  const iconSize = size === 'sm' ? 'h-3.5 w-3.5' : size === 'lg' ? 'h-5 w-5' : 'h-4 w-4';

  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="ghost"
          size={size === 'sm' ? 'sm' : 'default'}
          className={size === 'sm' ? 'h-7 w-7 p-0' : 'h-9 w-9 p-0'}
        >
          <Smile className={iconSize} />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-full p-0 border-0" align="end">
        <EmojiPicker
          onEmojiClick={handleEmojiClick}
          theme={theme === 'dark' ? Theme.DARK : Theme.LIGHT}
          width={350}
          height={400}
          searchPlaceHolder="Search emoji..."
          previewConfig={{ showPreview: false }}
        />
      </PopoverContent>
    </Popover>
  );
}
