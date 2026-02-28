import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { preferences } from '@/lib/api';

type Theme = 'light' | 'dark';

interface ThemeContextType {
  theme: Theme;
  toggleTheme: () => void;
  isLoading: boolean;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<Theme>('dark'); // Default to dark to avoid flash
  const [isLoading, setIsLoading] = useState(true);

  // Apply theme immediately on mount
  useEffect(() => {
    const getSystemPreference = (): Theme => {
      if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
        return 'dark';
      }
      return 'light';
    };

    // Load theme from preferences
    const loadTheme = async () => {
      try {
        const prefs = await preferences.get();
        const savedTheme = prefs.theme as Theme;
        if (savedTheme) {
          setTheme(savedTheme);
          applyTheme(savedTheme);
        } else {
          // No saved preference, use system preference
          const systemPreference = getSystemPreference();
          setTheme(systemPreference);
          applyTheme(systemPreference);
        }
      } catch (error) {
        // Fallback to localStorage or system preference
        const localTheme = localStorage.getItem('theme') as Theme | null;
        if (localTheme) {
          setTheme(localTheme);
          applyTheme(localTheme);
        } else {
          // Use system preference
          const systemPreference = getSystemPreference();
          setTheme(systemPreference);
          applyTheme(systemPreference);
        }
      } finally {
        setIsLoading(false);
      }
    };

    loadTheme();

    // Listen for system theme changes only if no saved preference
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (e: MediaQueryListEvent) => {
      // Only auto-switch if user hasn't manually set a preference
      const hasManualPreference = localStorage.getItem('theme');
      if (!hasManualPreference) {
        const newTheme = e.matches ? 'dark' : 'light';
        setTheme(newTheme);
        applyTheme(newTheme);
      }
    };

    mediaQuery.addEventListener('change', handleChange);

    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }, []);

  const applyTheme = (newTheme: Theme) => {
    const root = document.documentElement;
    if (newTheme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  };

  const toggleTheme = async () => {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    applyTheme(newTheme);
    
    // Save to backend
    try {
      await preferences.update({ theme: newTheme });
    } catch (error) {
      console.error('Failed to save theme preference:', error);
    }
    
    // Fallback to localStorage
    localStorage.setItem('theme', newTheme);
  };

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme, isLoading }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}
