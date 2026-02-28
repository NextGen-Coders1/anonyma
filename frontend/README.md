# Anonyma Frontend

Modern React-based user interface for the Anonyma anonymous messaging platform.

## Technology Stack

- Framework: Vite with React 18
- Language: TypeScript for type safety
- Styling: Tailwind CSS utility framework
- UI Components: shadcn/ui component library
- Icons: Lucide React icon set
- State Management: TanStack Query v5
- Animations: Framer Motion
- Emoji Picker: emoji-picker-react

## Getting Started

### Installation

Install project dependencies:

```bash
npm install
```

### Configuration

Create environment configuration file (optional):

```bash
cp .env.example .env
```

The default configuration connects to http://localhost:3000 for the backend API.

### Development

Start the development server:

```bash
npm run dev
```

The application will be available at http://localhost:8080

### Production Build

Create an optimized production build:

```bash
npm run build
```

Preview the production build:

```bash
npm run preview
```

## Project Structure

```
frontend/
├── src/
│   ├── components/          # React components
│   ├── hooks/              # Custom React hooks
│   ├── pages/              # Page components
│   ├── providers/          # Context providers
│   ├── lib/                # Utilities and API client
│   ├── App.tsx
│   └── main.tsx
├── public/                 # Static assets
└── package.json
```

## Development Tools

### Linting

Run ESLint to check code quality:

```bash
npm run lint
```

### Testing

Run the test suite with Vitest:

```bash
npm run test
```

## API Integration

The frontend communicates with the backend through a typed API client located in src/lib/api.ts. This provides type-safe request and response handling for all backend endpoints.

## License

This project is licensed under the MIT License.
