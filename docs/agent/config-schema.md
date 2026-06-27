# Configuration Schema (Auto-generated)

```rust
pub struct DecapodProjectConfig {
    pub schema_version: String,
    pub init: InitConfigSection,
    pub repo: RepoContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud: Option<CloudConfigSection>,
}
```
