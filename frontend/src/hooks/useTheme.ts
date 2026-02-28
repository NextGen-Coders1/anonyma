import { useEffect, useState } from 'react';
import { preferences } from '@/lib/api';

export function useTheme() {
  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    // Check system preference first
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      return 'dark';
    }
    return 'light';
  });
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Load theme from preferences
    const loadTheme = async () => {
      try {
        const prefs = await preferences.get();
        const savedTheme = prefs.theme as 'light' | 'dark';
        if (savedTheme) {
          setTheme(savedTheme);
          applyTheme(savedTheme);
        } else {
          // No saved preference, use system preference
          const systemPreference = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
          setTheme(systemPreference);
          applyTheme(systemPreference);
        }
      } catch (error) {
        // Fallback to localStorage or system preference
        const localTheme = localStorage.getItem('theme') as 'light' | 'dark' | null;
        if (localTheme) {
          setTheme(localTheme);
          applyTheme(localTheme);
        } else {
          // Use system preference
          const systemPreference = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
          setTheme(systemPreference);
          applyTheme(systemPreference);
        }
      } finally {
        setIsLoading(false);
      }
    };

    loadTheme();

    // Listen for system theme changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (e: MediaQueryListEvent) => {
      const newTheme = e.matches ? 'dark' : 'light';
      setTheme(newTheme);
      applyTheme(newTheme);
    };

    mediaQuery.addEventListener('change', handleChange);

    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }, []);

  const applyTheme = (newTheme: 'light' | 'dark') => {
    if (newTheme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
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

  return { theme, toggleTheme, isLoading };
}
