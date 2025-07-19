# DeltaOps (deltalake)

High level interface for executing commands against a DeltaTable.

---

## Struct Definition

```rust
pub struct DeltaOps(pub DeltaTable);
```

The deltalake crate is currently just a meta-package shim for deltalake-core.

---

## Tuple Fields
- `0: DeltaTable`

---

## Implementations

### Constructors

#### `pub async fn try_from_uri(uri: impl AsRef<str>) -> Result<DeltaOps, DeltaTableError>`
Create a new DeltaOps instance, operating on DeltaTable at given uri.

```rust
use deltalake_core::DeltaOps;

async {
    let ops = DeltaOps::try_from_uri("memory:///").await.unwrap();
};
```

#### `pub async fn try_from_uri_with_storage_options(uri: impl AsRef<str>, storage_options: HashMap<String, String>) -> Result<DeltaOps, DeltaTableError>`
Try from uri with storage options.

#### `pub fn new_in_memory() -> DeltaOps`
Create a new DeltaOps instance, backed by an un-initialized in memory table. Useful for testing.

```rust
use deltalake_core::DeltaOps;

let ops = DeltaOps::new_in_memory();
```

---

### Table Operations

- `create(self) -> CreateBuilder`  
  Create a new Delta table.
  
  ```rust
  use deltalake_core::DeltaOps;
  
  async {
      let ops = DeltaOps::try_from_uri("memory:///").await.unwrap();
      let table = ops.create().with_table_name("my_table").await.unwrap();
      assert_eq!(table.version(), Some(0));
  };
  ```

- `load(self) -> LoadBuilder`  
  Load data from a DeltaTable.

- `load_cdf(self) -> CdfLoadBuilder`  
  Load a table with CDF Enabled.

- `write(self, batches: impl IntoIterator<Item = RecordBatch>) -> WriteBuilder`  
  Write data to Delta table.

- `vacuum(self) -> VacuumBuilder`  
  Vacuum stale files from delta table.

- `filesystem_check(self) -> FileSystemCheckBuilder`  
  Audit active files with files present on the filesystem.

- `optimize<'a>(self) -> OptimizeBuilder<'a>`  
  Optimize table files.

- `delete(self) -> DeleteBuilder`  
  Delete data from Delta table.

- `update(self) -> UpdateBuilder`  
  Update data from Delta table.

- `restore(self) -> RestoreBuilder`  
  Restore delta table to a specified version or datetime.

- `merge<E>(self, source: DataFrame, predicate: E) -> MergeBuilder where E: Into<Expression>`  
  Merge data into Delta table.

- `add_constraint(self) -> ConstraintBuilder`  
  Add a check constraint to a table.

- `add_feature(self) -> AddTableFeatureBuilder`  
  Enable a table feature for a table.

- `drop_constraints(self) -> DropConstraintBuilder`  
  Drops constraints from a table.

- `set_tbl_properties(self) -> SetTablePropertiesBuilder`  
  Set table properties.

- `add_columns(self) -> AddColumnBuilder`  
  Add new columns.

- `update_field_metadata(self) -> UpdateFieldMetadataBuilder`  
  Update field metadata.

- `update_table_metadata(self) -> UpdateTableMetadataBuilder`  
  Update table metadata.

---

## Trait Implementations

- `AsRef<DeltaTable> for DeltaOps`
- `From<DeltaOps> for DeltaTable`
- `From<DeltaTable> for DeltaOps`

---

## See Also
- [DeltaTable](https://docs.rs/deltalake/latest/deltalake/struct.DeltaTable.html)
- [deltalake crate docs](https://docs.rs/deltalake/latest/deltalake/index.html)
- [Source code](https://docs.rs/deltalake-core/0.27.0/x86_64-unknown-linux-gnu/src/deltalake_core/operations/mod.rs.html)
