# Localization (i18n) Implementation

## Overview

LatticeFS now supports localized view names and descriptions with automatic operating system locale detection. The implementation defaults to **German** for all non-English locales.

## Features

### Supported Locales

- **English** (`Locale::English`): Standard English translations
- **German** (`Locale::German`): German translations (default fallback)

### Automatic Detection

The system automatically detects the OS locale using the `sys-locale` crate:

```rust
use latticefs_base::views::Locale;

let locale = Locale::from_system();
// Returns Locale::English if OS locale starts with "en"
// Otherwise returns Locale::German (fallback)
```

### Localized Built-in Views

All built-in views now have localized names and descriptions:

| View | English | German |
|------|---------|--------|
| Recent | Recent | Neueste |
| Projects | Projects | Projekte |
| Drafts | Drafts | Entwürfe |
| Review | Pending Review | Zur Prüfung |
| Approved | Approved | Genehmigt |
| All | All Objects | Alle Objekte |

## API Usage

### Basic Usage

```rust
use latticefs_base::views::{BuiltinView, Locale};

// Get localized name
let name = BuiltinView::Recent.name_localized(Locale::German);
// Returns: "Neueste"

// Get localized description
let desc = BuiltinView::Recent.description_localized(Locale::German);
// Returns: "Objekte, die in den letzten 7 Tagen aktualisiert wurden"
```

### With System Locale

```rust
use latticefs_base::views::{BuiltinView, Locale};

let locale = Locale::from_system();

for view in BuiltinView::all() {
    println!("{}: {}", 
             view.name_localized(locale),
             view.description_localized(locale));
}
```

### Backward Compatibility

The original `name()` and `description()` methods remain unchanged and return English strings:

```rust
// Old code still works
let name = BuiltinView::Recent.name(); // "Recent"
let desc = BuiltinView::Recent.description(); // "Objects updated within the last 7 days"
```

## Name Resolution

The `by_name()` method now accepts both English and German view names:

```rust
use latticefs_base::views::BuiltinView;

// English names
assert_eq!(BuiltinView::by_name("recent"), Some(BuiltinView::Recent));
assert_eq!(BuiltinView::by_name("projects"), Some(BuiltinView::Projects));

// German names
assert_eq!(BuiltinView::by_name("neueste"), Some(BuiltinView::Recent));
assert_eq!(BuiltinView::by_name("projekte"), Some(BuiltinView::Projects));
```

## Implementation Details

### Files Modified

1. **base/Cargo.toml**: Added `sys-locale = "0.3"` dependency
2. **base/src/views/builtin.rs**: Added `Locale` enum and localized methods
3. **base/src/views/mod.rs**: Exported `Locale` type
4. **cli/src/commands/stats.rs**: Updated to use localized names
5. **cli/src/commands/view.rs**: Updated to use localized names
6. **gui/src-tauri/src/commands.rs**: Updated to use localized names

### Design Decisions

1. **German as Fallback**: All non-English locales default to German
2. **Static Strings**: Uses `&'static str` for zero-cost localization
3. **Compile-time**: All translations are embedded at compile time
4. **Backward Compatible**: Existing code continues to work without changes

## Testing

Run the tests:

```bash
cargo test -p base --lib views::builtin::tests
```

Run the demonstration:

```bash
cargo run --example locale_demo -p base
```

## Future Enhancements

To add more locales:

1. Add a new variant to the `Locale` enum
2. Update the `from_system()` method to detect the new locale
3. Add translations in `name_localized()` and `description_localized()`
4. Add name mappings in `by_name()`

Example for French:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    English,
    German,
    French,  // New locale
}

impl Locale {
    pub fn from_system() -> Self {
        sys_locale::get_locale()
            .map(|locale| {
                if locale.starts_with("en") {
                    Locale::English
                } else if locale.starts_with("fr") {
                    Locale::French
                } else {
                    Locale::German  // fallback
                }
            })
            .unwrap_or(Locale::German)
    }
}

// Add French translations to name_localized() and description_localized()
```

## CLI Examples

With German locale detected:

```bash
$ lfs view list
Built-in views:
- Neueste: Objekte, die in den letzten 7 Tagen aktualisiert wurden
- Projekte: Objekte, die als Projekte gekennzeichnet sind
- Entwürfe: Objekte im Entwurfsstadium
- Zur Prüfung: Objekte, die auf Prüfung warten
- Genehmigt: Genehmigte Objekte
- Alle Objekte: Alle Objekte im Repository
```

With English locale:

```bash
$ lfs view list
Built-in views:
- Recent: Objects updated within the last 7 days
- Projects: Objects tagged as projects
- Drafts: Objects in draft state
- Pending Review: Objects pending review
- Approved: Approved objects
- All Objects: All objects in the repository
```

## GUI Integration

The Tauri GUI automatically uses the system locale for view names and descriptions in the `list_views()` command, providing a localized experience to users without any configuration needed.
