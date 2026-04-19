# web/

Next.js 15 web application. Proprietary.

**Status:** Not started. Will be scaffolded after the API has an alpha surface.

## Planned stack

- **Framework:** Next.js 15 (App Router, React 19, Server Components).
- **Styling:** Tailwind CSS 4 + a small in-house design-system package.
- **State:** TanStack Query for server state; Zustand for ephemeral UI state.
- **Graph editor:** ReactFlow for the model canvas.
- **Real-time:** Yjs over the Arko sync protocol (see `specs/sync/`).
- **Deploy:** Self-hosted container on Hetzner behind Traefik, not Vercel.

## Design language

- **Calm, not flashy.** This is a tool engineers live in 8 hours a day.
- **Keyboard-first.** Every action has a shortcut.
- **Density over whitespace** in data-heavy views. Marketing pages get the air.
- **No dark-pattern billing.** Trial → paid transitions are explicit and reversible.

## Non-goals

- No mobile-first. LCA is desktop work. Mobile is read-only, phase 6+.
- No "AI everywhere." AI is a tool, not a vibe. It appears where it earns
  its place — summarization, flow suggestion, data gap detection — and
  nowhere else.
