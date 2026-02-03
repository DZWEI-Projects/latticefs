# LFS-002: Lattice Query Language (LQL)

**Status:** Draft
**Version:** 0.1.0
**Date:** 2026-02-03
**Authors:** LatticeFS Team

---

## Abstract

This document specifies the Lattice Query Language (LQL), a human-readable domain-specific language for querying the LatticeFS object graph. LQL provides declarative filtering, graph traversal, sorting, and aggregation capabilities designed for semantic file discovery.

---

## 1. Introduction

### 1.1 Motivation

Traditional filesystems rely on hierarchical paths (`/path/to/file`). LatticeFS uses a graph model where objects are discovered through queries, not paths. LQL provides:

- **Semantic search**: Find by meaning, not location
- **Graph traversal**: Navigate relationships
- **Explainability**: Understand why results match
- **Composability**: Build complex queries from simple primitives

### 1.2 Design Principles

1. **Human-readable**: No SQL-like verbosity
2. **Type-safe**: Compile-time validation
3. **Deterministic**: Same query always returns same results (given same graph state)
4. **Efficient**: Indexable predicates, early termination
5. **Explainable**: Every result can be justified

### 1.3 Example

```lql
tag:project:phoenix AND type:pdf AND updated within 7d
SORT updated DESC
LIMIT 10
```

Finds: Recent PDF documents tagged with `project:phoenix`, newest first, max 10 results.

---

## 2. Lexical Structure

### 2.1 Tokens

```
KEYWORD     = "tag" | "type" | "state" | "trust" | "updated" | "created" |
              "references" | "closure" | "AND" | "OR" | "NOT" |
              "SORT" | "GROUP" | "LIMIT" | "ASC" | "DESC"

OPERATOR    = "=" | "!=" | ">=" | "<=" | ">" | "<"

LPAREN      = "("
RPAREN      = ")"
COLON       = ":"

IDENTIFIER  = [a-zA-Z_][a-zA-Z0-9_-]*
NUMBER      = [0-9]+
DURATION    = NUMBER ("s" | "m" | "h" | "d" | "w" | "y")
STRING      = '"' ([^"\\] | '\\' .)* '"'
```

### 2.2 Comments

Comments start with `#` and extend to end of line:

```lql
# Find all images
type:image/*
```

### 2.3 Whitespace

Whitespace (space, tab, newline) is ignored except within strings.

### 2.4 Case Sensitivity

- **Keywords**: Case-insensitive (`AND` = `and` = `And`)
- **Identifiers**: Case-sensitive (`project:Phoenix` ≠ `project:phoenix`)
- **Operators**: Case-insensitive

---

## 3. Grammar

### 3.1 EBNF Specification

```ebnf
query       = expr order? limit?

expr        = term (bool_op term)*
term        = predicate | NOT term | LPAREN expr RPAREN

predicate   = tag_pred
            | type_pred
            | state_pred
            | trust_pred
            | time_pred
            | traverse_pred
            | ref_pred

tag_pred    = "tag" COLON tag_path
tag_path    = IDENTIFIER (COLON IDENTIFIER)*

type_pred   = "type" COLON mimetype
mimetype    = IDENTIFIER "/" (IDENTIFIER | "*")

state_pred  = "state" COLON state_value
state_value = "draft" | "review" | "approved" | "archived"

trust_pred  = "trust" OPERATOR trust_level
trust_level = "untrusted" | "quarantined" | "trusted" | "approved"
            | NUMBER  /* numeric trust score 0-100 */

time_pred   = time_field time_op time_value
time_field  = "updated" | "created"
time_op     = "within" | "before" | "after" | "between"
time_value  = DURATION | timestamp | range

traverse_pred = "references" LPAREN ref RPAREN
              | "closure" LPAREN ref RPAREN

ref_pred    = "ref" COLON ref
ref         = uuid | hash | STRING

bool_op     = "AND" | "OR"

order       = "SORT" field direction?
direction   = "ASC" | "DESC"
field       = "updated" | "created" | "size" | "trust"

limit       = "LIMIT" NUMBER
```

### 3.2 Operator Precedence

From highest to lowest:

1. `()` (grouping)
2. `NOT`
3. `AND`
4. `OR`

Example:

```lql
A OR B AND NOT C  ==  A OR (B AND (NOT C))
```

---

## 4. Predicates

### 4.1 Tag Predicate

**Syntax:** `tag:<namespace>[:<key>[:<value>]]`

**Semantics:** Matches objects with specified tag.

**Examples:**

```lql
tag:project:phoenix              # Exact match: project/phoenix
tag:project                       # Any tag starting with "project:"
tag:priority:high                 # Exact match: priority/high
```

**Matching Rules:**

- `tag:foo` matches `tag:foo`, `tag:foo:bar`, `tag:foo:baz:qux`
- `tag:foo:bar` matches only `tag:foo:bar`
- Tags are case-sensitive

**Index:** Tag predicates SHOULD use an inverted index for O(log n) lookup.

### 4.2 Type Predicate

**Syntax:** `type:<mime-type>`

**Semantics:** Matches objects with specified MIME type. Supports wildcards.

**Examples:**

```lql
type:application/pdf              # Only PDFs
type:image/*                      # Any image (image/jpeg, image/png, ...)
type:text/plain                   # Plain text files
type:*                            # Any type (match all)
```

**Matching Rules:**

- Exact match: `type:image/jpeg` matches only JPEG images
- Wildcard: `type:image/*` matches any MIME type starting with `image/`

**Index:** MIME types SHOULD be indexed by category (major type).

### 4.3 State Predicate

**Syntax:** `state:<state-value>`

**State Values:** `draft`, `review`, `approved`, `archived`

**Examples:**

```lql
state:draft                       # Objects in draft state
state:approved                    # Approved objects
NOT state:archived                # Exclude archived
```

**State Transitions:**

```
draft → review → approved → archived
  ↓                ↓
  └───────────────→ archived
```

### 4.4 Trust Predicate

**Syntax:** `trust <op> <level>`

**Operators:** `=`, `!=`, `>`, `<`, `>=`, `<=`

**Trust Levels:**

- `untrusted` (0)
- `quarantined` (25)
- `trusted` (75)
- `approved` (100)

**Examples:**

```lql
trust >= trusted                  # Trust level ≥ 75
trust = approved                  # Only approved (100)
trust < quarantined               # Less than quarantined (< 25)
```

**Numeric Trust:**

```lql
trust >= 50                       # Numeric comparison
```

### 4.5 Time Predicate

**Syntax:** `<field> <op> <value>`

**Fields:** `updated`, `created`

**Operators:**

- `within <duration>`: Within last N time units
- `before <timestamp>`: Before absolute time
- `after <timestamp>`: After absolute time
- `between <t1> <t2>`: Between two times

**Duration Format:** `<number><unit>`

Units: `s` (seconds), `m` (minutes), `h` (hours), `d` (days), `w` (weeks), `y` (years)

**Examples:**

```lql
updated within 7d                 # Last 7 days
created after 2025-01-01          # Created in 2025+
updated between 2025-01 2025-03   # Q1 2025
```

**Timestamp Formats:**

- ISO 8601: `2025-01-15T10:30:00Z`
- Date only: `2025-01-15` (implies start of day UTC)
- Year-month: `2025-01` (implies start of month)

### 4.6 Reference Predicate

**Syntax:** `ref:<object-ref>`

**Object Reference:**

- UUID: `ref:01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e`
- Hash: `ref:af1349b9f5f9a1a6...` (BLAKE3)
- Alias: `ref:"my-important-doc"`

**Examples:**

```lql
ref:01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e  # By UUID
ref:"project-readme"                       # By alias
```

### 4.7 Graph Traversal Predicates

#### 4.7.1 References

**Syntax:** `references(<ref>)`

**Semantics:** Matches objects that directly reference the specified object (1-hop).

**Example:**

```lql
references(ref:01934e3a...)       # Objects linking to this ref
```

#### 4.7.2 Closure

**Syntax:** `closure(<ref>)`

**Semantics:** Matches objects in the transitive closure (all descendants).

**Example:**

```lql
closure(tag:project:phoenix)      # All objects transitively linked to phoenix
```

**Link Types Considered:**

- `DerivedFrom`: A is derived from B
- `References`: A references B
- `BelongsTo`: A belongs to collection B
- `Replaces`: A replaces B (newer version)

**Traversal Direction:**

- `references(X)`: Finds objects pointing TO X
- `closure(X)`: Finds all reachable objects FROM X

**Cycle Detection:** Implementations MUST detect cycles and terminate traversal.

---

## 5. Boolean Logic

### 5.1 AND

**Syntax:** `<expr> AND <expr>`

**Semantics:** Logical conjunction. Both expressions must match.

**Example:**

```lql
tag:project:phoenix AND type:pdf
```

**Short-circuit:** Implementations MAY short-circuit evaluation.

### 5.2 OR

**Syntax:** `<expr> OR <expr>`

**Semantics:** Logical disjunction. Either expression must match.

**Example:**

```lql
type:image/* OR type:video/*
```

### 5.3 NOT

**Syntax:** `NOT <expr>`

**Semantics:** Logical negation. Expression must NOT match.

**Example:**

```lql
tag:project:phoenix AND NOT state:archived
```

**Precedence:** NOT binds tighter than AND/OR.

### 5.4 Grouping

**Syntax:** `( <expr> )`

**Example:**

```lql
(tag:urgent OR tag:priority:high) AND state:review
```

---

## 6. Sorting

### 6.1 Syntax

```lql
SORT <field> [ASC | DESC]
```

**Fields:**

- `updated`: Last modification time
- `created`: Creation time
- `size`: Object size in bytes
- `trust`: Trust level (numeric)

**Default Order:** `DESC` (newest/largest/highest first)

### 6.2 Examples

```lql
SORT updated DESC                 # Newest first (default)
SORT size ASC                     # Smallest first
SORT created                      # Oldest first (implicit ASC for created)
```

### 6.3 Multi-key Sorting

**Not supported in MVP.** Future extension:

```lql
SORT trust DESC, updated DESC     # Future
```

---

## 7. Limiting

### 7.1 Syntax

```lql
LIMIT <number>
```

**Semantics:** Return at most N results.

**Example:**

```lql
tag:project:phoenix LIMIT 10
```

**Offset:** Not supported in MVP. Use pagination token in future.

---

## 8. Query Execution

### 8.1 Execution Model

```
1. Parse query → AST
2. Validate query (type checking)
3. Optimize (predicate pushdown, index selection)
4. Execute predicates → candidate set
5. Apply graph traversals
6. Sort results
7. Apply limit
8. Return results
```

### 8.2 Index Usage

Implementations SHOULD use indexes for:

- Tag predicates (inverted index: tag → object IDs)
- Type predicates (MIME type index)
- State predicates (state → object IDs)
- Time predicates (temporal index: timestamp → object IDs)

### 8.3 Optimization

**Predicate Pushdown:**

```lql
closure(X) AND tag:foo
```

Optimize: Filter by `tag:foo` first, then apply closure (smaller working set).

**Early Termination:**

```lql
tag:project:phoenix LIMIT 10
```

Stop after finding 10 matches (don't scan entire index).

---

## 9. Explainability

### 9.1 Explain Command

**CLI:**

```bash
lfs view explain <ref> --query '<lql>'
```

**Output:**

```
Object: 01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e
Matched: true

Predicates:
  ✓ tag:project:phoenix
    - Object has tag: project/phoenix
  ✓ type:application/pdf
    - MIME type: application/pdf
  ✓ updated within 7d
    - Updated: 2025-01-29 (5 days ago)

Traversal: None

Final Result: MATCH
```

### 9.2 Non-match Explanation

```
Object: 01934e3b-7c5a-7b3c-8d2e-1f4a5b6c7d8f
Matched: false

Predicates:
  ✓ tag:project:phoenix
    - Object has tag: project/phoenix
  ✗ type:application/pdf
    - MIME type: text/plain (expected: application/pdf)

Final Result: NO MATCH (failed: type:application/pdf)
```

---

## 10. Examples

### 10.1 Simple Queries

```lql
# All PDFs
type:application/pdf

# Recent images
type:image/* AND updated within 7d

# Approved documents
state:approved

# Trusted files only
trust >= trusted
```

### 10.2 Complex Queries

```lql
# Recent project documents (approved, not archived)
tag:project:phoenix AND type:application/pdf AND state:approved AND NOT state:archived
SORT updated DESC
LIMIT 10

# Quarantined executables
type:application/* AND trust = quarantined

# All documents referencing a specific report
references(ref:01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e)
```

### 10.3 Graph Traversal

```lql
# All files in a project (including derived works)
closure(tag:project:phoenix)

# High-priority items and their references
(tag:priority:high OR tag:urgent) AND references(*)
```

---

## 11. Error Handling

### 11.1 Parse Errors

**Invalid Syntax:**

```
Error: Expected COLON after "tag"
Query: tag project:phoenix
           ^
```

**Unmatched Parentheses:**

```
Error: Unmatched LPAREN
Query: (tag:project:phoenix AND type:pdf
       ^
```

### 11.2 Semantic Errors

**Invalid MIME Type:**

```
Error: Invalid MIME type: "pdf" (expected format: type/subtype)
Query: type:pdf
```

**Invalid Trust Level:**

```
Error: Unknown trust level: "medium" (expected: untrusted, quarantined, trusted, approved)
Query: trust = medium
```

### 11.3 Runtime Errors

**Nonexistent Reference:**

```
Error: Object not found: ref:01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e
```

**Query Timeout:**

```
Error: Query timeout after 30s (traversal too deep?)
```

---

## 12. Performance Considerations

### 12.1 Query Complexity

**Index Lookups (O(log n)):**

- Tag predicates
- Type predicates
- Time range queries

**Linear Scans (O(n)):**

- Trust comparisons (without index)
- Full-text search (future)

**Graph Traversal (O(V + E)):**

- `references()`
- `closure()` (bounded by max depth)

### 12.2 Optimization Hints

**Use Specific Tags:**

```lql
# Good: Narrow tag
tag:project:phoenix:deliverables

# Bad: Broad tag (large result set)
tag:project
```

**Combine Indexed Predicates:**

```lql
# Good: Both indexed
tag:project:phoenix AND type:pdf

# Bad: Unindexed filter
tag:project:phoenix AND size > 1000000
```

### 12.3 Query Limits

Implementations SHOULD enforce:

- **Max traversal depth:** 10 hops
- **Max result set:** 100,000 objects
- **Query timeout:** 30 seconds

---

## 13. Future Extensions

### 13.1 Full-Text Search

```lql
# Search content (future)
content:"machine learning"
```

### 13.2 Aggregation

```lql
# Count by type (future)
GROUP BY type
```

### 13.3 Pagination

```lql
# Pagination (future)
OFFSET 100 LIMIT 50
```

### 13.4 Semantic Search

```lql
# Vector similarity (future)
similar(ref:01934e3a...) threshold=0.8
```

---

## 14. Security Considerations

### 14.1 Query Injection

LQL does NOT support string interpolation. Queries MUST be parameterized:

```rust
// Good: Parameterized
let query = Query::parse("tag:$1 AND type:$2")?;
query.bind(1, user_input)?;

// Bad: Concatenation (vulnerable)
let query = format!("tag:{} AND type:pdf", user_input);
```

### 14.2 Resource Exhaustion

**Mitigations:**

- Query timeouts (30s)
- Max traversal depth (10 hops)
- Max result set size (100k objects)
- Rate limiting (1000 queries/min)

### 14.3 Information Disclosure

Queries MUST respect capability-based access:

- Only query objects user has READ capability for
- Explain results MUST NOT leak unauthorized object metadata

---

## 15. Conformance

### 15.1 Test Suite

Implementations MUST pass the LQL test suite (Appendix A).

### 15.2 Parser Implementation

Implementations SHOULD use:

- Recursive descent parser (hand-written)
- Pratt parser (operator precedence)

Implementations MUST NOT:

- Use regex-based parsing (not composable)
- Allow arbitrary code execution

---

## Appendix A: Test Vectors

### A.1 Basic Predicates

```yaml
- query: "tag:project:phoenix"
  matches: [obj1, obj3]

- query: "type:application/pdf"
  matches: [obj2, obj4]

- query: "state:approved"
  matches: [obj1, obj2, obj5]
```

### A.2 Boolean Logic

```yaml
- query: "tag:project:phoenix AND type:pdf"
  matches: [obj1]

- query: "tag:urgent OR tag:priority:high"
  matches: [obj2, obj3, obj6]

- query: "NOT state:archived"
  matches: [obj1, obj2, obj3, obj4]
```

### A.3 Precedence

```yaml
- query: "A OR B AND C"
  ast: OR(A, AND(B, C))

- query: "(A OR B) AND C"
  ast: AND(OR(A, B), C)
```

---

## Appendix B: Grammar Railroad Diagram

```
query:  ─┬─ expr ─┬─┬─ SORT ... ─┬─┬─ LIMIT ... ─┬─
         └─────────┘ └────────────┘ └─────────────┘

expr:   ─┬─ term ─┬─┬─ AND ─┬─ term ─┬─
         └────────┘ └─ OR ──┘         └─

term:   ─┬─ predicate ─┬─
         ├─ NOT ─ term ─┤
         └─ ( expr ) ───┘
```

---

## Appendix C: MIME Type Reference

Common MIME types for `type:` predicate:

```
text/plain
text/markdown
text/html
application/pdf
application/json
application/xml
image/jpeg
image/png
image/gif
video/mp4
audio/mpeg
```

Wildcards: `type:text/*`, `type:image/*`, etc.

---

**End of LFS-002**
