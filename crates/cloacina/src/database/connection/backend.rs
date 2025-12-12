/*
 *  Copyright 2025 Colliery Software
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

//! Database backend types and runtime backend selection.

#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("cloacina requires either the \"postgres\" or \"sqlite\" feature to be enabled");

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use deadpool_diesel::postgres::Manager as PgManager;
#[cfg(feature = "postgres")]
use deadpool_diesel::postgres::Pool as PgPool;
#[cfg(feature = "postgres")]
use diesel::PgConnection;

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use deadpool_diesel::sqlite::Manager as SqliteManager;
#[cfg(feature = "sqlite")]
use deadpool_diesel::sqlite::Pool as SqlitePool;
#[cfg(feature = "sqlite")]
use diesel::SqliteConnection;

// =============================================================================
// Runtime Database Backend Selection
// =============================================================================

/// Represents the database backend type, detected at runtime from the connection URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// PostgreSQL backend
    #[cfg(feature = "postgres")]
    Postgres,
    /// SQLite backend
    #[cfg(feature = "sqlite")]
    Sqlite,
}

impl BackendType {
    /// Detect the backend type from a connection URL.
    ///
    /// # Arguments
    /// * `url` - The database connection URL
    ///
    /// # Returns
    /// The detected `BackendType`
    ///
    /// # Panics
    /// Panics if the URL scheme doesn't match any enabled backend.
    pub fn from_url(url: &str) -> Self {
        #[cfg(feature = "postgres")]
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            return BackendType::Postgres;
        }

        // SQLite URLs can be:
        // - sqlite:// prefix
        // - file: URI format (e.g., file:test?mode=memory&cache=shared)
        // - file paths (relative or absolute)
        // - :memory: for in-memory databases
        #[cfg(feature = "sqlite")]
        if url.starts_with("sqlite://")
            || url.starts_with("file:")
            || url.starts_with("/")
            || url.starts_with("./")
            || url.starts_with("../")
            || url == ":memory:"
            || url.ends_with(".db")
            || url.ends_with(".sqlite")
            || url.ends_with(".sqlite3")
        {
            return BackendType::Sqlite;
        }

        panic!(
            "Unable to detect database backend from URL '{}'. \
             Expected postgres://, postgresql://, sqlite://, or a file path.",
            url
        );
    }
}

/// Multi-connection enum that wraps both PostgreSQL and SQLite connections.
///
/// This enum enables runtime database backend selection using Diesel's
/// `MultiConnection` derive macro. The actual connection type is determined
/// at runtime based on the connection URL.
#[cfg(all(feature = "postgres", feature = "sqlite"))]
#[derive(diesel::MultiConnection)]
pub enum AnyConnection {
    /// PostgreSQL connection variant
    Postgres(PgConnection),
    /// SQLite connection variant
    Sqlite(SqliteConnection),
}

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
pub enum AnyConnection {
    /// PostgreSQL connection variant
    Postgres(PgConnection),
}

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub enum AnyConnection {
    /// SQLite connection variant
    Sqlite(SqliteConnection),
}

/// Pool enum that wraps both PostgreSQL and SQLite connection pools.
///
/// This enum enables runtime pool selection based on the detected backend.
#[derive(Clone)]
pub enum AnyPool {
    /// PostgreSQL connection pool
    #[cfg(feature = "postgres")]
    Postgres(PgPool),
    /// SQLite connection pool
    #[cfg(feature = "sqlite")]
    Sqlite(SqlitePool),
}

impl std::fmt::Debug for AnyPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "postgres")]
            AnyPool::Postgres(_) => write!(f, "AnyPool::Postgres(...)"),
            #[cfg(feature = "sqlite")]
            AnyPool::Sqlite(_) => write!(f, "AnyPool::Sqlite(...)"),
        }
    }
}

impl AnyPool {
    /// Returns a reference to the PostgreSQL pool if this is a PostgreSQL backend.
    #[cfg(feature = "postgres")]
    pub fn as_postgres(&self) -> Option<&PgPool> {
        match self {
            AnyPool::Postgres(pool) => Some(pool),
            #[cfg(feature = "sqlite")]
            _ => None,
        }
    }

    /// Returns a reference to the SQLite pool if this is a SQLite backend.
    #[cfg(feature = "sqlite")]
    pub fn as_sqlite(&self) -> Option<&SqlitePool> {
        match self {
            AnyPool::Sqlite(pool) => Some(pool),
            #[cfg(feature = "postgres")]
            _ => None,
        }
    }

    /// Returns the PostgreSQL pool, panicking if this is not a PostgreSQL backend.
    #[cfg(feature = "postgres")]
    pub fn expect_postgres(&self) -> &PgPool {
        match self {
            AnyPool::Postgres(pool) => pool,
            #[cfg(feature = "sqlite")]
            _ => panic!("Expected PostgreSQL pool but got SQLite"),
        }
    }

    /// Returns the SQLite pool, panicking if this is not a SQLite backend.
    #[cfg(feature = "sqlite")]
    pub fn expect_sqlite(&self) -> &SqlitePool {
        match self {
            AnyPool::Sqlite(pool) => pool,
            #[cfg(feature = "postgres")]
            _ => panic!("Expected SQLite pool but got PostgreSQL"),
        }
    }
}

// =============================================================================
// Legacy Type Aliases (for backward compatibility during migration)
// =============================================================================
// Note: With dual-backend support, use AnyConnection and AnyPool instead.
// These aliases default to PostgreSQL for backwards compatibility.

/// Type alias for the connection type (defaults to PostgreSQL)
#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
pub type DbConnection = PgConnection;

/// Type alias for the connection manager (defaults to PostgreSQL)
#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
pub type DbConnectionManager = PgManager;

/// Type alias for the connection pool (defaults to PostgreSQL)
#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
pub type DbPool = PgPool;

/// Type alias for the connection type (sqlite-only builds)
#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub type DbConnection = SqliteConnection;

/// Type alias for the connection manager (sqlite-only builds)
#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub type DbConnectionManager = SqliteManager;

/// Type alias for the connection pool (sqlite-only builds)
#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub type DbPool = SqlitePool;
