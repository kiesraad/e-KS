**Let op: dit project bevindt zich momenteel in een opstartfase. Documentatie en code zullen onvolledig en soms incorrect zijn.**

# e-KS

Om te kunnen deelnemen aan een verkiezing moet een politieke groepering aangeven met welke kandidaten ze mee wil doen. Hiervoor moeten ze verschillende documenten inleveren bij het centraal stembureau. Dit heet de kandidaatstellingsprocedure.

e-KS staat voor het elektronisch Kandidaatstellingssysteem: een webapplicatie waarmee de Kiesraad de huidige kandidaatstellingsprocedure op een eerlijke, transparante en controleerbare manier wil moderniseren. Het nieuwe systeem zal op termijn de huidige ondersteunende software (OSV2020-PP en OSV2020-KS) vervangen.

## Requirements

De kandidaatstellingsprocedure is verankerd in de [Kieswet](https://wetten.overheid.nl/BWBR0004627/2025-08-01).

Een overzicht van het huidige proces en e-KS is te lezen in [deze presentatie](https://github.com/user-attachments/files/24053768/e-KS-Proces.pdf).

Belangrijke stukken of [formulieren voor de kandidaatstellingsprocedure](https://www.kiesraad.nl/verkiezingen/eerste-kamer/kandidaatstelling/stukken-kandidaatstelling) zijn:

- [Kandidatenlijst H1](https://www.rijksoverheid.nl/onderwerpen/verkiezingen/documenten/publicaties/2020/12/15/model-h-1-kandidatenlijst)
- [Instemmingsverklaring H9](https://www.rijksoverheid.nl/onderwerpen/verkiezingen/documenten/publicaties/2020/12/15/model-h-9-instemmingsverklaring)
- [Machtiging om aanduiding boven lijst te plaatsen H3-1](https://www.rijksoverheid.nl/documenten/publicaties/2020/12/15/model-h-3-1-machtiging-om-aanduiding-boven-kandidatenlijst-te-plaatsen)
- [Samenvoeging aanduidingen H3-2](https://www.rijksoverheid.nl/onderwerpen/verkiezingen/documenten/publicaties/2020/12/15/model-h-3-2-machtiging-om-samengevoegde-aanduiding-boven-kandidatenlijst-te-plaatsen)
- [Ondersteuningsverklaringen H4](https://www.rijksoverheid.nl/onderwerpen/verkiezingen/documenten/publicaties/2021/08/19/model-h-4-ondersteuningsverklaring)

## Technische architectuur

Een overzicht van de voorgestelde technische afwegingen staat in [deze presentatie](https://github.com/user-attachments/files/24053801/e-KS-PSA.pdf).

### Event storage and the event hash chain

All domain changes are stored as an append-only stream of events, partitioned per
`(stream_id, election)`. Three backends exist (selected via `STORAGE_URL`):
in-memory (`memory://`), local files (`local://`), and PostgreSQL (`postgres://`).
On the file and database backends each event payload is encrypted at rest with a
per-`(stream_id, election)` AES-256-GCM key (see `src/store/encryption.rs`).

Every persisted event also carries a 32-byte `hash` that links it to the previous
event, forming a tamper-evident hash chain over the stream:

```
hash_n = SHA256( hash_{n-1} ‖ event_id_n (u64 LE) ‖ created_at_n (i64 LE, microseconds) ‖ body_n )
```

- `hash_0` (the predecessor of the first event) is the all-zero "genesis" hash
  (`GENESIS_HASH`).
- `body_n` is the *persisted* representation of the payload: the
  `nonce ‖ ciphertext ‖ tag` AES-GCM blob for the file/database backends, or the
  postcard encoding of the plaintext for the in-memory backend. Hashing the
  *encrypted* blob (which is indistinguishable from random and carries a fresh
  nonce) is deliberate: it lets the hash be stored **unencrypted** without leaking
  anything about the plaintext, while still committing to the exact stored bytes.
- `created_at` is hashed at microsecond precision because that is all that
  survives a round-trip through the on-disk frame format and Postgres `timestamptz`.

In addition, the AES-GCM *associated data* for each event is
`event_id ‖ created_at ‖ hash_{n-1}`. This authenticates the cleartext metadata
stored next to the ciphertext and pins each ciphertext to its position in the
chain: a modified payload or `event_id`/`created_at`, a reordered event, or a
removed middle event all make decryption fail on replay with
`AppError::EventDecodeError`, even without an explicit chain check. The database
backend also tracks the highest `event_id` in a `streams` row, so dropping the
*last* event is detected too.

The explicit chain check on replay (recomputing each event's stored `hash` from
the previous hash and the stored body) is gated behind the
`verify-event-hash-chain` cargo feature, off by default — it adds a SHA-256 over
every event loaded. Its only unique contribution over the AES-GCM binding above is
detecting an in-place rewrite of the stored `hash` value itself; enable it where
that extra check is worth the load-time cost.

On the database backend the `events.hash` column has an index (`events_hash_idx`)
to support looking up an event by its chain hash.

The chain is not a substitute for storing the database/files on an encrypted,
access-controlled volume; it is a defence-in-depth, integrity-detection measure.

## Development setup

1) Install prerequisites:

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://docs.docker.com/get-docker/)

2) Build and download development tools:

```bash
bin/init
```

3) Start the development environment (postgres, esbuild, cargo watch, etc.):

```bash
bin/dev
```

## Development tools

- `bin/esbuild`: transpile and bundle Typescript and CSS, also services frontend assets in development
- `bin/biome`: format and lint Typescript
- `bin/setup`: download tools, setup database, load fixtures, etc.
- `bin/dev`: start development environment (postgres, esbuild, cargo watch, etc.)
- `bin/test`: run backend and frontend tests
- `bin/init`: build and download development tools
- `bin/check`: run linters and formatters
- `bin/build`: build backend and frontend for production
- `bin/update_locales`: update locale files based on used keys in the codebase

## Playwright tests

Playwright lives in `playwright`. See `playwright/README.md` for setup and run instructions.
