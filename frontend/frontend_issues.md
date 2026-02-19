# StelloVault — Frontend: GitHub Issues

> These issues track building missing pages and features for the Next.js frontend (`/frontend`).
> All paths are relative to `frontend/src/app/`. Pages with ✅ already exist; all others are new.
>
> **Existing pages:** `/ (home)` ✅ · `/dashboard` ✅ · `/escrows` ✅ · `/collateral` ✅ · `/profile` ✅

---

## Issue #F1: [MARKETING] Build About Page

**Labels:** `frontend`, `marketing`, `priority: medium`

**Path:** `frontend/src/app/about/page.tsx`

**Description:**
Create a public-facing About page that communicates StelloVault's mission, the trade finance gap it addresses, and the core team.

**Sections to implement:**
- **Hero** — headline + subheadline summarizing the mission
- **Problem Statement** — trade finance gap ($100–120B), SME impact, AfCFTA opportunity
- **How It Works** — 3-step visual flow: Tokenize → Escrow → Finance
- **Technology Stack** — Stellar/Soroban, Account Abstraction, Oracle integration diagram
- **Team / Contributors** — grid of contributor cards (name, role, GitHub/Twitter links)
- **Roadmap** — timeline of past milestones and upcoming deliverables

**Tasks:**
- [ ] `frontend/src/app/about/page.tsx` — page component with all sections
- [ ] `frontend/src/components/marketing/HowItWorks.tsx` — 3-step visual
- [ ] `frontend/src/components/marketing/TeamCard.tsx` — contributor card
- [ ] `frontend/src/components/marketing/Roadmap.tsx` — timeline component
- [ ] Add `/about` link to global `Navbar` component

**Acceptance Criteria:**
- Page is fully responsive (mobile, tablet, desktop)
- Server-side rendered (no `"use client"` on page root)
- Lighthouse accessibility score ≥ 90

---

## Issue #F2: [MARKETING] Build Contact Page

**Labels:** `frontend`, `marketing`, `priority: medium`

**Path:** `frontend/src/app/contact/page.tsx`

**Description:**
Create a Contact page for users to reach the team, report issues, or submit partnership inquiries.

**Sections to implement:**
- **Contact Form** — name, email, subject (dropdown: General / Partnership / Bug Report / Oracle Registration), message, submit
- **Social Links** — GitHub, Twitter/X, Discord, Telegram
- **FAQ Accordion** — 5–8 common questions (How do I list collateral? What fees apply? etc.)

**Tasks:**
- [ ] `frontend/src/app/contact/page.tsx`
- [ ] `frontend/src/components/marketing/ContactForm.tsx` — controlled form with client-side validation
- [ ] `frontend/src/components/marketing/FaqAccordion.tsx` — expandable Q&A component
- [ ] API action: `POST /api/contact` (or mailto fallback for v1)
- [ ] Add `/contact` link to global `Navbar` and footer

**Acceptance Criteria:**
- Form validates required fields client-side (non-empty, valid email format)
- Success/error toast displayed after submission
- Form resets on successful submission

---

## Issue #F3: [AUTH] Build Login / Connect Wallet Page

**Labels:** `frontend`, `auth`, `priority: critical`

**Path:** `frontend/src/app/login/page.tsx`

**Description:**
The entry point for non-custodial wallet authentication. Supports Freighter wallet (browser extension) and WalletConnect as the primary connect methods.

**Flow:**
1. User clicks "Connect Wallet" → picker modal (Freighter / WalletConnect)
2. On connect, client calls `POST /api/v1/auth/challenge` with `walletAddress`
3. Client signs returned nonce with Freighter `signMessage()`
4. Client calls `POST /api/v1/auth/verify` → receives `{ accessToken, refreshToken }`
5. Tokens stored in `httpOnly` cookie (via server action) or `localStorage`
6. Redirect to `/dashboard`

**Tasks:**
- [ ] `frontend/src/app/login/page.tsx`
- [ ] `frontend/src/components/auth/WalletPickerModal.tsx` — Freighter + WalletConnect options
- [ ] `frontend/src/components/auth/ConnectButton.tsx` — reusable connect/disconnect button for Navbar
- [ ] `frontend/src/hooks/useWalletAuth.ts` — encapsulates challenge → sign → verify flow
- [ ] `frontend/src/lib/auth.ts` — token storage and refresh logic
- [ ] Protect all `/dashboard/*` routes via middleware (`frontend/src/middleware.ts`)
- [ ] Redirect authenticated users away from `/login`

**Acceptance Criteria:**
- Freighter not installed → shows install link
- Expired session → silently refreshes token or redirects to `/login`
- Auth state persists across page reloads

---

## Issue #F4: [LOANS] Build Loans Pages

**Labels:** `frontend`, `loans`, `priority: high`

**Paths:**
- `frontend/src/app/loans/page.tsx` — loan list
- `frontend/src/app/loans/[id]/page.tsx` — loan detail
- `frontend/src/app/loans/new/page.tsx` — new loan form

**Description:**
UI for the loan lifecycle: browsing available loans, viewing individual loan details with repayment schedule, and issuing a new loan.

**Tasks:**
- [ ] `frontend/src/app/loans/page.tsx` — filterable table (status: PENDING / ACTIVE / REPAID / DEFAULTED)
- [ ] `frontend/src/app/loans/[id]/page.tsx` — loan card, collateral info, repayment progress bar, repay button
- [ ] `frontend/src/app/loans/new/page.tsx` — multi-step form: select collateral → loan terms → review → sign XDR
- [ ] `frontend/src/components/loans/LoanCard.tsx`
- [ ] `frontend/src/components/loans/RepaymentSchedule.tsx` — table + progress bar
- [ ] `frontend/src/components/loans/LoanForm.tsx` — controlled multi-step form
- [ ] `frontend/src/hooks/useLoans.ts` — data fetching from `GET /api/v1/loans`
- [ ] XDR signing step: display transaction XDR → call Freighter `signTransaction()` → submit

**Acceptance Criteria:**
- `POST /api/v1/loans` on form submission returns XDR; Freighter signs it
- Repayment progress bar reflects `Repayment[]` records
- Loan list supports status filter and pagination

---

## Issue #F5: [GOVERNANCE] Build Governance Pages

**Labels:** `frontend`, `governance`, `priority: medium`

**Paths:**
- `frontend/src/app/governance/page.tsx` — proposal list
- `frontend/src/app/governance/[id]/page.tsx` — proposal detail + voting
- `frontend/src/app/governance/new/page.tsx` — create proposal form

**Description:**
On-chain DAO governance UI. Users browse proposals, vote, and submit new proposals. Vote weight is proportional to on-chain stake.

**Tasks:**
- [ ] `frontend/src/app/governance/page.tsx` — proposal list with status badges (OPEN / PASSED / REJECTED / EXECUTED)
- [ ] `frontend/src/app/governance/[id]/page.tsx` — proposal details, vote tally bar, cast vote buttons (For / Against / Abstain)
- [ ] `frontend/src/app/governance/new/page.tsx` — proposal creation form (title, description, type, duration)
- [ ] `frontend/src/components/governance/ProposalCard.tsx`
- [ ] `frontend/src/components/governance/VoteTallyBar.tsx` — for/against/abstain visual bar
- [ ] `frontend/src/components/governance/VoteButton.tsx` — triggers XDR sign flow
- [ ] `frontend/src/hooks/useGovernance.ts`

**Acceptance Criteria:**
- Proposals show time remaining (countdown) for OPEN proposals
- Duplicate vote attempt shows `409` error toast
- Voting requires connected wallet

---

## Issue #F6: [RISK] Build Risk Score Pages

**Labels:** `frontend`, `risk`, `priority: medium`

**Paths:**
- `frontend/src/app/risk/page.tsx` — lookup form
- `frontend/src/app/risk/[wallet]/page.tsx` — score breakdown + history

**Description:**
Allow any user to look up a Stellar wallet's risk score and view a historical trend chart.

**Tasks:**
- [ ] `frontend/src/app/risk/page.tsx` — wallet address input + lookup button
- [ ] `frontend/src/app/risk/[wallet]/page.tsx` — score gauge, component breakdown, historical line chart
- [ ] `frontend/src/components/risk/ScoreGauge.tsx` — circular progress gauge (0–1000, colored by grade A–F)
- [ ] `frontend/src/components/risk/ScoreBreakdown.tsx` — 4-component bar chart
- [ ] `frontend/src/components/risk/ScoreHistoryChart.tsx` — line chart using `recharts` or `chart.js`
- [ ] `frontend/src/hooks/useRiskScore.ts`
- [ ] Simulation panel: input hypothetical loan amount → show projected score delta

**Acceptance Criteria:**
- Gauge color: A (green) → B (teal) → C (yellow) → D (orange) → F (red)
- History chart supports `?start_date=&end_date=` query params
- Invalid wallet address shows inline validation error

---

## Issue #F7: [ANALYTICS] Build Analytics / Stats Page

**Labels:** `frontend`, `analytics`, `priority: low`

**Path:** `frontend/src/app/analytics/page.tsx`

**Description:**
Public-facing platform statistics dashboard showing aggregate protocol health.

**Sections to implement:**
- **KPI Cards** — Total Escrows, Active Loans, Total Volume (USDC), Active Users
- **Escrow Status Breakdown** — donut chart (Pending / Active / Completed / Disputed)
- **Loan Volume Over Time** — area chart by week/month
- **Oracle Network Health** — active oracles, avg confirmation time, dispute rate
- **Governance Activity** — proposals created / passed / rejected over time

**Tasks:**
- [ ] `frontend/src/app/analytics/page.tsx`
- [ ] `frontend/src/components/analytics/KpiCard.tsx`
- [ ] `frontend/src/components/analytics/EscrowDonut.tsx`
- [ ] `frontend/src/components/analytics/LoanVolumeChart.tsx`
- [ ] `frontend/src/components/analytics/OracleHealthPanel.tsx`
- [ ] `frontend/src/hooks/useAnalytics.ts` — fetches `GET /api/v1/analytics`
- [ ] Data refreshes every 60 seconds (matches server-side cache TTL)

**Acceptance Criteria:**
- Page is publicly accessible (no auth required)
- All charts are responsive
- Shows loading skeletons during data fetch

---

## Issue #F8: [ORACLES] Build Oracle Network Page

**Labels:** `frontend`, `oracle`, `priority: low`

**Path:** `frontend/src/app/oracles/page.tsx`

**Description:**
Public registry of oracle nodes showing their address, status, uptime, and confirmation rates.

**Tasks:**
- [ ] `frontend/src/app/oracles/page.tsx` — sortable table of oracle nodes
- [ ] `frontend/src/components/oracles/OracleTable.tsx`
- [ ] `frontend/src/components/oracles/OracleStatusBadge.tsx` — Active / Inactive / Degraded
- [ ] `frontend/src/components/oracles/RegisterOracleModal.tsx` — form for oracle self-registration
- [ ] `frontend/src/hooks/useOracles.ts`

**Acceptance Criteria:**
- Table sortable by confirmation rate, uptime, and last-seen timestamp
- Oracle registration requires connected wallet + signature

---

## Issue #F9: [LAYOUT] Build Global Navbar and Footer

**Labels:** `frontend`, `layout`, `priority: critical`

**Description:**
The current project lacks a persistent global Navbar and Footer. These must be added to `frontend/src/app/layout.tsx`.

**Navbar items:**
- Logo (links to `/`)
- `About` · `Analytics` · `Governance` · `Risk` · `Oracles` · `Contact`
- `Connect Wallet` button (→ `/login` or triggers WalletPickerModal)
- Connected state: shows shortened address + dropdown (Dashboard, Profile, Disconnect)

**Footer sections:**
- Product links: Dashboard, Escrows, Collateral, Loans, Governance
- Company links: About, Contact, GitHub repo
- Legal: Terms of Service, Privacy Policy (placeholder pages)
- Copyright + Stellar/Soroban attribution

**Tasks:**
- [ ] `frontend/src/components/layout/Navbar.tsx`
- [ ] `frontend/src/components/layout/Footer.tsx`
- [ ] `frontend/src/components/layout/MobileMenu.tsx` — hamburger dropdown for mobile
- [ ] `frontend/src/components/auth/ConnectButton.tsx` (shared with #F3)
- [ ] Update `frontend/src/app/layout.tsx` to wrap children with `<Navbar/>` and `<Footer/>`
- [ ] Active link highlighting based on current pathname

**Acceptance Criteria:**
- Navbar collapses to hamburger on mobile (breakpoint: `md`)
- Wallet connection state correctly reflected in Navbar across all pages
- Footer is present on all public pages

---

## Issue #F10: [LEGAL] Build Terms of Service and Privacy Policy Pages

**Labels:** `frontend`, `legal`, `priority: low`

**Paths:**
- `frontend/src/app/legal/terms/page.tsx`
- `frontend/src/app/legal/privacy/page.tsx`

**Description:**
Placeholder legal pages linked from the footer. Required before public launch.

**Tasks:**
- [ ] `frontend/src/app/legal/terms/page.tsx` — Terms of Service (placeholder or actual content)
- [ ] `frontend/src/app/legal/privacy/page.tsx` — Privacy Policy
- [ ] `frontend/src/components/legal/LegalLayout.tsx` — shared layout with sidebar TOC
- [ ] Link both pages from `Footer.tsx`

**Acceptance Criteria:**
- Pages are server-rendered static content
- Last updated date displayed at the top
- Sidebar table of contents auto-scrolls to anchored sections

---

## Issue #F11: [ERROR] Build 404 and Error Pages

**Labels:** `frontend`, `ux`, `priority: medium`

**Paths:**
- `frontend/src/app/not-found.tsx` — global 404 page
- `frontend/src/app/error.tsx` — global error boundary

**Description:**
Next.js App Router uses `not-found.tsx` and `error.tsx` for handling unknown routes and runtime errors.

**Tasks:**
- [ ] `frontend/src/app/not-found.tsx` — branded 404 with "Go Home" and "Dashboard" CTAs
- [ ] `frontend/src/app/error.tsx` — client error boundary with error message + retry button
- [ ] `frontend/src/app/loading.tsx` — global loading skeleton for Suspense boundaries
- [ ] `frontend/src/components/ui/ErrorState.tsx` — reusable empty/error state component

**Acceptance Criteria:**
- 404 page maintains Navbar/Footer layout
- Error boundary displays a user-friendly message (not a raw stack trace)
- `loading.tsx` uses skeleton placeholders matching page content layout

---

## Issue #F12: [NOTIFICATIONS] Build Toast / Notification System

**Labels:** `frontend`, `ux`, `priority: high`

**Description:**
A global notification system for showing success, error, warning, and info toasts across all pages. Used by auth flows, XDR signing, form submissions, and WebSocket events.

**Tasks:**
- [ ] Install `sonner` or implement a custom toast provider
- [ ] `frontend/src/components/ui/Toast.tsx` — styled toast component
- [ ] `frontend/src/context/ToastContext.tsx` — global toast provider + `useToast()` hook
- [ ] Wire into `frontend/src/app/layout.tsx`
- [ ] WebSocket integration: show toast on `ESCROW_UPDATED`, `LOAN_UPDATED`, `GOVERNANCE_VOTE_CAST` events
- [ ] `frontend/src/hooks/useWebSocket.ts` — WebSocket client connecting to `ws://[server]/ws`

**Acceptance Criteria:**
- Toasts auto-dismiss after 5 seconds
- Max 3 toasts visible simultaneously; older ones queue
- WebSocket reconnects automatically on disconnect

---

## Issue #F13: [SETTINGS] Build User Settings Page

**Labels:** `frontend`, `settings`, `priority: low`

**Path:** `frontend/src/app/settings/page.tsx`

**Description:**
Authenticated settings page for managing wallet connections, notification preferences, and display options.

**Sections:**
- **Wallets** — list linked wallets, set primary, unlink, add new wallet (reuses challenge flow from #F3)
- **Notifications** — toggle WebSocket event types
- **Display** — theme (light/dark), date format preference

**Tasks:**
- [ ] `frontend/src/app/settings/page.tsx`
- [ ] `frontend/src/components/settings/WalletManager.tsx`
- [ ] `frontend/src/components/settings/NotificationPreferences.tsx`
- [ ] `frontend/src/hooks/useSettings.ts`
- [ ] Dark mode toggle — stores preference in `localStorage`; applies via `next-themes`

**Acceptance Criteria:**
- Wallet changes call `PUT /api/v1/wallets/:id/primary` and `DELETE /api/v1/wallets/:id`
- Dark mode preference persists across sessions
- Page requires authentication

---

## Implementation Order

```
#F9 Navbar + Footer → #F3 Auth/Login → #F11 404/Error Pages → #F12 Notifications
    → #F4 Loans → #F5 Governance → #F6 Risk → #F7 Analytics
    → #F8 Oracles → #F1 About → #F2 Contact → #F10 Legal → #F13 Settings
```

## Summary Table

| # | Page(s) | Status | Priority |
|---|---------|--------|----------|
| F1 | `/about` | ❌ Missing | Medium |
| F2 | `/contact` | ❌ Missing | Medium |
| F3 | `/login` | ❌ Missing | **Critical** |
| F4 | `/loans`, `/loans/[id]`, `/loans/new` | ❌ Missing | High |
| F5 | `/governance`, `/governance/[id]`, `/governance/new` | ❌ Missing | Medium |
| F6 | `/risk`, `/risk/[wallet]` | ❌ Missing | Medium |
| F7 | `/analytics` | ❌ Missing | Low |
| F8 | `/oracles` | ❌ Missing | Low |
| F9 | Global Navbar + Footer | ❌ Missing | **Critical** |
| F10 | `/legal/terms`, `/legal/privacy` | ❌ Missing | Low |
| F11 | `not-found.tsx`, `error.tsx`, `loading.tsx` | ❌ Missing | Medium |
| F12 | Toast system + WebSocket client | ❌ Missing | High |
| F13 | `/settings` | ❌ Missing | Low |
