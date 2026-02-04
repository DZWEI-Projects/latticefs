# Events and Audit Logs

NeuralFS records important actions to an append-only audit log.

## Event log

Events are written to:

```
~/.latticefs/logs/events.jsonl
```

Each line is a JSON record with a timestamp and event type.

## Event types (MVP)

- `object_created`
- `version_added`
- `share_issued`
- `policy_violation`

## Revocation log

Revocations are stored separately in:

```
~/.latticefs/logs/revocations.jsonl
```

The share server reads this log to enforce UCAN revocations.
