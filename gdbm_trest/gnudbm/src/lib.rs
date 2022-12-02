// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
#![warn(missing_docs)]

//! Gnudbm is an ergonomic, idiomatic wrapper for [gdbm].
//!
//! With built in support for [Serde] and [bincode], It provides fast and easy
//! local key/value storage of any type implementing [`Serialize`].
//!
//! For an overview of available database operations, see the documentation for
//! [`RwHandle`].
//!
//! # Examples
//!
//! Opening or creating a database file with a [`GdbmOpener`]:
//!
//! ```no_run
//! # use std::path::PathBuf;
//! # use gnudbm::*;
//! let db_path = PathBuf::from("path/to/my.db");
//! let mut db = GdbmOpener::new()
//!     .create(true)
//!     // there are two types of database handle;
//!     // the other is instantiated with `GdbmOpener::readonly`
//!     .readwrite(&db_path)
//!     .expect("db creation failed");
//! ```
//!
//! When fetching and storing, your key can be any type that implements
//! `AsRef<[u8]>`; for instance one of `String`/`&str`.
//! Values are any type that implements `Serialize` and `Deserialize`.
//!
//! ```no_run
//! # use gnudbm::*;
//! # let mut db = RwHandle::dummy();
//! // 'db' is a previously configured database
//! db.store("my key", "an important value").unwrap();
//!
//! // fetch returns an Entry, which wraps a pointer to the raw data
//! let entry = db.fetch("my key").unwrap();
//! assert_eq!(entry.as_bytes(), "my key".as_bytes());
//!
//! // the data can be deserialized, borrowing if possible.
//! // The result is bound to the lifetime of the Entry.
//! let as_str: &str = entry.deserialize().unwrap();
//! assert_eq!(as_str, "my key");
//! ```
//!
//! Use a custom type with Serde:
//!
//! ```no_run
//! # #[macro_use] extern crate serde_derive;
//! # extern crate serde;
//! # extern crate gnudbm;
//! # use gnudbm::*;
//! use serde::{Serialize, Deserialize};
//! # fn main() {
//! # let mut db = RwHandle::dummy();
//!
//! #[derive(Serialize, Deserialize)]
//! struct MyStruct<'a> {
//!     name: &'a str,
//!     counts: Vec<u64>,
//! }
//!
//! let name: String = "Colin".into();
//! let value = MyStruct {
//!     name: &name,
//!     counts: vec![4, 2, 0],
//! };
//!
//! // 'db' is a previously configured database
//! db.store("my key", &value).unwrap();
//!
//! let entry = db.fetch("my key").unwrap();
//! let fetched: MyStruct = entry.deserialize().unwrap();
//!
//! assert_eq!(value.name, fetched.name);
//! # }
//! ```
//!
//! [gdbm]: http://puszcza.gnu.org.ua/software/gdbm/
//! [serde]: https://serde.rs
//! [bincode]: https://github.com/TyOverby/bincode
//! [`Serialize`]: https://serde.rs/impl-serialize.html
//! [`RwHandle`]: struct.RwHandle.html
//! [`GdbmOpener`]: struct.GdbmOpener.html
//!

extern crate bincode;
extern crate libc;
extern crate serde;


mod gdbm_sys;
mod error;

use std::ops::Drop;
use std::default::Default;
use std::path::Path;
use std::slice;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::os::raw::c_void as os_c_void;
use std::mem;

use serde::{Deserialize, Serialize};

use error::last_errno;
pub use error::{Error, GdbmError, GdbmResult};

//TODO: use umask
const DEFAULT_MODE: i32 = 0o666;

/// A read/write reference to a gdbm database.
#[derive(Debug)]
pub struct RwHandle {
    handle: gdbm_sys::GDBM_FILE,
}

/// A readonly reference to a gdbm database.
///
/// This type only exposes non-modifying methods, to avoid having to deal with
/// errors around attempting to modify a database opened in read-only mode.
#[derive(Debug)]
pub struct ReadHandle(RwHandle);

/// A builder used to open gdbm files.
#[derive(Debug, Default)]
pub struct GdbmOpener {
    sync: bool,
    no_lock: bool,
    no_mmap: bool,
    create: bool,
    overwrite: bool,
    readonly: bool,
    block_size: i32,
}

/// An entry in a gdbm database.
///
/// This type represents arbitrary data retrieved from the database.
#[derive(Debug)]
pub struct Entry<'a> {
    datum: gdbm_sys::datum,
    slice: &'a [u8],
}

/// A key retrieved from a gdbm database.
///
/// This type is only used as the return value of `RwHandle::iter`. It is
/// distinct from [`Entry`] for the sake of clarity.
#[derive(Debug)]
pub struct Key<'a>(Entry<'a>);

/// An iterator over keys and values in a gdbm database.
#[derive(Debug)]
pub struct Iter<'a> {
    db: &'a RwHandle,
    nxt_key: Option<gdbm_sys::datum>,
}

impl RwHandle {
    /// Inserts a key value pair into the database, replacing any existing
    /// value for that key.
    ///
    /// Returns an [`Error`] if the store fails. The only reason this might happen
    /// is if there is a problem writing to disk, which is likely not recoverable.
    ///
    /// [`Error`]: error/enum.Error.html
    ///
    /// ```no_run
    /// # use gnudbm::*;
    /// # let mut db = RwHandle::dummy();
    /// let key = "my key";
    /// let value = "my value";
    /// db.store(key, value).unwrap();
    /// ```
    pub fn store<K, V>(&mut self, key: K, value: &V) -> GdbmResult<()>
    where
        K: AsRef<[u8]>,
        V: ?Sized + Serialize,
    {
        self.store_impl(key, value, true).map(|_| ())
    }

    /// Inserts a key value pair into the database, failing of the key already
    /// exists.
    ///
    /// Returns an [`Error`] if the store fails, including if it fails because the
    /// key already exists.
    ///
    /// [`Error`]: error/enum.Error.html
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use gnudbm::*;
    /// # let mut db = RwHandle::dummy();
    /// let key = "my key";
    /// let value = "my value";
    /// db.store_checked(key, value).unwrap();
    ///
    /// // second store will fail
    /// assert!(db.store_checked(key, value).is_err());
    /// ```
    pub fn store_checked<K, V>(&mut self, key: K, value: &V) -> GdbmResult<()>
    where
        K: AsRef<[u8]>,
        V: ?Sized + Serialize,
    {
        let r = self.store_impl(key, value, false)?;
        if r == 1 {
            Err(Error::KeyExists)
        } else {
            Ok(())
        }
    }

    fn store_impl<K, V>(&mut self, key: K, value: &V, replace: bool) -> GdbmResult<i32>
    where
        K: AsRef<[u8]>,
        V: ?Sized + Serialize,
    {
        let bytes = bincode::serialize(value)?;
        let key_d: gdbm_sys::datum = key.as_ref().into();

        let value_d = gdbm_sys::datum {
            dptr: bytes.as_ptr() as *mut i8,
            dsize: bytes.len() as i32,
        };

        let flag = if replace {
            gdbm_sys::GDBM_REPLACE
        } else {
            gdbm_sys::GDBM_INSERT
        };

        let result = unsafe { gdbm_sys::gdbm_store(self.handle, key_d, value_d, flag as i32) };

        if result == -1 {
            Err(GdbmError::from_last().into())
        } else {
            Ok(result)
        }
    }

    /// Attempts to fetch an item from the database.
    ///
    /// Returns an [`Entry`] if `key` exists in the database. Returns an
    /// [`Error`] if the key does not exist, or if an error occurs while
    /// reading the database.
    ///
    /// [`Entry`]: struct.Entry.html
    /// [`Error`]: error/enum.Error.html
    ///
    /// ```no_run
    /// # use gnudbm::*;
    /// # let mut db = RwHandle::dummy();
    /// let key = "my key";
    /// let value = "my value";
    /// db.store(key.as_bytes(), &value).unwrap();
    ///
    /// let entry = db.fetch(key.as_bytes()).unwrap();
    /// let as_str: &str = entry.deserialize().unwrap();
    ///
    /// assert_eq!(as_str, value);
    /// ```
    pub fn fetch<K>(&self, key: K) -> GdbmResult<Entry>
        where K: AsRef<[u8]>,
    {
        let key_d = key.as_ref().into();
        let result = unsafe { gdbm_sys::gdbm_fetch(self.handle, key_d) };

        if result.dptr.is_null() {
            Err(GdbmError::from_last().into())
        } else {
            Ok(Entry::new(result))
        }
    }

    /// Removes an entry from the database. Returns `true` if an entry was
    /// removed, and `false` if no entry was present. Returns an [`Error`]
    /// if there is a problem with the database file.
    ///
    /// [`Error`]: error/enum.Error.html
    pub fn remove<K>(&self, key: K) -> GdbmResult<bool>
        where K: AsRef<[u8]>,
    {
        let key_d = key.as_ref().into();
        let result = unsafe { gdbm_sys::gdbm_delete(self.handle, key_d) };
        if result != 0 {
            match Error::from_last() {
                ref e if e.is_no_record() => Ok(false),
                e => Err(e)
            }
        } else {
            Ok(true)
        }
    }

    /// Counts the number of items in this database. This is not cached.
    ///
    /// Returns the total number of items in the database, or
    /// an [`Error`] if there was a problem reading the database.
    ///
    /// [`Error`]: error/enum.Error.html
    /// # Examples
    ///
    /// ```no_run
    /// # use gnudbm::*;
    /// # let mut db = RwHandle::dummy();
    /// for i in 0..100 {
    ///     let key = format!("key {}", i);
    ///     let value = format!("value {}", i);
    ///     db.store(key.as_bytes(), &value).unwrap();
    /// }
    ///
    /// assert_eq!(db.count().unwrap(), 100);
    /// ```
    pub fn count(&self) -> GdbmResult<usize> {
        let mut count = 0_u64;
        let count_ptr: *mut u64 = &mut count;
        let r = unsafe { gdbm_sys::gdbm_count(self.handle, count_ptr) };
        if r == -1 {
            Err(GdbmError::from_last().into())
        } else {
            Ok(count as usize)
        }
    }

    /// Returns an iterator over the keys and values in this database.
    /// The iterator's element type is `(`[`Key`], [`Entry`]`)`.
    ///
    /// [`Key`]: struct.Key.html
    /// [`Entry`]: struct.Entry.html
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use gnudbm::*;
    /// # let mut db = RwHandle::dummy();
    /// assert_eq!(db.count().unwrap(), db.iter().count());
    /// ```
    pub fn iter<'a>(&'a self) -> Iter<'a> {
        Iter::new(self)
    }

    /// Checks the database for the existence of `key`.
    ///
    /// Returns an [`Error`] if there was a problem reading the database file,
    /// otherwise returns a `true` if the key is present in the database.
    ///
    /// [`Error`]: error/enum.Error.html
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use gnudbm::*;
    /// # let mut db = RwHandle::dummy();
    /// let key = "my key";
    /// let value = "my value";
    /// db.store_checked(key.as_bytes(), &value).unwrap();
    ///
    /// assert!(db.contains_key(key.as_bytes()).unwrap());
    /// assert!(!db.contains_key("missing key".as_bytes()).unwrap());
    /// ```
    pub fn contains_key(&self, key: &[u8]) -> GdbmResult<bool> {
        let key_d: gdbm_sys::datum = key.into();
        let result = unsafe { gdbm_sys::gdbm_exists(self.handle, key_d) };

        if result == 0 {
            let errno = last_errno();
            if errno != gdbm_sys::GDBM_NO_ERROR {
                Err(errno.into())
            } else {
                Ok(false)
            }
        } else {
            Ok(true)
        }
    }

    /// Synchronizes the changes in the database with the file on disk.
    pub fn sync(&self) {
        //TODO: this should be failable, but docs don't show how we get the error :|
        unsafe { gdbm_sys::gdbm_sync(self.handle) }
    }

    /// Reorganizes the database file, potentially reducing its size on disk.
    ///
    /// # Note
    ///
    /// This is expensive, and should be used rarely. From the [gdbm docs]:
    ///
    /// > If you have had a lot of deletions and would like to shrink the
    /// > space used by the gdbm file, this function will reorganize the database.
    /// > This results, in particular, in shortening the length of a gdbm file
    /// > by removing the space occupied by deleted records.
    ///
    /// [gdbm docs]: http://puszcza.gnu.org.ua/software/gdbm/manual/gdbm.html#Reorganization
    pub fn reorganize(&mut self) -> GdbmResult<()> {
        let result = unsafe { gdbm_sys::gdbm_reorganize(self.handle) };
        if result != 0 {
            Err(Error::from_last())
        } else {
            Ok(())
        }
    }

    //TODO: this should actually be an option in GdbmOpener
    /// Set the size of the internal bucket cache.
    ///
    /// # Note
    ///
    /// This option may only be set _once_ on each database handle. Subsequent
    /// calls may fail silently.
    pub fn set_cache_size(&mut self, size: usize) -> GdbmResult<()> {
        self.set_opt(gdbm_sys::GDBM_SETCACHESIZE, size);
        Ok(())
    }

    /// Returns the size of the internal bucket cache.
    pub fn get_cache_size(&self) -> GdbmResult<usize> {
        Ok(self.get_opt(gdbm_sys::GDBM_GETCACHESIZE))
    }

    /// Sets whether the database is in sync mode; if this is `true`,
    /// changes to the database are written to disk as they occur.
    pub fn set_sync_mode(&mut self, mode: bool) -> GdbmResult<()> {
        let mode = if mode { 1i32 } else { 0 };
        self.set_opt(gdbm_sys::GDBM_SETSYNCMODE, mode);
        Ok(())
    }

    /// Returns `true` if the database is in sync mode.
    pub fn get_sync_mode(&self) -> GdbmResult<bool> {
        // a c bool
        let r: i32 = self.get_opt(gdbm_sys::GDBM_GETSYNCMODE);
        match r {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::from_last()),
        }
    }

    /// Sets the maximum size of a memory mapped region. This will be rounded
    /// to the nearest page boundary.
    ///
    /// # Note
    ///
    /// By default, this is equal to `usize::max_value()`.
    pub fn set_max_mmap_size(&mut self, size: usize) -> GdbmResult<()> {
        self.set_opt(gdbm_sys::GDBM_SETMAXMAPSIZE, size);
        Ok(())
    }

    /// Returns the current maximum size of a memory mapped region.
    pub fn get_max_mmap_size(&self) -> GdbmResult<usize> {
        Ok(self.get_opt(gdbm_sys::GDBM_GETMAXMAPSIZE))
    }

    /// Set whether or not the database should use memory mapping.
    pub fn set_mmap_enabled(&mut self, mode: bool) -> GdbmResult<()> {
        let mode = if mode { 1i32 } else { 0 };
        self.set_opt(gdbm_sys::GDBM_SETMMAP, mode);
        Ok(())
    }

    /// Returns whether or not the database should use memory mapping.
    pub fn get_mmap_enabled(&self) -> GdbmResult<bool> {
        let r: i32 = self.get_opt(gdbm_sys::GDBM_GETMMAP);
        match r {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::from_last()),
        }
    }

    /// Returns the block size, in bytes. Block size is set when the database
    /// is first created, and cannot be changed.
    pub fn get_block_size(&self) -> GdbmResult<usize> {
        Ok(self.get_opt(gdbm_sys::GDBM_GETBLOCKSIZE))
    }

    fn set_opt<T>(&self, opt: u32, value: T) {
        let mut value = value;
        let ptr = &mut value as *mut T;
        let ptr = ptr as *mut os_c_void;
        let size = mem::size_of::<T>() as i32;
        unsafe { gdbm_sys::gdbm_setopt(self.handle, opt as i32, ptr, size) };
    }

    fn get_opt<T: Default>(&self, opt: u32) -> T {
        let mut value = T::default();
        let ptr = &mut value as *mut T;
        let ptr = ptr as *mut os_c_void;
        let size = mem::size_of::<T>() as i32;
        unsafe { gdbm_sys::gdbm_setopt(self.handle, opt as i32, ptr, size) };
        value
    }

    //TODO: fdesc? do we want to expose the file descriptor for locking?

    /// returns a dummy database for use with doctests
    #[allow(dead_code)]
    #[doc(hidden)]
    pub fn dummy() -> RwHandle {
        use std::ptr;
        RwHandle { handle: ptr::null_mut() }
    }
}

#[doc(hidden)]
impl Drop for RwHandle {
    fn drop(&mut self) {
        unsafe { gdbm_sys::gdbm_close(self.handle) }
    }
}

impl ReadHandle {
    /// Attempts to fetch an item from the database. See [`RwHandle::fetch`]
    /// for more information.
    ///
    /// [`RwHandle::fetch`]: struct.Database.html#method.fetch
    pub fn fetch<K>(&self, key: K) -> GdbmResult<Entry>
        where K: AsRef<[u8]>,
    {
        self.0.fetch(key)
    }

    /// Returns the number of items in this database. This is not cached.
    pub fn count(&self) -> GdbmResult<usize> {
        self.0.count()
    }

    /// Returns an iterator over the keys and values in this database.
    /// See [`RwHandle::iter`] for more information.
    ///
    /// [`RwHandle::iter`]: struct.Database.html#method.iter
    pub fn iter<'a>(&'a self) -> Iter<'a> {
        self.0.iter()
    }
}

impl GdbmOpener {
    /// Create a new `GdbmOpener`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the option to create the file if it does not exist.
    ///
    /// This corresponds to gdbm's `GDBM_WRCREAT` flag.
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// Sets the option to overwrite any existing file. Implies `create`.
    ///
    /// This corresponds to gdbm's `GDBM_NEWDB` flag.
    pub fn overwrite(&mut self, overwrite: bool) -> &mut Self {
        self.overwrite = overwrite;
        self
    }

    /// Sets the option to always immediately sync changes to disk.
    pub fn sync(&mut self, sync: bool) -> &mut Self {
        self.sync = sync;
        self
    }

    /// Sets the option to avoid file locking.
    pub fn no_lock(&mut self, no_lock: bool) -> &mut Self {
        self.no_lock = no_lock;
        self
    }

    /// Sets the option to disable memory mapping.
    pub fn no_mmap(&mut self, no_mmap: bool) -> &mut Self {
        self.no_mmap = no_mmap;
        self
    }

    /// Attempts to open the file at `path` with the options provided,
    /// returning a read/write database handle.
    pub fn readwrite<P: AsRef<Path>>(&self, path: P) -> GdbmResult<RwHandle> {
        let path = path.as_ref();
        let handle = self.gdbm_open(&path)?;
        Ok(RwHandle { handle })
    }

    /// Attempts to open the file at `path` with the options provided,
    /// returning a read-only database handle.
    ///
    /// This ignores any settings applied by `create` or `overwrite`.
    pub fn readonly<P: AsRef<Path>>(&mut self, path: P) -> GdbmResult<ReadHandle> {
        self.readonly = true;
        let db = self.readwrite(path)?;
        Ok(ReadHandle(db))
    }

    fn gdbm_open(&self, path: &Path) -> GdbmResult<gdbm_sys::GDBM_FILE> {
        let path = CString::new(path.as_os_str().as_bytes())?;
        let path_ptr = path.as_ptr() as *mut i8;

        let mut flags = gdbm_sys::GDBM_WRITER as i32;
        if self.readonly {
            flags = gdbm_sys::GDBM_READER as i32;
        } else if self.overwrite {
            flags = gdbm_sys::GDBM_NEWDB as i32;
        } else if self.create {
            flags = gdbm_sys::GDBM_WRCREAT as i32;
        }

        if self.sync {
            flags |= gdbm_sys::GDBM_SYNC as i32
        }
        if self.no_lock {
            flags |= gdbm_sys::GDBM_NOLOCK as i32
        }
        if self.no_mmap {
            flags |= gdbm_sys::GDBM_NOMMAP as i32
        }

        let handle =
            unsafe { gdbm_sys::gdbm_open(path_ptr, self.block_size, flags, DEFAULT_MODE, None) };

        if handle.is_null() {
            Err(GdbmError::from_last().into())
        } else {
            Ok(handle)
        }
    }
}

impl<'a> Entry<'a> {
    fn new(datum: gdbm_sys::datum) -> Self {
        let slice = unsafe { slice::from_raw_parts(datum.dptr as *const u8, datum.dsize as usize) };
        Entry { datum, slice }
    }

    /// Returns the contents of the entry as a slice of bytes.
    ///
    /// This is zero-cost; the returned slice is the memory returned by the
    /// gdbm C library.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use gnudbm::*;
    /// # let mut db = RwHandle::dummy();
    /// let key = "my key";
    /// let value = "my value";
    /// db.store(key.as_bytes(), &value).unwrap();
    ///
    /// let entry = db.fetch(key.as_bytes()).unwrap();
    ///
    /// assert_eq!(entry.as_bytes(), value.as_bytes());
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.slice
    }

    /// Attempts to deserialize this entry into some type which implements
    /// [`serde::Serialize`].
    ///
    /// This can borrow data, bound to the lifetime of `self`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use gnudbm::*;
    /// # use std::collections::BTreeMap;
    /// # let mut db = RwHandle::dummy();
    /// let key = "my key";
    ///
    /// // BTreeMap implements `Serialize` so we can insert it and parse from an entry.
    /// let value = [("hi", 5), ("you", 2), ("eye", 1)].iter()
    ///     .cloned()
    ///     .collect::<BTreeMap<&'static str, i32>>();
    /// db.store(key.as_bytes(), &value).unwrap();
    ///
    /// let entry = db.fetch(key.as_bytes()).unwrap();
    /// let as_map: BTreeMap<&str, i32> = entry.deserialize().unwrap();
    ///
    /// assert_eq!(as_map.get("hi"), Some(&5));
    ///
    /// ```
    pub fn deserialize<'de, T>(&'de self) -> Result<T, bincode::Error>
    where
        T: Deserialize<'de>,
    {
        bincode::deserialize(self.slice)
    }
}

#[doc(hidden)]
impl<'a> Drop for Entry<'a> {
    fn drop(&mut self) {
        if self.datum.dptr.is_null() { return };
        unsafe {
            libc::free(self.datum.dptr as *mut libc::c_void);
        }
    }
}

impl<'a> Key<'a> {
    /// Returns the contents of the key as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.slice
    }
}

impl<'a> Iter<'a> {
    fn new(db: &'a RwHandle) -> Self {
        let firstkey = unsafe { gdbm_sys::gdbm_firstkey(db.handle) };
        let nxt_key = if firstkey.dptr.is_null() {
            None
        } else {
            Some(firstkey)
        };
        Iter { db, nxt_key }
    }
}

#[doc(hidden)]
impl<'a> Iterator for Iter<'a> {
    type Item = (Key<'a>, Entry<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(d) = self.nxt_key.take() {
            let value_d = unsafe { gdbm_sys::gdbm_fetch(self.db.handle, d) };
            let nxt = unsafe { gdbm_sys::gdbm_nextkey(self.db.handle, d) };
            //TODO: check this error :{
            if value_d.dptr.is_null() {
                return None;
            }
            if !nxt.dptr.is_null() {
                self.nxt_key = Some(nxt);
            } else {
                //TODO? how do we want to handle errors in the iterator?
            }
            Some((Key(Entry::new(d)), Entry::new(value_d)))
        } else {
            None
        }
    }
}

#[doc(hidden)]
impl<'a> Drop for Iter<'a> {
    fn drop(&mut self) {
        if let Some(datum) = self.nxt_key {
            if !datum.dptr.is_null() {
                unsafe {
                    libc::free(datum.dptr as *mut libc::c_void);
                }
            }
        }
    }
}
