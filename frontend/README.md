# Anonyma Frontend

A sleek interface for the Anonyma network.

## Tech Stack

- **Framework**: Vite and React
- **Language**: TypeScript
- **Styling**: Tailwind CSS
- **UI Components**: shadcn/ui
- **Icons**: Lucide React
- **Data Fetching**: TanStack Query v5
- **Animations**: Framer Motion

## Getting Started

1. **Install Dependencies**:
   ```bash
   npm install
   ```

2. **Configuration**:
   Ensure VITE_API_URL is set in your .env (defaults to http://localhost:3000).

3. **Development Mode**:
   ```bash
   npm run dev
   ```

## Project Structure

- src/components/ui/: Atomic UI components from shadcn.
- src/lib/api.ts: Typed API client for interacting with the Rust backend.
- src/pages/: Main application views (Inbox, Broadcast, Settings).
- src/providers/: React Context providers (Auth, Theme).

## Development Tools

- **Linting**: npm run lint
- **Build**: npm run build
- **Testing**: npm run test (Vitest)

## UI Features

- **Glassmorphism**: High-quality frosted glass effects throughout the UI.
- **Neon Accents**: Dynamic neon borders and glows that react to user interaction.
- **Responsive Design**: Optimized for both desktop and mobile agents.
