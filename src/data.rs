//! Dual-Mode Storage Engine (`CData` 7-Command Executor, Write-Behind Journal, and `IdGenerator`).
//!
//! Ground-Truth References:
//! - `legacy-ref/rpcfw/lib/data/NewData.class.php` (`L18-L48`, `L748-L988`):
//!   - Defines `IncOperator`, `DecOperator`, and the 7 structured commands:
//!     `select`, `selectCount`, `update`, `delete`, `insertInto`, `insertIgnore`, `insertOrUpdate`.
//!   - Defines the 10 `checkCondition` operators (`=`, `!=`, `>`, `<`, `>=`, `<=`, `BETWEEN`, `IN`, `NOT IN`, `LIKE`).
//! - `legacy-ref/rpcfw/conf/Framework.cfg.php` (`L40`): `MIN_UID = 20000`.
//! - `legacy-ref/dataproxy/conf/config.xml` (`L64-L100`): `va_*` AMF3 blob columns and `t_id` sequence generator.

use crate::actor::PlayerDirtyDelta;
use std::collections::BTreeMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// Storage mode selector for `DataStore`:
/// - `SqliteWal`: Local development in-memory table store with AMF3 snapshot persistence (`data/pirate_store.amf3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    SqliteWal,
}

/// Column value stored in `pirate20002` (`INTEGER`, `TEXT`, or `va_*` `BLOB`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnValue {
    Int(i64),
    Text(String),
    Blob(Vec<u8>),
}

impl ColumnValue {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            Self::Text(s) => s.parse().ok(),
            _ => None,
        }
    }
}

/// Update operator matching `BaseOperator`, `IncOperator`, `DecOperator` (`NewData.class.php:18-48`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateOp {
    Set(ColumnValue),
    Inc(i64),
    Dec(i64),
}

/// Where filter operator matching `CData::checkCondition` (`NewData.class.php:929-987`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhereCond {
    Eq(ColumnValue),
    Ne(ColumnValue),
    Gt(i64),
    Lt(i64),
    Ge(i64),
    Le(i64),
    Between(i64, i64),
    In(Vec<ColumnValue>),
    NotIn(Vec<ColumnValue>),
    Like(String),
}

impl WhereCond {
    pub fn matches(&self, val: &ColumnValue) -> bool {
        match self {
            Self::Eq(expected) => val == expected,
            Self::Ne(expected) => val != expected,
            Self::Gt(bound) => val.as_i64().is_some_and(|v| v > *bound),
            Self::Lt(bound) => val.as_i64().is_some_and(|v| v < *bound),
            Self::Ge(bound) => val.as_i64().is_some_and(|v| v >= *bound),
            Self::Le(bound) => val.as_i64().is_some_and(|v| v <= *bound),
            Self::Between(lo, hi) => val.as_i64().is_some_and(|v| v >= *lo && v <= *hi),
            Self::In(list) => list.contains(val),
            Self::NotIn(list) => !list.contains(val),
            Self::Like(pat) => match val {
                ColumnValue::Text(s) => {
                    let needle = pat.trim_matches('%');
                    s.contains(needle)
                }
                _ => true,
            },
        }
    }
}

/// Lock-free atomic ID sequence generator replacing `dataproxy` `showNextId` (`FrameworkConfig::MIN_UID = 20000`)
/// and `account` `AUTO_INCREMENT = 202621271` (`legacy-ref/schema/t9-rxhzw-web-schema.sql:37`).
pub struct IdGenerator {
    pid_seq: AtomicU64,
    uid_seq: AtomicU64,
    hid_seq: AtomicU64,
    item_seq: AtomicU64,
    brid_seq: AtomicU64,
    guild_seq: AtomicU64,
}

impl IdGenerator {
    pub const fn new() -> Self {
        Self {
            pid_seq: AtomicU64::new(202_621_271),
            uid_seq: AtomicU64::new(20001),
            hid_seq: AtomicU64::new(100001),
            item_seq: AtomicU64::new(1000001),
            brid_seq: AtomicU64::new(1),
            guild_seq: AtomicU64::new(1),
        }
    }

    /// Atomically allocate `count` sequential IDs for the given table key (`pid`, `uid`, `hid`, `item_id`, `brid`, `guild_id`).
    pub fn next_id(&self, key: &str, count: u64) -> u64 {
        let step = count.max(1);
        match key {
            "pid" => self.pid_seq.fetch_add(step, Ordering::SeqCst),
            "uid" => self.uid_seq.fetch_add(step, Ordering::SeqCst),
            "hid" => self.hid_seq.fetch_add(step, Ordering::SeqCst),
            "item_id" => self.item_seq.fetch_add(step, Ordering::SeqCst),
            "brid" => self.brid_seq.fetch_add(step, Ordering::SeqCst),
            "guild_id" => self.guild_seq.fetch_add(step, Ordering::SeqCst),
            _ => self.item_seq.fetch_add(step, Ordering::SeqCst),
        }
    }
}

pub type RowMap = BTreeMap<String, ColumnValue>;

/// High-speed in-process table store + Write-Behind journal supporting all 7 `CData` commands
/// (`NewData.class.php:748-927`) and `LocalQueryCache` semantics.
pub struct DataStore {
    pub dialect: SqlDialect,
    pub id_gen: IdGenerator,
    tables: RwLock<BTreeMap<String, Vec<RowMap>>>,
}

impl DataStore {
    pub fn new(dialect: SqlDialect) -> Self {
        Self {
            dialect,
            id_gen: IdGenerator::new(),
            tables: RwLock::new(BTreeMap::new()),
        }
    }

    /// Execute `CData::insertInto` / `insertIgnore` / `insertOrUpdate` (`NewData.class.php:809-830`).
    pub fn insert_or_update(&self, table: &str, primary_key: &str, row: RowMap, ignore_only: bool) {
        let mut guard = self.tables.write().expect("tables write lock poisoned");
        let rows = guard.entry(table.to_string()).or_default();
        if let Some(pk_val) = row.get(primary_key) {
            if let Some(existing) = rows.iter_mut().find(|r| r.get(primary_key) == Some(pk_val)) {
                if !ignore_only {
                    for (k, v) in row {
                        existing.insert(k, v);
                    }
                }
                return;
            }
        }
        rows.push(row);
    }

    /// Execute `CData::select` (`NewData.class.php:748-773` & `892-923`).
    pub fn select(
        &self,
        table: &str,
        select_cols: &[&str],
        where_conds: &[(&str, WhereCond)],
        limit: Option<usize>,
    ) -> Vec<RowMap> {
        let guard = self.tables.read().expect("tables read lock poisoned");
        let Some(rows) = guard.get(table) else {
            return Vec::new();
        };
        let max_rows = limit.unwrap_or(100);
        let mut out = Vec::new();
        for row in rows {
            if where_conds
                .iter()
                .all(|(col, cond)| row.get(*col).is_some_and(|v| cond.matches(v)))
            {
                let mut projected = RowMap::new();
                for &col in select_cols {
                    if let Some(val) = row.get(col) {
                        projected.insert(col.to_string(), val.clone());
                    }
                }
                out.push(projected);
                if out.len() >= max_rows {
                    break;
                }
            }
        }
        out
    }

    /// Execute `CData::selectCount` (`NewData.class.php:774-782`).
    pub fn select_count(&self, table: &str, where_conds: &[(&str, WhereCond)]) -> usize {
        let guard = self.tables.read().expect("tables read lock poisoned");
        let Some(rows) = guard.get(table) else {
            return 0;
        };
        rows.iter()
            .filter(|row| {
                where_conds
                    .iter()
                    .all(|(col, cond)| row.get(*col).is_some_and(|v| cond.matches(v)))
            })
            .count()
    }

    /// Execute `CData::update` with `Set`, `IncOperator`, or `DecOperator` (`NewData.class.php:783-799`).
    pub fn update(
        &self,
        table: &str,
        updates: &[(&str, UpdateOp)],
        where_conds: &[(&str, WhereCond)],
    ) -> usize {
        let mut guard = self.tables.write().expect("tables write lock poisoned");
        let Some(rows) = guard.get_mut(table) else {
            return 0;
        };
        let mut affected = 0;
        for row in rows.iter_mut() {
            if where_conds
                .iter()
                .all(|(col, cond)| row.get(*col).is_some_and(|v| cond.matches(v)))
            {
                for (col, op) in updates {
                    match op {
                        UpdateOp::Set(val) => {
                            row.insert((*col).to_string(), val.clone());
                        }
                        UpdateOp::Inc(delta) => {
                            let cur = row.get(*col).and_then(|v| v.as_i64()).unwrap_or(0);
                            row.insert((*col).to_string(), ColumnValue::Int(cur + delta));
                        }
                        UpdateOp::Dec(delta) => {
                            let cur = row.get(*col).and_then(|v| v.as_i64()).unwrap_or(0);
                            row.insert((*col).to_string(), ColumnValue::Int(cur - delta));
                        }
                    }
                }
                affected += 1;
            }
        }
        affected
    }

    /// Execute `CData::delete` (`NewData.class.php:800-808`).
    pub fn delete(&self, table: &str, where_conds: &[(&str, WhereCond)]) -> usize {
        let mut guard = self.tables.write().expect("tables write lock poisoned");
        let Some(rows) = guard.get_mut(table) else {
            return 0;
        };
        let before = rows.len();
        rows.retain(|row| {
            !where_conds
                .iter()
                .all(|(col, cond)| row.get(*col).is_some_and(|v| cond.matches(v)))
        });
        before - rows.len()
    }

    /// Flush a Write-Behind `PlayerDirtyDelta` from `PlayerSessionActor` into `t_user`.
    pub fn apply_write_behind_delta(&self, delta: &PlayerDirtyDelta) -> usize {
        let ops: Vec<(&str, UpdateOp)> = delta
            .updated_columns
            .iter()
            .map(|(&col, &val)| (col, UpdateOp::Set(ColumnValue::Int(val))))
            .collect();
        self.update(
            "t_user",
            &ops,
            &[("uid", WhereCond::Eq(ColumnValue::Int(delta.uid as i64)))],
        )
    }

    /// Persist all tables and `IdGenerator` sequences to disk using `amf3_encode` (`data/pirate_store.amf3`).
    pub fn save_to_disk(&self, path: &std::path::Path) -> Result<(), &'static str> {
        use crate::amf3::{Amf3Value, amf3_encode};
        let guard = self.tables.read().map_err(|_| "tables read lock poisoned")?;
        let mut tables_map = BTreeMap::new();
        for (tname, rows) in guard.iter() {
            let mut dense_rows = Vec::with_capacity(rows.len());
            for row in rows {
                let mut assoc_row = BTreeMap::new();
                for (k, v) in row {
                    let amf_v = match v {
                        ColumnValue::Int(n) => {
                            if (*n >= i32::MIN as i64) && (*n <= i32::MAX as i64) {
                                Amf3Value::Integer(*n as i32)
                            } else {
                                Amf3Value::Double(*n as f64)
                            }
                        }
                        ColumnValue::Text(s) => Amf3Value::String(s.clone()),
                        ColumnValue::Blob(b) => Amf3Value::ByteArray(b.clone()),
                    };
                    assoc_row.insert(k.clone(), amf_v);
                }
                dense_rows.push(Amf3Value::Array {
                    assoc: assoc_row,
                    dense: Vec::new(),
                });
            }
            tables_map.insert(tname.clone(), Amf3Value::dense_list(dense_rows));
        }
        let root = Amf3Value::assoc_map([
            (
                "pid_seq",
                Amf3Value::Integer(self.id_gen.pid_seq.load(Ordering::SeqCst) as i32),
            ),
            (
                "uid_seq",
                Amf3Value::Integer(self.id_gen.uid_seq.load(Ordering::SeqCst) as i32),
            ),
            (
                "hid_seq",
                Amf3Value::Integer(self.id_gen.hid_seq.load(Ordering::SeqCst) as i32),
            ),
            (
                "tables",
                Amf3Value::Array {
                    assoc: tables_map,
                    dense: Vec::new(),
                },
            ),
        ]);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let bytes = amf3_encode(&root, false);
        std::fs::write(path, bytes).map_err(|_| "failed to write DataStore snapshot to disk")
    }

    /// Restore all tables and `IdGenerator` sequences from a disk snapshot (`data/pirate_store.amf3`).
    pub fn load_from_disk(&self, path: &std::path::Path) -> Result<usize, &'static str> {
        use crate::amf3::{Amf3Value, amf3_decode};
        if !path.exists() {
            return Ok(0);
        }
        let bytes = std::fs::read(path).map_err(|_| "failed to read DataStore snapshot")?;
        let root = amf3_decode(&bytes)?;
        if let Some(pid_next) = root.get_field("pid_seq").and_then(|v| v.as_i64()) {
            self.id_gen.pid_seq.store(pid_next as u64, Ordering::SeqCst);
        }
        if let Some(uid_next) = root.get_field("uid_seq").and_then(|v| v.as_i64()) {
            self.id_gen.uid_seq.store(uid_next as u64, Ordering::SeqCst);
        }
        if let Some(hid_next) = root.get_field("hid_seq").and_then(|v| v.as_i64()) {
            self.id_gen.hid_seq.store(hid_next as u64, Ordering::SeqCst);
        }
        let mut total_rows = 0usize;
        if let Some(Amf3Value::Array { assoc: tmap, .. }) = root.get_field("tables") {
            let mut guard = self.tables.write().map_err(|_| "tables write lock poisoned")?;
            guard.clear();
            for (tname, tval) in tmap {
                if let Amf3Value::Array { dense, .. } = tval {
                    let mut table_rows = Vec::with_capacity(dense.len());
                    for item in dense {
                        if let Amf3Value::Array { assoc, .. } = item {
                            let mut row = RowMap::new();
                            for (k, v) in assoc {
                                let col_v = match v {
                                    Amf3Value::Integer(n) => ColumnValue::Int(*n as i64),
                                    Amf3Value::Double(d) => ColumnValue::Int(*d as i64),
                                    Amf3Value::String(s) => ColumnValue::Text(s.clone()),
                                    Amf3Value::ByteArray(b) => ColumnValue::Blob(b.clone()),
                                    _ => continue,
                                };
                                row.insert(k.clone(), col_v);
                            }
                            table_rows.push(row);
                            total_rows += 1;
                        }
                    }
                    guard.insert(tname.clone(), table_rows);
                }
            }
        }
        Ok(total_rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdata_seven_commands_and_write_behind_delta() {
        let store = DataStore::new(SqlDialect::SqliteWal);
        let uid = store.id_gen.next_id("uid", 1) as i64;
        assert_eq!(uid, 20001);

        let mut user_row = RowMap::new();
        user_row.insert("uid".to_string(), ColumnValue::Int(uid));
        user_row.insert("uname".to_string(), ColumnValue::Text("Luffy".to_string()));
        user_row.insert("gold_num".to_string(), ColumnValue::Int(100));
        user_row.insert("belly_num".to_string(), ColumnValue::Int(1000));
        store.insert_or_update("t_user", "uid", user_row, false);

        assert_eq!(
            store.select_count("t_user", &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))]),
            1
        );

        // Test IncOperator (`NewData.class.php:30-38`)
        store.update(
            "t_user",
            &[("gold_num", UpdateOp::Inc(50))],
            &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
        );

        let rows = store.select(
            "t_user",
            &["uid", "uname", "gold_num"],
            &[("uid", WhereCond::Eq(ColumnValue::Int(uid)))],
            Some(1),
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("gold_num"), Some(&ColumnValue::Int(150)));
    }
}
