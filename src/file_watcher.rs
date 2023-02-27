use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, TryRecvError};

pub struct FileRevisions {
    rx: Receiver<notify::Result<Event>>,
    _watcher: RecommendedWatcher,
}

impl FileRevisions {
    pub fn from_path(filename: &Path) -> notify::Result<Self> {
        let (tx, rx) = channel::<notify::Result<Event>>();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Default::default())?;
        watcher.watch(filename, RecursiveMode::NonRecursive)?;
        Ok(FileRevisions {
            rx,
            _watcher: watcher,
        })
    }
}

impl FileRevisions {
    pub fn changed(&mut self) -> anyhow::Result<bool> {
        let mut ret = false;
        loop {
            match self.rx.try_recv() {
                Ok(Ok(_event)) => ret = true,
                Ok(Err(e)) => return Err(e.into()),
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => panic!(),
            }
        }
        Ok(ret)
    }
}
