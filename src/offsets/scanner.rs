// signature scanner by segfault based on hazedumper
// for use with memflow-rs

use ::std::collections::BTreeMap;
use ::std::convert::TryInto;
use ::std::time::Duration;

use failure::Fail;
use log::{info, warn, debug};
use memflow::prelude::*;

use crate::offsets::{find_pattern, SIG_CONFIG};
use crate::utils::thread::*;

use super::hconfig::Signature;

type Map<T> = BTreeMap<String, T>;

/// the memory scanners data such as dumped modules
#[derive(Default)]
pub struct Scanner {
    module_info: Map<ModuleInfo>,
    module_bytes: Map<Vec<u8>>,
}

impl Scanner {
    pub fn init() -> Self {
        Self::default()
    }

    pub fn init_with_info(module_info: Map<ModuleInfo>) -> Self {
        Self{
            module_info,
            ..Default::default()
        }
    }
    
    /// Scan the signatures from the config and return a `Map<usize>`.
    pub fn scan_signatures(&mut self, process: &mut (impl Process + MemoryView)) -> Map<usize> {
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

    pub fn find_signature(&mut self, sig: &Signature, proc: &mut (impl Process + MemoryView)) -> ScanResult<usize> {
        // get module info from the Map
        // or else find the info from proc then store it in the map and return a reference to the newly stored value
        let module_info = if let Some(module_info) = self.module_info.get::<String>(&sig.module) {
            module_info
        } else {
            let info = wait_for(proc.module_by_name(sig.module.as_str()),Duration::from_secs(10));
            self.module_info.insert(sig.module.clone(), info.to_owned());
            self.module_info.get::<String>(&sig.module).unwrap()
        };
        
        // get module dump bytes from the Map
        // or else dump the bytes from proc then store it in the map and return a reference to the newly stored value
        let mut write_bytes_to_map = false;
        let get_module_bytes = self.module_bytes.get::<String>(&sig.module)
            .map_or_else(
                || {
                    info!("Dumping {}",sig.module);
                    let mod_buff = proc
                        .read_raw(module_info.base, module_info.size as usize)
                        .data().map_or(None, |d| Some(d));
                    write_bytes_to_map = true;
                    if mod_buff.is_none() {
                        debug!("Failed to dump module {}", sig.module);
                    }
                    mod_buff
                },
                |d| Some(d.to_vec())// this will copy the whole ass module dump every signature scan. Kill me please fix this
            );

        if let Some(module_bytes) = &get_module_bytes {
            if write_bytes_to_map {
                self.module_bytes.insert(sig.module.to_owned(), module_bytes.clone());
            }
            let mut addr = find_pattern(module_bytes, sig.pattern.as_str()).ok_or(ScanError::PatternNotFound)?;
            debug!(
                "Pattern found at: {:#X} (+ base = {:#X})",
                addr,
                module_info.base + addr
            );

            for (i, o) in sig.offsets.iter().enumerate() {
                debug!("Offset #{}: ptr: {:#X} offset: {:#X}", i, addr, o);

                let pos = (addr as isize).wrapping_add(*o) as usize;
                // let data_u32 = module_bytes.get(pos).ok_or_else(|| {
                //     debug!("WARN OOB - ptr: {:#X} module size: {:#X}", pos, module_info.size);
                //     ScanError::OffsetOutOfBounds
                // })?;
                // let data_u64 = module_bytes.get(pos).ok_or_else(|| {
                //     debug!("WARN OOB - ptr: {:#X} module size: {:#X}", pos, module_info.size);
                //     ScanError::OffsetOutOfBounds
                // })?;

                //let arch = module_info.arch;
                let arch = proc.info().proc_arch;
                let sys_arch = proc.info().sys_arch;

                let is_wow64 = match arch {
                    ArchitectureIdent::AArch64(_) => false,
                    ArchitectureIdent::X86(32,_) => {
                        match sys_arch {
                            ArchitectureIdent::AArch64(_) => true,
                            ArchitectureIdent::X86(64,_) => true,
                            _ => false,
                        }
                    },
                    ArchitectureIdent::X86(64,_) => false,
                    _ => false,
                };

                debug!("wow64 is {} for {}", is_wow64, module_info.name);
                let tmp = if is_wow64 {
                    let raw: u32 = u32::from_le_bytes(module_bytes[pos..pos+4].try_into().map_err(|_|ScanError::OffsetOutOfBounds)?);//= unsafe { std::mem::transmute(data) };
                    raw as usize
                } else {
                    let raw: u64 = u64::from_le_bytes(module_bytes[pos..pos+8].try_into().map_err(|_|ScanError::OffsetOutOfBounds)?);
                    raw as usize
                };

                addr = tmp.wrapping_sub(module_info.base.to_umem() as usize);
                debug!("Offset #{}: raw: {:#X} - base => {:#X}", i, tmp, addr);
            }

            if sig.rip_relative {
                debug!(
                    "rip_relative: addr {:#X} + rip_offset {:#X}",
                    addr, sig.rip_offset
                );
                addr = (addr as isize).wrapping_add(sig.rip_offset) as usize;
                debug!("rip_relative: addr = {:#X}", addr);
        
                debug!("getting raw");
                let rip: u32 = get_raw(module_bytes, module_info.base.to_umem() as usize, addr, true)
                    .ok_or(ScanError::RIPRelativeFailed)?;
        
                debug!(
                    "rip_relative: addr {:#X} + rip {:#X} + {:#X}",
                    addr,
                    rip,
                    ::std::mem::size_of::<u32>()
                );
                addr = addr.wrapping_add(rip as usize + ::std::mem::size_of::<u32>());
                debug!("rip_relative: addr => {:#X}", addr);
            }
        
            debug!("Adding extra {:#X}", sig.extra);
            addr = (addr as isize).wrapping_add(sig.extra) as usize;
            if !sig.relative {
                debug!(
                    "Not relative, addr {:#X} + base {:#X} => {:#X}",
                    addr,
                    module_info.base,
                    addr.wrapping_add(module_info.base.to_umem() as usize)
                );
                addr = addr.wrapping_add(module_info.base.to_umem() as usize);
            }
        
            Ok(addr)
        } else {
            debug!("Module not found at get_moudle_bytes");
            Err(ScanError::ModuleNotFound)
        }
    }
}



/// o: Offset
/// is_relative: Base has already been subtracted.
fn get_raw<T: Copy>(data: &Vec<u8>, base: usize, mut o: usize, is_relative: bool) -> Option<T> {
    if !is_relative {
        o -= base;
    }
    if o + std::mem::size_of::<T>() >= data.len() {
        return None;
    }
    let ptr = data.get(o)?;
    let raw: T = unsafe { std::mem::transmute_copy(ptr) };
    Some(raw)
}


pub type ScanResult<T> = ::std::result::Result<T, ScanError>;

#[derive(Debug, Fail)]
pub enum ScanError {
    #[fail(display = "Module not found")]
    ModuleNotFound,

    #[fail(display = "Pattern not found")]
    PatternNotFound,

    #[fail(display = "Offset out of module bounds")]
    OffsetOutOfBounds,

    #[fail(display = "rip_relative failed")]
    RIPRelativeFailed,
}