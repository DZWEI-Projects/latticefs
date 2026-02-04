# Architecture Overview

This document summarizes how the CLI, base library, and share service fit together at a high level.

## System flow (MVP)

```mermaid
flowchart LR
    CLI[lfs CLI] -->|commands| Repo[LatticeRepo]
    Repo -->|metadata ops| Sled[(sled metadata)]
    Repo -->|chunk ops| ChunkStore[Chunk store]
    Repo -->|events| Bus[Event bus]
    Repo -->|policy checks| Policy[Policy engine]
    Repo -->|views| Query[Query engine]

    subgraph Services
        Share[Share server]
        IPC[IPC Unix socket]
    end

    CLI -->|shares, revoke| Repo
    Share -->|UCAN validation| Repo
    IPC <-->|request/response| Repo
```

## Module boundaries

- **CLI**: argument parsing, UX, and command dispatch.
- **base**: storage, policies, crypto, DAG, views, FUSE.
- **services**: share server APIs and IPC plumbing.

## Data flow highlights

- **Write**: CLI → repo → chunk store + metadata → event bus → audit log.
- **Read**: CLI → repo → metadata → chunk store → output.
- **Share**: CLI → repo → UCAN capability → share server (optional).
