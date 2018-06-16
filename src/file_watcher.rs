use notify;
use notify::{watcher, DebouncedEvent, Error, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub struct FileRevisions {
    rx: Receiver<DebouncedEvent>,
    watcher: RecommendedWatcher,
}

impl FileRevisions {
    pub fn from_path(filename: &Path) -> notify::Result<Self> {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
        watcher.watch(filename, RecursiveMode::NonRecursive)?;
        Ok(FileRevisions {
            rx: rx,
            watcher: watcher,
        })
    }
}

impl Iterator for FileRevisions {
    type Item = io::Result<File>;

    fn next(&mut self) -> Option<io::Result<File>> {
        match self.rx.try_recv() {
            Ok(event) => {
                use notify::DebouncedEvent::{Chmod, Create, Error, NoticeRemove, NoticeWrite,
                                             Remove, Rename, Rescan, Write};
                match event {
                    Chmod(path) => Some(path),
                    Create(path) => Some(path),
                    Error(_, _) => None, // TODO: Handle this
                    NoticeRemove(path) => Some(path),
                    NoticeWrite(path) => Some(path),
                    Remove(path) => Some(path),
                    Rename(_, path) => Some(path),
                    Rescan => None,
                    Write(path) => Some(path),
                }.map(File::open)
            }
            Err(a) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fake_test() {
        assert_eq!(2 + 2, 4);
    }
}
