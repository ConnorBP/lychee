// signature scanner by segfault based on hazedumper
// for use with memflow-rs

use ::std::collections::BTreeMap;
use ::std::time::Duration;

use log::{info, warn};
use memflow::prelude::*;

use crate::offsets::{find_pattern, SIG_CONFIG};
use crate::utils::thread::*;

use super::hconfig::Signature;

type Map<T> = BTreeMap<String, T>;

/// the memory scanners data such as dumped modules
#[derive(Default)]
struct Scanner {
    module_info: Map<ModuleInfo>,
    module_bytes: Map<Vec<u8>>,
}

impl Scanner {
    fn init() -> Self {
        Self::default()
    }

    fn init_with_info(module_info: Map<ModuleInfo>) -> Self {
        Self{
            module_info,
            ..Default::default()
        }
    }
    /*
    /// Scan the signatures from the config and return a `Map<usize>`.
    fn scan_signatures(&mut self, process: &mut (impl Process + MemoryView)) -> Map<usize> {
        let conf = SIG_CONFIG
                    .read()
                    .expect("getting lock on sig config");
        info!(
            "Starting signature scanning: {} items",
            conf.signatures.len()
        );
        let mut res = BTreeMap::new();

        for sig in &conf.signatures {
            match self.find_signature(sig, process) {
                Ok(r) => {
                    res.insert(sig.name.clone(), r);
                    info!("Found signature: {} => {:#X}", sig.name, r);
                }
                Err(err) => warn!("{} sigscan failed: {}", sig.name, err),
            };
        }

        info!(
            "Finished signature scanning: {}/{} items successful",
            res.len(),
            conf.signatures.len()
        );
        res
    }

    fn find_signature(&mut self, sig: &Signature, proc: &mut (impl Process + MemoryView)) -> std::result::Result<usize, Box<dyn std::error::Error>> {
        let mut binding = self.module_info.get_mut::<String>(&sig.module);
        let module_info  = binding.get_or_insert_with(|| {
            info!("Getting info on {}",sig.module);
            let mut info = wait_for(proc.module_by_name(sig.module.as_str()),Duration::from_secs(10));
            self.module_info.insert(sig.module.clone(), info);
            info!("Got Module:\n {:?}", info);
            &mut info
        });
        let module_bytes = self.module_bytes.get::<String>(&sig.module).map_or_else(|| {
            info!("Dumping {}",sig.module);
            let mod_buf = proc
                .read_raw(module_info.base, module_info.size as usize)
                .data_part().map_or(None, |d| Some(d));
            &mod_buf
        },
        |d| {
            &Some(*d)
        });
        
   


        Ok(0)//tmp to get linter to shut up
    }*/
}

// fn find_signature(module_info: &ModuleInfo, module_data: &Vec<u8>, sig: &str) {
//     let index = find_pattern(module_data, sig);
//     println!("found {index:?}");

    
// }