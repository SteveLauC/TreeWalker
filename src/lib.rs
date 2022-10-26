/// TreeWalker: A toy implementation of [`walkdir`](https://crates.io/crates/walkdir)
///
/// Different from `walkdir`, [`TreeWalker`] yields [`DirEntry`](https://doc.rust-lang.org/std/fs/struct.DirEntry.html)
/// type from the standard library each time. This also makes `TreeWalker` unusable
/// on `/` since you simply can't get a `DirEntry` representing `/`.
///
///
/// The traversal is done is `pre-order`.

use std::{
    env::current_dir,
    fs::{metadata, read_dir, DirEntry},
    io::Result,
    os::linux::fs::MetadataExt,
    path::{Path, PathBuf},
};

use path_absolutize::Absolutize;

#[derive(Default, Debug)]
pub struct TreeWalker {
    stack: Vec<DirEntry>,
    fatal_error: bool,
}

/// Adjust the `length` field of a `PathBuf` to make it become its parent
/// directory.
fn cd_to_parent(path: PathBuf) -> PathBuf {
    if let Some(parent) = path.parent() {
        assert!(path.capacity() >= parent.as_os_str().len());
        unsafe {
            let ptr_to_len =
                (&path as *const PathBuf as *mut PathBuf as *mut usize).add(2);
            *ptr_to_len = parent.as_os_str().len();
        }
    }

    path
}

/// Return a clean, absolute path of `path`
fn absolute_path<P: AsRef<Path>>(path: P) -> PathBuf {
    // get current working directory, and concatenate it with `path` so that we
    // can get the full path
    let mut abs_path = current_dir().expect("can not get CWD");
    abs_path.push(path.as_ref());
    // canonicalize `path` to remove extra `/`, `.` and `..`
    abs_path
        .absolutize()
        .expect("can not clear path")
        .to_path_buf()
}

impl TreeWalker {
    /// Construct a [`TreeWalker`] instance.
    pub fn new<P: AsRef<Path>>(start: P) -> Result<Self> {
        let start_metadata = metadata(start.as_ref())?;

        let mut walker = TreeWalker::default();

        // Absolutize `start`
        let start = absolute_path(start);
        // get start's parent directory
        let parent = cd_to_parent(start);

        // iterate over the entries in `parent` to find `start`
        let parent_dir = read_dir(parent.as_path())?;
        for res_item in parent_dir {
            let item = res_item?;
            let item_metadata = item.metadata()?;
            if item_metadata.st_dev() == start_metadata.st_dev()
                && item_metadata.st_ino() == start_metadata.st_ino()
            {
                // push `start` to the stack
                walker.stack.push(item);
                break;
            }
        }

        // When used on `/`, this assertion will fail...
        assert_eq!(walker.stack.len(), 1);
        Ok(walker)
    }
}

impl Iterator for TreeWalker {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        // To avoid dead loop
        if self.fatal_error {
            return None;
        }

        if let Some(entry) = self.stack.pop() {
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    // This is a fatal error, since we need the metadata to
                    // determine the file type
                    self.fatal_error = true;
                    return Some(Err(e));
                }
            };

            // If the popping node is a directory, push its files to the stack.
            if metadata.is_dir() {
                // To do a pre-order traversal, we have to use a temporary stack to
                // reverse the order of its files.
                let mut temp_stack = Vec::with_capacity(5);

                let dir = match read_dir(entry.path()) {
                    Ok(d) => d,
                    Err(e) => {
                        self.fatal_error = false;
                        return Some(Err(e));
                    }
                };

                for res_entry in dir {
                    let entry = match res_entry {
                        Ok(e) => e,
                        Err(e) => {
                            self.fatal_error = true;
                            return Some(Err(e));
                        }
                    };

                    temp_stack.push(entry);
                }

                // push its files into the stack
                while let Some(entry) = temp_stack.pop() {
                    self.stack.push(entry);
                }
            }

            return Some(Ok(entry));
        }

        // stack is empty, traversal is done.
        None
    }
}
