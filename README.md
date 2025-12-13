# CTD - Crash To Desktop Reporter

A crash reporter for modded games that helps players share crash context with mod creators.

**Version**: 0.1.0
**Hosted**: [ctd.ezmode.games](https://ctd.ezmode.games)
**License**: AGPL-3.0
**API Spec**: [OpenAPI 3.1](https://ctd.ezmode.games/openapi.json)

## What is CTD?

CTD captures crash context from modded games and makes it easy to share with mod creators:

- **Stack traces** with module offsets for debugging
- **Load order snapshots** at crash time
- **Pattern detection** across users (same crash = same cause)
- **Anonymous or account-linked** reports

## Supported Games

**Phase 1 (MVP)**
- Skyrim Special Edition (SKSE64)

**Phase 2**
- Fallout 4 (F4SE)
- Fallout New Vegas (NVSE)
- Fallout 3 (FOSE)
- Oblivion (OBSE)
- Morrowind (MWSE)

**Phase 3**
- Baldur's Gate 3
- Cyberpunk 2077
- Stardew Valley

## Repository Structure

```
ctd/
├── crates/
│   ├── ctd-core/       # Rust library - crash capture, load order parsing
│   ├── ctd-skse64/     # SKSE64 plugin for Skyrim SE
│   └── ctd-app/        # System tray application
├── api/                # Hono API (TypeScript)
│   ├── src/
│   │   ├── index.ts
│   │   ├── routes/
│   │   ├── db/
│   │   └── lib/
│   ├── package.json
│   └── drizzle.config.ts
└── README.md
```

## API Documentation

The API follows OpenAPI 3.1 specification. Interactive documentation available at:

- **Swagger UI**: [ctd.ezmode.games/docs](https://ctd.ezmode.games/docs)
- **OpenAPI JSON**: [ctd.ezmode.games/openapi.json](https://ctd.ezmode.games/openapi.json)

### Public Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/crash-reports` | Submit a new crash report |
| `GET` | `/api/crash-reports/{id}` | Get crash report by ID |
| `GET` | `/api/games/{gameId}/patterns` | List known crash patterns |
| `GET` | `/api/patterns/{id}` | Get pattern details |

### Authenticated Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/me/crash-reports` | List user's crash reports |
| `PATCH` | `/api/crash-reports/{id}` | Update report |
| `DELETE` | `/api/crash-reports/{id}` | Delete report |

### Creator Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/creator/crash-subscriptions` | List subscribed mods |
| `POST` | `/api/creator/crash-subscriptions` | Subscribe to mod crashes |
| `GET` | `/api/creator/crash-feed` | Crash feed for your mods |

## Self-Hosting

Full source code available. Run the exact same code we do:

```bash
# Clone and run
git clone https://github.com/ezmode-games/ctd
cd ctd/api
pnpm install
pnpm dev

# Or build and run
pnpm build
node dist/index.js --port 3000 --db ./ctd.db
```

### Database Options

```bash
# SQLite (default, zero config)
node dist/index.js --db ./ctd.db

# PostgreSQL
node dist/index.js --database-url postgres://user:pass@host/db

# MySQL
node dist/index.js --database-url mysql://user:pass@host/db
```

## For Mod Creators

Subscribe to crashes that mention your mods:

1. **See crashes mentioning your mod** - Not blame, but visibility
2. **Pattern detection** - "50 crashes with your mod + ENB + this other mod"
3. **User communication** - Reply to crash reports with fix suggestions
4. **Export data** - CSV export for analysis

## Privacy

**Anonymous mode (default):**
- No account required
- Crash reports stored with random ID
- Load order stored (mod names only, no paths)
- No PII collected

**Account mode:**
- Link reports to ezmode account
- Private by default
- Can share via token
- Can delete reports

**Data retention:**
- Anonymous reports: 90 days
- Account reports: Until deleted
- Minidumps: 30 days

## Development

### API (TypeScript/Hono)

```bash
cd api
pnpm install
pnpm dev
```

Uses `@hono/zod-openapi` for type-safe routes with automatic OpenAPI spec generation.

### Rust Components

```bash
# Build all crates
cargo build --release

# Run tests
cargo test
```

## Contributing

Contributions welcome. This is AGPL-3.0 licensed - any modifications must be open sourced.

## License

AGPL-3.0 - See [LICENSE](LICENSE) for details.
