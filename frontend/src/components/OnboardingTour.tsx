import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, ArrowRight, ArrowLeft, Check } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface TourStep {
  title: string;
  description: string;
  target?: string;
  position?: 'top' | 'bottom' | 'left' | 'right';
}

const tourSteps: TourStep[] = [
  {
    title: 'Welcome to Anonyma',
    description: 'A secure platform for anonymous communication. Let us show you around!',
  },
  {
    title: 'Inbox',
    description: 'View all your anonymous messages here. Click on any conversation to reply.',
    target: '[href="/dashboard/inbox"]',
    position: 'right',
  },
  {
    title: 'Users',
    description: 'Discover other users and send them anonymous messages. They will never know who you are.',
    target: '[href="/dashboard/users"]',
    position: 'right',
  },
  {
    title: 'Broadcasts',
    description: 'Post public messages visible to everyone. You can choose to post anonymously or with your username.',
    target: '[href="/dashboard/broadcasts"]',
    position: 'right',
  },
  {
    title: 'Settings',
    description: 'Customize your profile, manage notifications, and adjust your preferences.',
    target: '[href="/dashboard/settings"]',
    position: 'right',
  },
  {
    title: 'Theme Toggle',
    description: 'Switch between dark and light modes. The app adapts to your system preferences by default.',
    target: '.theme-toggle',
    position: 'top',
  },
  {
    title: 'You\'re All Set!',
    description: 'Start exploring Anonyma. Send anonymous messages, post broadcasts, and connect with others securely.',
  },
];

export function OnboardingTour() {
  const [isOpen, setIsOpen] = useState(false);
  const [currentStep, setCurrentStep] = useState(0);
  const [highlightedElement, setHighlightedElement] = useState<HTMLElement | null>(null);

  useEffect(() => {
    // Check if user has seen the tour
    const hasSeenTour = localStorage.getItem('anonyma_tour_completed');
    if (!hasSeenTour) {
      // Wait a bit before showing the tour
      const timeout = setTimeout(() => {
        setIsOpen(true);
      }, 1000);
      return () => clearTimeout(timeout);
    }
  }, []);

  useEffect(() => {
    if (!isOpen) return;

    const step = tourSteps[currentStep];
    if (step.target) {
      const element = document.querySelector(step.target) as HTMLElement;
      if (element) {
        setHighlightedElement(element);
        element.scrollIntoView({ behavior: 'smooth', block: 'center' });
        // Bring element to front
        element.style.position = 'relative';
        element.style.zIndex = '102';
      }
    } else {
      setHighlightedElement(null);
    }

    return () => {
      // Reset z-index when moving to next step
      if (highlightedElement) {
        highlightedElement.style.zIndex = '';
        highlightedElement.style.position = '';
      }
    };
  }, [currentStep, isOpen, highlightedElement]);

  const handleNext = () => {
    if (currentStep < tourSteps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      handleComplete();
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSkip = () => {
    localStorage.setItem('anonyma_tour_completed', 'true');
    setIsOpen(false);
  };

  const handleComplete = () => {
    localStorage.setItem('anonyma_tour_completed', 'true');
    setIsOpen(false);
  };

  const getTooltipPosition = () => {
    if (!highlightedElement) return { top: '50%', left: '50%', transform: 'translate(-50%, -50%)' };

    const rect = highlightedElement.getBoundingClientRect();
    const step = tourSteps[currentStep];
    const offset = 20;

    switch (step.position) {
      case 'right':
        return {
          top: `${rect.top + rect.height / 2}px`,
          left: `${rect.right + offset}px`,
          transform: 'translateY(-50%)',
        };
      case 'left':
        return {
          top: `${rect.top + rect.height / 2}px`,
          right: `${window.innerWidth - rect.left + offset}px`,
          transform: 'translateY(-50%)',
        };
      case 'top':
        return {
          bottom: `${window.innerHeight - rect.top + offset}px`,
          left: `${rect.left + rect.width / 2}px`,
          transform: 'translateX(-50%)',
        };
      case 'bottom':
        return {
          top: `${rect.bottom + offset}px`,
          left: `${rect.left + rect.width / 2}px`,
          transform: 'translateX(-50%)',
        };
      default:
        return { top: '50%', left: '50%', transform: 'translate(-50%, -50%)' };
    }
  };

  const step = tourSteps[currentStep];

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* Overlay */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-[100] bg-background/80 backdrop-blur-sm"
            onClick={handleSkip}
          />

          {/* Highlight with cutout effect */}
          {highlightedElement && (
            <>
              {/* Create a cutout effect using clip-path */}
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-[101] pointer-events-none"
                style={{
                  background: 'rgba(0, 0, 0, 0.7)',
                  clipPath: `polygon(
                    0% 0%, 
                    0% 100%, 
                    ${highlightedElement.getBoundingClientRect().left - 8}px 100%, 
                    ${highlightedElement.getBoundingClientRect().left - 8}px ${highlightedElement.getBoundingClientRect().top - 8}px,
                    ${highlightedElement.getBoundingClientRect().right + 8}px ${highlightedElement.getBoundingClientRect().top - 8}px,
                    ${highlightedElement.getBoundingClientRect().right + 8}px ${highlightedElement.getBoundingClientRect().bottom + 8}px,
                    ${highlightedElement.getBoundingClientRect().left - 8}px ${highlightedElement.getBoundingClientRect().bottom + 8}px,
                    ${highlightedElement.getBoundingClientRect().left - 8}px 100%,
                    100% 100%,
                    100% 0%
                  )`,
                }}
              />
              
              {/* Highlight border */}
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed z-[102] pointer-events-none rounded-lg"
                style={{
                  top: highlightedElement.getBoundingClientRect().top - 4,
                  left: highlightedElement.getBoundingClientRect().left - 4,
                  width: highlightedElement.getBoundingClientRect().width + 8,
                  height: highlightedElement.getBoundingClientRect().height + 8,
                  border: '3px solid hsl(var(--primary))',
                  boxShadow: '0 0 20px hsl(var(--primary))',
                }}
              />
            </>
          )}

          {/* Tooltip */}
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.9 }}
            className="fixed z-[103] w-full max-w-md"
            style={getTooltipPosition()}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="glass-strong rounded-xl p-6 shadow-2xl">
              {/* Header */}
              <div className="flex items-start justify-between mb-4">
                <div className="flex-1">
                  <h3 className="font-mono text-lg font-bold text-foreground mb-1">
                    {step.title}
                  </h3>
                  <p className="text-sm text-muted-foreground font-mono">
                    Step {currentStep + 1} of {tourSteps.length}
                  </p>
                </div>
                <button
                  onClick={handleSkip}
                  className="rounded-lg p-2 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
                >
                  <X className="h-4 w-4" />
                </button>
              </div>

              {/* Content */}
              <p className="text-sm text-foreground mb-6 leading-relaxed">
                {step.description}
              </p>

              {/* Progress */}
              <div className="flex gap-1 mb-6">
                {tourSteps.map((_, index) => (
                  <div
                    key={index}
                    className={`h-1 flex-1 rounded-full transition-colors ${
                      index <= currentStep ? 'bg-primary' : 'bg-muted'
                    }`}
                  />
                ))}
              </div>

              {/* Actions */}
              <div className="flex items-center justify-between">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handlePrevious}
                  disabled={currentStep === 0}
                  className="font-mono"
                >
                  <ArrowLeft className="h-4 w-4 mr-2" />
                  Previous
                </Button>

                {currentStep === tourSteps.length - 1 ? (
                  <Button
                    variant="default"
                    size="sm"
                    onClick={handleComplete}
                    className="font-mono"
                  >
                    <Check className="h-4 w-4 mr-2" />
                    Get Started
                  </Button>
                ) : (
                  <Button
                    variant="default"
                    size="sm"
                    onClick={handleNext}
                    className="font-mono"
                  >
                    Next
                    <ArrowRight className="h-4 w-4 ml-2" />
                  </Button>
                )}
              </div>

              {/* Skip button */}
              <div className="mt-4 text-center">
                <button
                  onClick={handleSkip}
                  className="text-xs text-muted-foreground hover:text-foreground transition-colors font-mono"
                >
                  Skip tour
                </button>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
