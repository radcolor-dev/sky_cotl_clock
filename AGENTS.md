# AGENTS.md

Behavioral guidelines to reduce common LLM coding mistakes.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" -> "Write tests for invalid inputs, then make them pass"
- "Fix the bug" -> "Write a test that reproduces it, then make it pass"
- "Refactor X" -> "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] -> verify: [check]
2. [Step] -> verify: [check]
3. [Step] -> verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## Project

Isekai is a Tauri 2 desktop app for Sky: Children of the Light, built with React, TypeScript, Vite, Rust, Tailwind CSS, and shadcn/ui.

## Local Skills

Always use the project-installed shadcn skill before any shadcn/ui or component work:

`./.agents/skills/shadcn/SKILL.md`

Follow its linked rules in `./.agents/skills/shadcn/rules/`, especially composition, styling, forms, and icons. Prefer existing shadcn components in `src-ui/components/ui` before writing custom UI.

## Commands

- Install dependencies: `bun install`
- Run tests: `bun run test`
- Build app: `bun run build`
- Run desktop app: `bun tauri dev`
- Check Rust backend: `cd src-rs && cargo check`

## Structure

- `src-ui/` contains the React app.
- `src-ui/components/ui/` contains local shadcn/ui components.
- `src-ui/pages.tsx` contains the main page components.
- `src-rs/` contains the Tauri/Rust app.
- `docs/` contains the static website.

## Rust vs TypeScript

Over time we want more logic in Rust (`src-rs/`) and less in TypeScript (`src-ui/`). Rust stays in `src-rs/`; `src-ui/` remains TypeScript/React for the UI. This is not about building UI in Rust.

Before writing code, decide where it belongs:

- **Core logic** always lives in Rust.
- **Heavy and medium logic** should land in Rust when practical; expose it to the UI through Tauri commands or events instead of reimplementing it in TypeScript.
- **TypeScript** is for the UI layer: components, layout, user input, and thin glue that calls into Rust.

This is not a rule to always pick Rust. Use judgment — if something is clearly view-only or trivial frontend wiring, keep it in TypeScript.

## UI Rules

- Use shadcn primitives such as `Button`, `Card`, `Tabs`, `Dialog`, `Input`, `Select`, `Switch`, `Badge`, `Separator`, and `Tooltip` when available.
- Keep `TabsTrigger` inside `TabsList`.
- Use full `Card` composition: `CardHeader`, `CardTitle`, `CardDescription`, and `CardContent`.
- Use semantic Tailwind tokens like `bg-background`, `bg-card`, `text-foreground`, `text-muted-foreground`, `border-border`, and `text-primary`.
- Use lucide icons consistently with the existing app.
- Avoid unrelated refactors and preserve user changes.

## Verification

After code changes, run the narrowest useful check. For UI or TypeScript changes, prefer `bun run build`.
