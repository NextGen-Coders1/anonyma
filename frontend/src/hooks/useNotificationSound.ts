import { useEffect, useState, useRef } from 'react';
import { preferences } from '@/lib/api';

export function useNotificationSound() {
  const [enabled, setEnabled] = useState(true);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  useEffect(() => {
    // Load preference
    const loadPreference = async () => {
      try {
        const prefs = await preferences.get();
        setEnabled(prefs.notification_sound);
      } catch (error) {
        // Fallback to localStorage
        const saved = localStorage.getItem('notification_sound');
        setEnabled(saved !== 'false');
      }
    };

    loadPreference();

    // Preload audio
    audioRef.current = new Audio('/sounds/notification.mp3');
    audioRef.current.volume = 0.5;
  }, []);

  const play = () => {
    if (enabled && audioRef.current) {
      // Reset to beginning
      audioRef.current.currentTime = 0;
      
      // Play with error handling
      const playPromise = audioRef.current.play();
      
      if (playPromise !== undefined) {
        playPromise
          .then(() => {
            console.log('Notification sound played successfully');
          })
          .catch(err => {
            console.error('Failed to play notification sound:', err);
            // Try to reload and play again
            if (audioRef.current) {
              audioRef.current.load();
              audioRef.current.play().catch(e => console.error('Retry failed:', e));
            }
          });
      }
    }
  };

  const toggle = async () => {
    const newValue = !enabled;
    setEnabled(newValue);
    
    try {
      await preferences.update({ notification_sound: newValue });
    } catch (error) {
      console.error('Failed to save notification preference:', error);
    }
    
    localStorage.setItem('notification_sound', String(newValue));
  };

  return { enabled, play, toggle };
}
