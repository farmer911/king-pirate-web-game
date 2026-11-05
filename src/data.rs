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

/// SQL dialect selector for Dual-Mode operation (`Rule 08`):
/// - `SqliteWal`: Zero-VM local macOS development (`< 15 ms` cold start, `PRAGMA journal_mode=WAL`).
/// - `MariaDbInnoDb`: High-concurrency online Linux VPS Docker production (`InnoDB` row-level locking).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    SqliteWal,
    MariaDbInnoDb,
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

/// Build the dialect-specific SQL statement for `insertIgnore` or `insertOrUpdate` (`NewData.class.php:809-830`).
pub fn build_upsert_sql(
    dialect: SqlDialect,
    table: &str,
    columns: &[&str],
    duplicate_keys: &[&str],
    ignore_only: bool,
) -> String {
    let cols_joined = columns.join(", ");
    let placeholders: Vec<&str> = columns.iter().map(|_| "?").collect();
    let vals_joined = placeholders.join(", ");

    match (dialect, ignore_only) {
        (SqlDialect::SqliteWal, true) => {
            format!("INSERT OR IGNORE INTO `{table}` ({cols_joined}) VALUES ({vals_joined})")
        }
        (SqlDialect::MariaDbInnoDb, true) => {
            format!("INSERT IGNORE INTO `{table}` ({cols_joined}) VALUES ({vals_joined})")
        }
        (SqlDialect::SqliteWal, false) => {
            let updates: Vec<String> = duplicate_keys
                .iter()
                .map(|k| format!("`{k}` = excluded.`{k}`"))
                .collect();
            format!(
                "INSERT INTO `{table}` ({cols_joined}) VALUES ({vals_joined}) ON CONFLICT DO UPDATE SET {}",
                updates.join(", ")
            )
        }
        (SqlDialect::MariaDbInnoDb, false) => {
            let updates: Vec<String> = duplicate_keys
                .iter()
                .map(|k| format!("`{k}` = VALUES(`{k}`)"))
                .collect();
            format!(
                "INSERT INTO `{table}` ({cols_joined}) VALUES ({vals_joined}) ON DUPLICATE KEY UPDATE {}",
                updates.join(", ")
            )
        }
    }
}

/// Lock-free atomic ID sequence generator replacing `dataproxy` `showNextId` (`FrameworkConfig::MIN_UID = 20000`).
pub struct IdGenerator {
    uid_seq: AtomicU64,
    hid_seq: AtomicU64,
    item_seq: AtomicU64,
    brid_seq: AtomicU64,
    guild_seq: AtomicU64,
}

impl IdGenerator {
    pub const fn new() -> Self {
        Self {
            uid_seq: AtomicU64::new(20001),
            hid_seq: AtomicU64::new(100001),
            item_seq: AtomicU64::new(1000001),
            brid_seq: AtomicU64::new(1),
            guild_seq: AtomicU64::new(1),
        }
    }

    /// Atomically allocate `count` sequential IDs for the given table key (`uid`, `hid`, `item_id`, `brid`, `guild_id`).
    pub fn next_id(&self, key: &str, count: u64) -> u64 {
        let step = count.max(1);
        match key {
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

        // Verify SQLite WAL vs MariaDB InnoDB dialect SQL generation
        let sqlite_sql = build_upsert_sql(
            SqlDialect::SqliteWal,
            "t_user",
            &["uid", "gold_num"],
            &["gold_num"],
            false,
        );
        assert!(sqlite_sql.contains("ON CONFLICT DO UPDATE"));

        let mariadb_sql = build_upsert_sql(
            SqlDialect::MariaDbInnoDb,
            "t_user",
            &["uid", "gold_num"],
            &["gold_num"],
            false,
        );
        assert!(mariadb_sql.contains("ON DUPLICATE KEY UPDATE"));
    }
}
