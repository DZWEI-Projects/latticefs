# Policies

Policies are declarative restrictions attached to objects (and indirectly to shared snapshots). Policies **never grant** access; they only reduce what is otherwise allowed.

## Concepts
- **Allow list**: If non-empty, only these permissions are allowed.
- **Deny list**: Explicitly removes permissions.
- **Requirements**: Additional constraints (approval, trust, tags).
- **External sharing**: If false, sharing to external parties is blocked.

Policies are stored in the metadata database and referenced by `policy_refs` on objects.

## Built-in templates
- `project-collab`
  - allow: `read`, `write`, `comment`
  - deny: `admin`
  - require: approval from `lead-architect`
  - retain: 7 years
  - external share: false
- `personal`
  - allow: `read`, `write`, `comment`, `share`
  - deny: `admin`
  - external share: true
- `compliance`
  - allow: `read`
  - deny: `write`, `comment`, `share`
  - require: minimum trust `90`
  - retain: 10 years
  - external share: false

## Requirements
Requirements are evaluated per object:
- `ApprovalFrom([names])`: requires tags `sys:approved-by:<name>` for all names.
- `MinTrust(value)`: requires `sys:trust` tag to be >= value (0-100).
- `RequireTag(tag)`: tag must match the required pattern.

## Enforcement
Policy enforcement applies to:
- object reads (CLI, export, FUSE, share fetch)
- object writes (revise, tag, link, state changes, checkout)
- sharing (`share`, share server)
- admin operations (policy apply/remove)

Most restrictive wins across all attached policies.

## CLI
Create and apply policies:
```bash
lfs policy create project-collab --template project-collab
lfs policy apply <object-id> project-collab
lfs policy remove <object-id> project-collab
```

## Related configuration
See `config.md` for quota/rate-limit configuration. Policies are separate from quotas.
