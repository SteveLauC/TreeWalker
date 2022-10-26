#### TreeWalker 

A toy replicate of [`walkdir`](https://crates.io/crates/walkdir)

Different from `walkdir`, `TreeWalker` yields [`DirEntry`](https://doc.rust-lang.org/std/fs/struct.DirEntry.html)
type from the standard library each time. This also makes `TreeWalker` unusable
on `/` since you simply can't get a `DirEntry` representing `/`.

The traversal is done is `pre-order`.

#### Why did you create this crate

I am just curious about why doesn't `walkdir` returns the std `DirEntry`. To verify
this, I decided to try to implement it myself. Well, I totally understand the
reason right now and it makes sense to have their customized type.

#### Usage

```rust
use tree_walker::TreeWalker;

fn main() {
    let mut walker = TreeWalker::new("/home/steve").unwrap();

    while let Some(Ok(entry)) = walker.next() {
        println!("{:?}", entry);
    }
}
```
