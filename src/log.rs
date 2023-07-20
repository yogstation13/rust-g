use crate::error::Result;
use chrono::Utc;
use std::{
    cell::{Cell, RefCell},
    collections::hash_map::{Entry, HashMap},
    ffi::OsString,
    fs,
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    sync::Once,
    thread::JoinHandle,
};

static ONCE: Once = Once::new();

thread_local! {
    static FILE_MAP: RefCell<HashMap<OsString, File>> = RefCell::new(HashMap::new()); //on worker thread
    static SENDER: RefCell<Option<flume::Sender<Box<(String, String, bool)>>>> = RefCell::new(None); //on main thread
    static HANDLE: Cell<Option<JoinHandle<()>>> = Cell::new(None); //on main thread
}

byond_fn!(fn log_write(path, data, ...rest) {
    init_worker();
    SENDER.with(|sender| {
        _ = sender.borrow().as_ref().unwrap().send(Box::new(
            (path.to_string(), data.to_string(), rest.first().map(|x| &**x) == Some("false"))
        ))
    });
    Some("")
});

byond_fn!(
    fn log_close_all() {
        SENDER.with(|cell| cell.replace(None));
        HANDLE.with(|cell| {
            if let Some(handle) = cell.replace(None) {
                let _ = handle.join();
            };
        });
        Some("")
    }
);

fn open(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?
    }

    Ok(OpenOptions::new().append(true).create(true).open(path)?)
}
fn init_worker() {
    ONCE.call_once(|| {
        let (sender, receiver) = flume::unbounded();
        SENDER.with(|cell| *cell.borrow_mut() = Some(sender));
        HANDLE.with(|cell| {
            let handle = std::thread::spawn(move || {
                loop {
                    let packet = receiver.recv();

                    if let Ok(packet) = packet {
                        let (path, data, rest) = *packet;
                        _ = FILE_MAP.with(|cell| -> Result<()> {
                            // open file
                            let mut map = cell.borrow_mut();
                            let path = Path::new(&path);
                            let file = match map.entry(path.into()) {
                                Entry::Occupied(elem) => elem.into_mut(),
                                Entry::Vacant(elem) => elem.insert(open(path)?),
                            };

                            let mut buffer = std::io::BufWriter::new(file);

                            if rest {
                                // Write the data to the file with no accoutrements.
                                write!(buffer, "{}", data)?;
                            } else {
                                // write first line, timestamped
                                let mut iter = data.split('\n');
                                if let Some(line) = iter.next() {
                                    write!(
                                        buffer,
                                        "[{}] {}\n",
                                        Utc::now().format("%F %T%.3f"),
                                        line
                                    )?;
                                }

                                // write remaining lines
                                for line in iter {
                                    write!(buffer, " - {}\n", line)?;
                                }
                            }

                            Ok(())
                        });
                    } else {
                        FILE_MAP.with(|cell| cell.borrow_mut().clear());
                        return;
                    }
                }
            });
            cell.set(Some(handle));
        });
    });
}
