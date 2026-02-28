import { Moon, Sun } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useTheme } from '@/providers/ThemeProvider';

export function ThemeToggle() {
  const { theme, toggleTheme, isLoading } = useTheme();

  if (isLoading) {
    return null;
  }

  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={toggleTheme}
      className="theme-toggle h-9 w-9 p-0"
      title={`Switch to ${theme === 'dark' ? 'light' : 'dark'} mode`}
    >
      {theme === 'dark' ? (
        <Sun className="h-4 w-4 text-yellow-500" />
      ) : (
        <Moon className="h-4 w-4 text-blue-500" />
      )}
    </Button>
  );
}
