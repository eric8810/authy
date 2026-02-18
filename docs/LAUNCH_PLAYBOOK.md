# Authy Launch Playbook

Target audience: developers building with AI agents (Claude Code, OpenClaw, MCP servers, CI/CD pipelines).

---

## Phase 1: Immediate (Day 1)

### GitHub Repo Optimization

The repo is the primary landing page for developers. Make it discoverable.

**Topics** (Settings > Topics):

```
secrets-management, cli, ai-agents, claude-code, mcp-servers,
rust, encryption, devtools, security, environment-variables,
agent-skills, secrets-vault
```

**Checklist:**

- [ ] Set "About" description: `CLI secrets store & dispatch for AI agents. Encrypted vault, scoped policies, run-only tokens. Single binary, no server.`
- [ ] Set website URL to `https://eric8810.github.io/authy`
- [ ] Upload social preview image (og-image.png) under Settings
- [ ] Create v0.2.0 GitHub Release with changelog from CHANGELOG.md
- [ ] Verify README renders correctly with badges, install instructions, and quick start

### Show HN (Hacker News)

Highest-signal channel for developer tools. One good Show HN post can drive thousands of visits in a single day.

**Title:**

```
Show HN: Authy – CLI secrets store for AI agents (run-only tokens, scoped policies)
```

**Body:**

```
AI agents like Claude Code read .env files silently. OpenClaw stores API keys
in plaintext JSON. MCP servers require secrets hardcoded in config. I built
Authy to give agents scoped access to secrets without ever seeing the values.

It's a single Rust binary — encrypted vault (age), glob-based policies,
run-only tokens (agents can inject secrets into subprocesses but can't read
them directly), and a tamper-evident audit log. No server, no accounts.

Install: npm install -g authy-cli
GitHub: https://github.com/eric8810/authy
Landing page: https://eric8810.github.io/authy

How it works:

  authy policy create claude-code --allow "anthropic-*" --run-only
  authy run --scope claude-code --uppercase --replace-dash _ -- claude
  # Claude sees ANTHROPIC_API_KEY in its env — but can't read or export it

Built with Rust, secured with age encryption. Feedback welcome.
```

**Timing:** Post between 9-11am ET on Tuesday, Wednesday, or Thursday for maximum visibility.

**Tips:**
- Be factual and concise. HN readers dislike hype.
- Respond to every comment in the first 2 hours.
- If asked "why not just use X?", answer honestly about tradeoffs.

### Reddit — r/ClaudeAI

Your most targeted subreddit. These users experience the exact problem Authy solves.

**Title:**

```
I built a secrets manager so Claude Code can't silently read my .env files
```

**Body:**

```
Claude Code reads .env files without asking. If you have API keys, database
credentials, or tokens in .env, Claude sees all of them — even the ones it
doesn't need.

I built Authy to fix this. It's a CLI secrets store where you:

1. Store secrets in an encrypted vault (not plaintext files)
2. Create scoped policies (Claude only sees anthropic-* and github-*)
3. Use run-only tokens (Claude can inject secrets into subprocesses
   but can never read the values directly)

  authy policy create claude-code --allow "anthropic-*" --allow "github-*" --run-only
  authy run --scope claude-code --uppercase --replace-dash _ -- claude

Single Rust binary, no server, no accounts. There's also an Agent Skill
so Claude learns the commands automatically:

  npx skills add eric8810/authy

GitHub: https://github.com/eric8810/authy

Would love feedback from other Claude Code users. What secrets management
workflow do you currently use?
```

---

## Phase 2: First Week

### Reddit — Additional Subreddits

Post one per day to avoid spam filters. Adjust the angle for each audience.

| Day | Subreddit | Title |
|-----|-----------|-------|
| 2 | r/LocalLLaMA | `Managing API keys for local AI agents — no server, scoped access, run-only tokens` |
| 3 | r/rust | `Show: age-encrypted CLI secrets vault with run-only enforcement, built in Rust` |
| 4 | r/commandline | `authy — CLI secrets store with scoped policies, subprocess injection, and audit trail` |
| 5 | r/devops | `Scoped secrets injection for AI agents and CI/CD — single binary, no server` |
| 6 | r/programming | Same angle as Show HN post |

**Tips:**
- Read each subreddit's rules before posting.
- Frame as sharing a project and asking for feedback, not advertising.
- Engage with comments. Answer questions thoroughly.

### Twitter/X Thread

Target the AI developer community.

```
1/ Your AI agent has access to all your API keys. Here's why that's
a problem and what I built to fix it.

2/ Claude Code reads .env files silently. OpenClaw stores keys in
plaintext JSON. MCP servers need secrets hardcoded in config files.
Every agent sees everything.

3/ I built Authy — a CLI secrets store designed for agents.

The core idea: run-only tokens. Agents inject secrets into subprocesses
but can never read the values directly.

4/ How it works:

  authy policy create claude-code --allow "anthropic-*" --run-only
  authy run --scope claude-code -- claude

Claude sees ANTHROPIC_API_KEY in its env. It can't authy get the value.
It can't export it. It can only use it.

5/ Single Rust binary. age encryption. No server. No accounts.
Full audit trail.

  npm install -g authy-cli

github.com/eric8810/authy

Feedback welcome.
```

**Tags:** @AnthropicAI, relevant AI dev accounts

**Hashtags:** #AIAgents #DevTools #Rust #Security #ClaudeCode

**Timing:** Post in the morning ET. Pin the thread.

### Blog Post (Dev.to + Hashnode)

Write one post, cross-post to both platforms.

**Title:** `Why Your AI Agent Shouldn't Have Access to All Your Secrets`

**Structure:**

1. **The problem** — Claude Code reads .env, OpenClaw stores plaintext, MCP configs commit secrets to repos. Use real examples.
2. **What run-only mode means** — diagram showing admin stores secrets, creates policy, agent can only inject via subprocess.
3. **5-minute getting started** — install, init, store, create policy, launch agent. Code blocks for each step.
4. **Agent Skills** — one command teaches your agent. `npx skills add eric8810/authy`.
5. **Comparison table** — Authy vs pass vs Vault vs 1Password CLI.
6. **Call to action** — GitHub link, star, try it, file issues.

**Dev.to tags:** `ai`, `security`, `rust`, `cli`
**Hashnode tags:** `ai-tools`, `rust`, `developer-tools`, `security`

### Product Hunt Launch

Schedule a launch for a Tuesday or Wednesday.

**Prepare:**
- Tagline: `Secrets store for AI agents — run-only tokens, scoped policies, full audit trail`
- Gallery: 3-4 images
  - Terminal screenshot showing `authy run` workflow
  - TUI screenshot showing the admin interface
  - Comparison table from the landing page
  - Architecture diagram showing the data flow
- First comment: explain your personal motivation (why you built it)
- Hunter: launch yourself or find a hunter with followers

---

## Phase 3: Ongoing

### Community Presence

Go where your target users already are. Don't just post links — participate in discussions, answer questions about secrets management, and mention Authy when relevant.

| Community | Platform | Angle |
|-----------|----------|-------|
| Claude Code users | Anthropic Discord, r/ClaudeAI | Secrets security for agent workflows |
| MCP developers | MCP Discord, GitHub Discussions | Keep secrets out of .mcp.json |
| Rust security | r/rust, Rust Users Forum | age encryption, zeroize, Rust CLI design |
| Agent Skills | skills.sh, ClawHub | Portable agent tool instructions |
| DevOps / CI | r/devops, DevOps Discord | Scoped secrets for pipelines and agents |

### Awesome Lists

Submit pull requests to curated lists for sustained organic discovery.

- [ ] [awesome-rust](https://github.com/rust-unofficial/awesome-rust) — Security / Cryptography section
- [ ] [awesome-cli-apps](https://github.com/agarrharr/awesome-cli-apps) — Security section
- [ ] [awesome-security](https://github.com/sbilly/awesome-security) — relevant section
- [ ] Any `awesome-mcp` or `awesome-ai-agents` list that exists
- [ ] [awesome-self-hosted](https://github.com/awesome-selfhosted/awesome-selfhosted) — if applicable

**PR format:** Follow each list's contribution guidelines exactly. Keep the description to one line matching the style of existing entries.

### Repeat Touchpoints

New releases are opportunities to re-engage. For each new version:

- [ ] Write a changelog post on Dev.to / Twitter
- [ ] Post release notes to relevant subreddits
- [ ] Update Product Hunt listing
- [ ] Announce in Discord communities

---

## Programmatic Growth

Engineering-driven strategies that compound over time. Manual posting gets a spike; these get a curve.

### Distribution Channels to Build

**GitHub Action (highest leverage)**

Create `authy-action` on the GitHub Actions Marketplace. Every CI/CD user who searches "secrets" or "environment variables" discovers you organically.

```yaml
# users would write:
- uses: eric8810/authy-action@v1
  with:
    scope: deploy
    keyfile: ${{ secrets.AUTHY_KEYFILE }}
    run: ./deploy.sh
```

The marketplace is a search engine. Actions with good READMEs rank well and get installed passively.

- [ ] Create `eric8810/authy-action` repo
- [ ] Implement action.yml + wrapper script
- [ ] Publish to GitHub Actions Marketplace
- [ ] Add usage example to Authy README

**Homebrew Tap**

macOS developers expect `brew install`. A tap is a GitHub repo with a formula.

```bash
brew tap eric8810/authy
brew install authy
```

Homebrew has its own search and discovery. Once listed, it surfaces in `brew search` results forever.

- [ ] Create `eric8810/homebrew-authy` repo
- [ ] Write formula (download binary, verify checksum)
- [ ] Add `brew install` to README install section

**Package Manager Breadth**

Each package manager is an independent discovery channel.

| Manager | Command | Audience | Effort |
|---------|---------|----------|--------|
| npm | `npm install -g authy-cli` | JS/TS devs | Done |
| Homebrew | `brew install eric8810/authy/authy` | macOS devs | Low |
| AUR | `yay -S authy-cli` | Arch Linux | Low |
| Chocolatey | `choco install authy-cli` | Windows devs | Medium |
| Nix | `nix-env -i authy` | Nix users | Medium |
| crates.io | `cargo install authy` | Rust devs | Low |
| Docker Hub | `docker pull eric8810/authy` | Container users | Low |

- [ ] Publish to crates.io (Rust community discovers via `cargo search`)
- [ ] Create Homebrew tap
- [ ] Submit AUR package
- [ ] Create Docker Hub image with minimal Dockerfile

**Starter Templates**

Create template repos that come with Authy pre-configured. GitHub suggests templates to users. Forks and clones are passive growth.

- [ ] `authy-claude-starter` — Claude Code project with vault, policy, and shell alias
- [ ] `authy-mcp-starter` — MCP servers with secrets managed by Authy
- [ ] `authy-cicd-starter` — GitHub Actions + Authy for deploy pipelines

**npm Postinstall Banner**

Add a helpful message after `npm install -g authy-cli`:

```
Installed authy-cli v0.2.0
  Get started:  authy init --generate-keyfile ~/.authy/keys/master.key
  Agent skill:  npx skills add eric8810/authy
  Docs:         https://github.com/eric8810/authy
```

Every install becomes a mini-onboarding moment.

- [ ] Add postinstall script to npm/authy-cli/package.json

**VS Code / Cursor Extension**

A simple extension that shows vault status in the status bar, provides snippets for `authy run`, and warns when `.env` files are detected. The extensions marketplace is a passive discovery channel.

- [ ] Create `authy-vscode` extension
- [ ] Publish to VS Code Marketplace and Open VSX

### Monitoring and Analytics

Track signals to know what's working:

- [ ] npm weekly downloads — `npm info authy-cli`
- [ ] GitHub traffic — Settings > Traffic > Clones, views, referrers
- [ ] GitHub star history — star-history.com
- [ ] Google Search Console — what queries lead to your site
- [ ] Landing page analytics — add Plausible or Umami (privacy-friendly)
- [ ] Track referral sources to understand which channels convert

---

## SEO Strategy

### Current Problems

The landing page is a React SPA that renders everything client-side. This creates three critical issues:

1. **Google only indexes the English version.** All 9 translations render via i18next at runtime. Google's crawler may not execute the JavaScript reliably, and even if it does, there's a single URL for all languages.
2. **No separate URLs per language.** Without distinct URLs, there are no hreflang signals, no per-language meta tags, and no way for search engines to serve the right language to the right user.
3. **Tailwind CDN hurts performance.** Loading the full Tailwind runtime from CDN (~300KB+) degrades Core Web Vitals (LCP, CLS). Google ranks faster pages higher.

### Fix: Pre-Render Per Language

Transform from a single SPA to pre-rendered pages per language. This is the single highest-impact SEO change — it multiplies organic surface area by 9x.

**Target URL structure:**

```
/authy/           → English (default)
/authy/zh/        → Simplified Chinese
/authy/zh-TW/     → Traditional Chinese
/authy/ja/        → Japanese
/authy/ko/        → Korean
/authy/fr/        → French
/authy/de/        → German
/authy/es/        → Spanish
/authy/pt/        → Portuguese
```

**Options:**

| Approach | Effort | Result |
|----------|--------|--------|
| Vite SSG plugin (vite-ssg) | Low | Pre-renders routes at build time, keep current React code |
| Migrate to Astro | Medium | Best static site perf, content-first, supports React components |
| Next.js static export | Medium | Familiar React patterns, built-in i18n routing |

Recommendation: Vite SSG plugin or Astro migration. Both produce static HTML per route that GitHub Pages can serve directly.

- [ ] Choose SSG approach
- [ ] Add route-per-language with pre-rendered HTML
- [ ] Set `<html lang="...">` per page
- [ ] Set per-language `<title>` and `<meta name="description">`
- [ ] Set per-language Open Graph tags (og:title, og:description, og:locale)
- [ ] Bundle Tailwind at build time (PostCSS or Vite plugin) — replace CDN

### Technical SEO Checklist

**Missing essentials:**

- [ ] `sitemap.xml` — list all language pages, comparison pages, guide pages
- [ ] `robots.txt` — allow all crawlers, point to sitemap
- [ ] `hreflang` tags on every page (see template below)
- [ ] Canonical URL per language page
- [ ] Submit to Google Search Console
- [ ] Submit to Baidu Webmaster Tools (ziyuan.baidu.com)

**hreflang template** (include on every page):

```html
<link rel="alternate" hreflang="en" href="https://eric8810.github.io/authy/" />
<link rel="alternate" hreflang="zh" href="https://eric8810.github.io/authy/zh/" />
<link rel="alternate" hreflang="zh-TW" href="https://eric8810.github.io/authy/zh-TW/" />
<link rel="alternate" hreflang="ja" href="https://eric8810.github.io/authy/ja/" />
<link rel="alternate" hreflang="ko" href="https://eric8810.github.io/authy/ko/" />
<link rel="alternate" hreflang="fr" href="https://eric8810.github.io/authy/fr/" />
<link rel="alternate" hreflang="de" href="https://eric8810.github.io/authy/de/" />
<link rel="alternate" hreflang="es" href="https://eric8810.github.io/authy/es/" />
<link rel="alternate" hreflang="pt" href="https://eric8810.github.io/authy/pt/" />
<link rel="alternate" hreflang="x-default" href="https://eric8810.github.io/authy/" />
```

### Keyword Strategy

Target keywords fall into three tiers. Each needs different content.

**Tier 1 — Direct intent** (people searching for what you built):

These should be covered by the landing page.

```
secrets manager for ai agents
cli secrets store
secure api keys for claude code
mcp server secrets management
run-only secrets tokens
```

**Tier 2 — Comparison queries** (people evaluating alternatives):

These need dedicated comparison pages.

```
pass vs vault
secrets manager comparison
1password cli alternative
hashicorp vault alternative local
sops vs vault
dotenv security risks
```

**Tier 3 — Problem queries** (people experiencing the pain):

These need blog posts or tutorial guides.

```
claude code reads env files
ai agent api key security
how to secure mcp server config
prevent .env file leaks
environment variable security best practices
```

### Multi-Page Content Architecture

Expand from a single landing page to multiple SEO-targeted pages:

```
/                          → Landing page (English)
/{lang}/                   → Landing page (per language)
/vs/pass                   → Authy vs pass
/vs/vault                  → Authy vs HashiCorp Vault
/vs/1password-cli          → Authy vs 1Password CLI
/vs/sops                   → Authy vs SOPS
/vs/dotenv                 → Authy vs .env files
/guides/claude-code        → Securing Claude Code with Authy
/guides/mcp-servers        → Managing MCP Server Secrets
/guides/cicd               → Authy in CI/CD Pipelines
/blog/why-env-dangerous    → Why .env Files Are Dangerous for AI Agents
```

Each page targets a distinct search query and earns its own backlinks.

- [ ] Create /vs/ comparison pages (one per competitor)
- [ ] Create /guides/ tutorial pages (one per integration)
- [ ] Add structured data (JSON-LD) to each page
- [ ] Internal link between pages (comparison pages link to guides, guides link to landing page)

### Per-Language Meta Tags

Each language version needs native-language meta tags. Don't just translate the page body — translate the title and description that appear in search results.

**English:**
```html
<title>Authy | CLI Secrets Store for AI Agents</title>
<meta name="description" content="Securely manage and dispatch secrets to AI agents with scoped access, run-only tokens, and complete audit trails. Single binary, no server." />
```

**Chinese (zh):**
```html
<title>Authy | AI Agent 密钥管理工具</title>
<meta name="description" content="为 AI Agent 提供安全的密钥管理与分发。加密存储、权限策略、仅运行令牌、完整审计追踪。单文件部署，无需服务器。" />
```

**Japanese (ja):**
```html
<title>Authy | AIエージェント向けCLIシークレット管理</title>
<meta name="description" content="AIエージェントにスコープ付きアクセス、実行専用トークン、完全な監査証跡でシークレットを安全に管理・配信。サーバー不要の単一バイナリ。" />
```

**Korean (ko):**
```html
<title>Authy | AI 에이전트용 CLI 시크릿 관리</title>
<meta name="description" content="AI 에이전트에게 스코프 기반 접근, 실행 전용 토큰, 완전한 감사 추적으로 시크릿을 안전하게 관리 및 배포. 서버 불필요, 단일 바이너리." />
```

Apply the same pattern for fr, de, es, pt, zh-TW.

---

## Geo Strategy

### Developer Populations by Region

| Region | Search Engine | Developer Communities | Language |
|--------|--------------|----------------------|----------|
| US / Europe | Google | HN, Reddit, Dev.to, Twitter | en |
| China | Baidu, Bing CN | V2EX, SegmentFault, Zhihu, Juejin, WeChat | zh |
| Japan | Google JP, Yahoo JP | Qiita, Zenn | ja |
| Korea | Naver, Google KR | Velog, GeekNews | ko |
| Brazil | Google BR | TabNews, Dev.to PT | pt |
| DACH | Google DE | Heise, Dev.to DE | de |
| Latin America | Google | Dev.to ES | es |
| France | Google FR | Dev.to FR, Journal du Hacker | fr |
| Taiwan | Google TW | PTT, iT 邦幫忙 | zh-TW |

### China (Biggest Untapped Market)

China has the largest developer population outside the US, and they're rapidly adopting AI agents. But they have unique infrastructure constraints.

**Access problem:** GitHub Pages is slow or blocked in parts of China. Your landing page may not load at all.

**Fix:**
- [ ] Mirror the landing page to a China-accessible host (Vercel, Netlify, or Chinese CDN like Alibaba Cloud / Tencent Cloud)
- [ ] Consider a custom domain with Cloudflare CDN for global edge caching
- [ ] If targeting China seriously, register a .cn or .com.cn domain (requires ICP filing)

**Baidu SEO specifics:**
- Baidu's crawler does NOT execute JavaScript well — pre-rendered HTML is essential (see SSG migration above)
- Baidu ranks Chinese backlinks much higher than global backlinks
- Submit sitemap to Baidu Webmaster Tools (ziyuan.baidu.com)
- Baidu favors pages hosted within China; a Chinese CDN helps ranking

**Community posts:**

| Platform | Format | Angle |
|----------|--------|-------|
| V2EX | `/t/` topic in Developer Tools node | "我做了一个 AI Agent 密钥管理工具" — problem-first, ask for feedback |
| Juejin | Article in "Tools" category | Tutorial: 5 分钟配置 AI Agent 安全密钥管理 |
| SegmentFault | Q&A or article | Answer existing questions about Claude Code / .env security |
| Zhihu | Answer relevant questions | Search for AI agent security questions, provide detailed answers |
| WeChat | Public account article | Broader reach if you have access to a public account |

**V2EX post draft:**

```
标题：开源：给 AI Agent 用的密钥管理工具（Rust，单文件，加密存储）

Claude Code 会默默读取你的 .env 文件里所有 API Key。
OpenClaw 把密钥明文存储在 JSON 配置中。
MCP Server 需要在配置文件里硬编码密钥。

我做了 Authy 来解决这个问题：

1. 密钥加密存储在 age 加密的保险库中
2. 用策略控制每个 Agent 能看到哪些密钥
3. 仅运行模式 — Agent 只能通过子进程注入密钥，不能直接读取值

  authy policy create claude-code --allow "anthropic-*" --run-only
  authy run --scope claude-code --uppercase --replace-dash _ -- claude

单个 Rust 可执行文件，无需服务器，无需注册。

安装：npm install -g authy-cli
GitHub：https://github.com/eric8810/authy

求反馈和建议。
```

### Japan

Japanese developers actively use AI tools and value polished, well-documented projects. Japanese-language content ranks very well on Google JP.

| Platform | Format | Notes |
|----------|--------|-------|
| Qiita | Tutorial article | Focus on step-by-step: install → store → policy → run. Japanese devs appreciate thoroughness. |
| Zenn | In-depth technical article | Architecture deep-dive: age encryption, HMAC tokens, audit chain. More technical audience than Qiita. |

Target keywords for Japanese SEO:
```
AIエージェント シークレット管理
CLI 暗号化 秘密鍵管理
Claude Code API キー セキュリティ
MCP サーバー 秘密鍵 設定
環境変数 安全管理
```

- [ ] Write Qiita tutorial in Japanese
- [ ] Write Zenn architecture article in Japanese
- [ ] Ensure Japanese landing page `/ja/` has native-language meta tags

### Korea

| Platform | Format | Notes |
|----------|--------|-------|
| GeekNews | Link submission | Korean equivalent of Hacker News. High-quality dev audience. |
| Velog | Tutorial article | Most popular Korean dev blogging platform. |

Target keywords for Korean SEO:
```
AI 에이전트 시크릿 관리
CLI 암호화 비밀 관리
Claude Code API 키 보안
환경변수 보안 관리
```

- [ ] Submit to GeekNews
- [ ] Write Velog tutorial in Korean
- [ ] Ensure Korean landing page `/ko/` has native-language meta tags

### Taiwan

| Platform | Format | Notes |
|----------|--------|-------|
| PTT | Post in Soft_Job or Programming board | Taiwan's largest forum. Technical audience. |
| iT 邦幫忙 | Tutorial series | Part of iThome, major Taiwanese tech media. |

- [ ] Post on PTT Soft_Job board
- [ ] Write tutorial on iT 邦幫忙

### Brazil / Latin America

| Platform | Format | Notes |
|----------|--------|-------|
| TabNews | Article | Brazilian dev community (growing fast) |
| Dev.to (PT/ES) | Cross-post translated blog | Tag with Portuguese/Spanish-specific tags |

- [ ] Cross-post blog to Dev.to in Portuguese
- [ ] Post on TabNews

### France / DACH

| Platform | Format | Notes |
|----------|--------|-------|
| Journal du Hacker | Link submission | French HN equivalent |
| Dev.to (FR/DE) | Cross-post translated blog | Smaller but engaged communities |

- [ ] Submit to Journal du Hacker
- [ ] Cross-post blog to Dev.to in French and German

---

## Drafts — Ready to Copy-Paste

### GitHub About

```
CLI secrets store & dispatch for AI agents. Encrypted vault, scoped policies, run-only tokens. Single binary, no server.
```

### One-Liner (for DMs, forums, comments)

```
Authy is a CLI secrets store for AI agents — encrypted vault, scoped policies,
and run-only tokens so agents inject secrets into subprocesses but never see
the values. Single Rust binary, no server. https://github.com/eric8810/authy
```

### npm Package Description (already set)

```
CLI secrets store & dispatch for AI agents
```

---

## Priority Summary

### Launch (Week 1)

| Priority | Action | Impact | Effort |
|----------|--------|--------|--------|
| 1 | GitHub topics + about + release | Discovery, SEO | 15 min |
| 2 | Show HN | Highest single-day traffic | 30 min |
| 3 | r/ClaudeAI post | Exact target audience | 20 min |
| 4 | Twitter thread | Reaches AI dev community | 20 min |
| 5 | Dev.to blog post | Long-tail SEO, credibility | 2 hours |
| 6 | Product Hunt | Broader dev audience | 1 hour prep |
| 7 | Awesome list PRs | Sustained discovery | 30 min each |
| 8 | Community engagement | Trust, word of mouth | Ongoing |

### Programmatic Growth

| Priority | Action | Why It Compounds |
|----------|--------|-----------------|
| 1 | GitHub Action | Marketplace is a passive search engine |
| 2 | Homebrew tap | `brew search` is how macOS devs find CLI tools |
| 3 | crates.io publish | Rust community discovery via `cargo search` |
| 4 | Starter templates | Forks and clones grow passively |
| 5 | npm postinstall banner | Every install becomes onboarding |
| 6 | VS Code extension | Extensions marketplace is passive discovery |

### SEO

| Priority | Action | Impact |
|----------|--------|--------|
| 1 | Pre-render per language (SSG migration) | Unlocks SEO for all 9 languages — 9x surface area |
| 2 | sitemap.xml + robots.txt + hreflang | Google discovers and indexes all pages |
| 3 | Bundle Tailwind at build time | Page speed + Core Web Vitals improvement |
| 4 | Comparison pages (/vs/...) | Captures "X vs Y" search traffic |
| 5 | Per-language meta tags | Native search result snippets in each language |
| 6 | Submit to Google Search Console + Baidu | Indexed in both search engines |
| 7 | Guide/tutorial pages | Long-tail SEO for integration queries |

### Geo

| Priority | Action | Market |
|----------|--------|--------|
| 1 | V2EX + Juejin posts | China — largest untapped developer market |
| 2 | Mirror landing page to China-accessible host | China — GitHub Pages may not load |
| 3 | Qiita + Zenn posts | Japan — engaged, high-quality dev community |
| 4 | GeekNews + Velog | Korea — growing AI dev ecosystem |
| 5 | Submit to Baidu Webmaster Tools | China — Baidu SEO |
| 6 | PTT + iT 邦幫忙 | Taiwan |
| 7 | TabNews | Brazil |
| 8 | Journal du Hacker | France |

---

The single highest-leverage technical change is **SSG migration** — pre-rendering per language. Right now 8 of your 9 language translations are invisible to every search engine. Fixing that multiplies your organic surface area overnight. Everything else in the SEO and geo sections depends on it.
