# Frontend Framework Selection: SvelteKit

> **Decision:** SvelteKit + TypeScript
> **Date:** 2026-01-07

---

## Summary

SvelteKit selected for mobile-first performance, built-in features, and optimal fit for a read-heavy dashboard with selective client-side interactivity.

---

## Key Factors

### 1. Mobile-First Bundle Size

| Framework | Base Runtime | Typical App (gzipped) |
|-----------|-------------|----------------------|
| React 18 | ~40KB | 120-180KB |
| SvelteKit | ~5KB | 30-60KB |

Smaller bundles mean faster load times, less JS parsing, and reduced battery consumption on mobile devices.

### 2. Batteries Included

SvelteKit provides out-of-box what React requires assembling:

| Feature | SvelteKit | React Equivalent |
|---------|-----------|------------------|
| Routing | Built-in | react-router |
| SSR | Built-in | Next.js or manual |
| Transitions | Built-in | framer-motion |
| State management | Stores | Redux/Zustand/Jotai |

Fewer dependencies = smaller bundle + less maintenance.

### 3. Server-Rendered Dashboard

Read-heavy dashboard benefits from server rendering:

- HTML arrives ready to display
- Minimal JavaScript execution on device
- Works well on slow connections
- Lower memory pressure

SvelteKit's `load` functions make this the default path.

### 4. Client-Side Where Needed

Conversation panel with AI streaming remains client-side:

```
┌─────────────────────────────────────┐
│  Dashboard Shell (server-rendered)  │
│  ┌─────────────┬─────────────────┐  │
│  │ Cycle Tree  │  Pugh Matrix    │  │
│  │ (server)    │  (server)       │  │
│  ├─────────────┴─────────────────┤  │
│  │  Conversation Panel            │  │
│  │  (client-side streaming)       │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

### 5. UI Requirements Are Simple

**Pugh Matrix (<10x10 cells):**
- CSS Grid sufficient
- No virtualization needed
- No heavy table library required

**Cycle Tree:**
- Custom SVG with built-in Svelte transitions
- No external visualization library needed

---

## Voice Input

Web Speech API (browser-native) for initial implementation:
- Purely client-side
- Zero additional cost
- Conversation panel is already client-side

Whisper API fallback can be added later via backend proxy if quality issues arise.

---

## Trade-off Accepted

Smaller ecosystem than React accepted in exchange for:
- Significantly smaller bundles
- Built-in features reducing dependency count
- Simpler mental model for server/client split

---

## Project Structure

```
frontend/
├── src/
│   ├── lib/
│   │   ├── domain/           # TypeScript types (mirrors backend)
│   │   ├── components/       # Shared UI components
│   │   └── stores/           # Svelte stores for client state
│   ├── routes/
│   │   ├── +layout.svelte
│   │   ├── +page.svelte      # Home/session list
│   │   ├── sessions/
│   │   │   ├── [id]/
│   │   │   │   ├── +page.svelte      # Dashboard
│   │   │   │   └── +page.server.ts   # Server load
│   │   │   └── ...
│   │   └── ...
│   └── app.html
├── static/
├── svelte.config.js
├── tsconfig.json
└── package.json
```

---

## Key Libraries

| Purpose | Library | Notes |
|---------|---------|-------|
| HTTP client | `fetch` | Built-in, works server & client |
| Forms | SvelteKit actions | Built-in |
| Validation | `zod` | Schema validation |
| Styling | `tailwindcss` | Utility-first CSS |
| Icons | `lucide-svelte` | Lightweight icon set |
