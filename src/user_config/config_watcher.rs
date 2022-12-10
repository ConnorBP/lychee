use config::Config;
use log::warn;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher, Error, ReadDirectoryChangesWatcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::Duration;

pub struct ConfigWatcher {
    rx: Receiver<Result<Event, Error>>,
    watcher: ReadDirectoryChangesWatcher,
    config_name: String,
}

impl ConfigWatcher {

    /// starts a file watcher to notify when the config is changed
    /// listen for this watchers events by calling the .watch(mut config_ref) method
    pub fn init(file_name: &str) -> std::result::Result<Self, Box<dyn std::error::Error>>{
        let pathname = format!("{}.json",file_name);
        let (tx, rx) = channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher: RecommendedWatcher = Watcher::new(
            tx,
            notify::Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch(
                Path::new(&pathname),
                RecursiveMode::NonRecursive,
            )?;

        // return a new instance of self with the receiver
        Ok(Self{
            rx,
            watcher,
            config_name: file_name.to_string(),
        })
    }


    /// hot reload the config if it gets changed
    /// returns true if config was reloaded or false otherwise
    pub fn watch(&self, config_ref: &mut Config) -> bool {
        match self.rx.try_recv() {
            Ok(Ok(Event {
                kind: notify::event::EventKind::Modify(_),
                ..
            })) => {
                println!(" * user config file changed; refreshing configuration ...");
                //config_ref.refresh().unwrap();
                let tmp = Config::builder()
                    .add_source(config::File::with_name(self.config_name.as_str()).required(true))
                    .build();
                if let Ok(newval) = tmp {
                    *config_ref = newval;
                    return true;
                } else {
                    println!("ERROR IN CONFIG. Failed to reload config. Please check your json syntax")
                }
                
            }
            Ok(Err(e)) => warn!("config file watcher error: {:?}", e),

            //Err(e) => println!("watch error: {:?}", e), // this would spam with try_recv
            Err(e) if e == TryRecvError::Empty => {
                // ignore
            }

            x => {
                warn!("config file watcher encountered unknown error: {:?}", x);
            }
        }
        false
    }
}