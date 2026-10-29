//! The shared shape of every status lifecycle (Thesis, Drop, Style,
//! Documentary, Production Order).
//!
//! Each lifecycle declares its enum with [`status_enum!`], which adds the
//! parts that are the same everywhere: `ALL` in lifecycle order, `label`,
//! `as_db`, `from_db`, `moves`, `Display`, and the [`Lifecycle`] trait. The
//! rules stay in the area's own file: each enum writes its `move_allowed`
//! there by hand, so its allowed moves read in one place.
//!
//! [`move_status`] is the common move step: check the lifecycle, then write
//! the new status. Areas with guards or cascades call [`check_move`] and do
//! their own write.

use rusqlite::params;

use crate::error::{CoreError, CoreResult};
use crate::library::Library;

/// What the shared move step needs from a status enum. [`status_enum!`]
/// implements it; areas do not implement it by hand.
pub(crate) trait Lifecycle: Copy {
    /// The record the status belongs to, for example "Style".
    const RECORD: &'static str;
    fn allows(self, to: Self) -> bool;
    fn name(self) -> &'static str;
    fn db_name(self) -> &'static str;
}

/// Refuses with `StatusMoveRefused` when the lifecycle does not allow the
/// move from `from` to `to`.
pub(crate) fn check_move<S: Lifecycle>(from: S, to: S) -> CoreResult<()> {
    if from.allows(to) {
        Ok(())
    } else {
        Err(CoreError::StatusMoveRefused { what: S::RECORD, from: from.name(), to: to.name() })
    }
}

/// Moves the row `id` of `table` from status `from` to `to`, after
/// [`check_move`]. `table` is always a table name from the code, with a
/// `status` column.
pub(crate) fn move_status<S: Lifecycle>(lib: &Library, table: &'static str, id: i64, from: S, to: S) -> CoreResult<()> {
    check_move(from, to)?;
    lib.writer()?.execute(&format!("UPDATE {table} SET status = ?1 WHERE id = ?2"), params![to.db_name(), id])?;
    Ok(())
}

/// Declares a status enum: each variant with its founder-facing label and
/// its database name. The enum must also have an inherent
/// `fn move_allowed(self, to: Self) -> bool`, written by hand next to it.
///
/// ```ignore
/// status_enum! {
///     /// Style status.
///     pub enum StyleStatus for "Style" {
///         Draft = ("Draft", "draft"),
///         Approved = ("Approved", "approved"),
///     }
/// }
/// ```
macro_rules! status_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident for $record:literal {
            $($variant:ident = ($label:literal, $db:literal)),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            /// Every status, in lifecycle order.
            pub const ALL: [$name; [$(stringify!($variant)),+].len()] = [$($name::$variant),+];

            /// Every status the lifecycle lets the record move to from `self`.
            pub fn moves(self) -> Vec<$name> {
                $name::ALL.into_iter().filter(|to| self.move_allowed(*to)).collect()
            }

            /// The founder-facing name.
            pub fn label(self) -> &'static str {
                match self {
                    $($name::$variant => $label),+
                }
            }

            pub(crate) fn as_db(self) -> &'static str {
                match self {
                    $($name::$variant => $db),+
                }
            }

            pub(crate) fn from_db(s: &str) -> Option<$name> {
                $name::ALL.into_iter().find(|st| st.as_db() == s)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.label())
            }
        }

        impl $crate::lifecycle::Lifecycle for $name {
            const RECORD: &'static str = $record;
            fn allows(self, to: Self) -> bool {
                self.move_allowed(to)
            }
            fn name(self) -> &'static str {
                self.label()
            }
            fn db_name(self) -> &'static str {
                self.as_db()
            }
        }
    };
}

pub(crate) use status_enum;
