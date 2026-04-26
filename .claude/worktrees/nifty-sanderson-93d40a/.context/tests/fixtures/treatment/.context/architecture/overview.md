# Architecture Overview

## Layer Structure

```
┌─────────────────────────────────────┐
│            Routes                    │
│   HTTP endpoint handlers             │
│   src/routes/                        │
└────────────────┬────────────────────┘
                 │
┌────────────────▼────────────────────┐
│           Services                   │
│   Business logic                     │
│   src/services/                      │
└────────────────┬────────────────────┘
                 │
┌────────────────▼────────────────────┐
│          Repositories                │
│   Data access layer                  │
│   src/repositories/                  │
└─────────────────────────────────────┘
```

## Directory Structure

```
src/
├── routes/          # HTTP handlers
├── services/        # Business logic
├── repositories/    # Data access
├── middleware/      # Request processing
├── schemas/         # Zod validation schemas
├── types/           # TypeScript interfaces
└── errors/          # Error definitions
```

## Request Flow

1. Request hits route
2. Middleware processes (auth, validation)
3. Handler calls service
4. Service calls repository
5. Response returned
