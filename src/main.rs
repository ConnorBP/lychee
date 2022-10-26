extern crate nalgebra_glm as glm;

use clap::{crate_authors, crate_version, Arg, ArgMatches, Command};
use gamedata::GameData;
use log::{info, warn, Level};
use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use patternscan::scan;
use serialport::SerialPort;
use std::io::Cursor;
use ::std::{ops::Add, time::Duration};

use config::Config;
use lazy_static::lazy_static;
use std::sync::RwLock;

mod gamedata;
mod offsets;
mod features;
mod math;
mod render;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // parse args and act accordingly
    let matches = parse_args();
    extract_args(&matches);

    let tx = render::start_window_render()?;

    // init the connection to the serial port for mouse and keyboard output
    println!("Fetching Serial Ports...");
    let ports = serialport::available_ports()?;
    for p in ports {
        println!("{}", p.port_name);
    }
    let mut port = serialport::new("COM3", 115_200)
        .timeout(Duration::from_millis(10))
        .open()?;
    // example usage for mouse 0 click:
    //port.write(b"m0\n")?;

    // create inventory + os
    let connector_args : ConnectorArgs = ":device=FPGA".parse()?;
    let connector = memflow_pcileech::create_connector(&connector_args)?;

    let mut os = Win32Kernel::builder(connector)
        .build_default_caches()
        //.arch(ArchitectureIdent::X86(64, false))
        .build()?;

    // load keyboard reader
    let mut keyboard = os.clone().into_keyboard()?;

    // get process info from victim computer

    let base_info = {
        let mut proc_info;
        loop {
            println!("Waiting for process handle");
            if let Ok(res) = os.process_info_by_name("csgo.exe") {
                proc_info = res;
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        proc_info
    };
    let process_info = os.process_info_from_base_info(base_info)?;
    //let mut process = Win32Process::with_kernel(os, process_info.clone());
    let mut process = os.clone().into_process_by_name("csgo.exe")?;
    info!("Got Proccess:\n {:?}", process_info);

    // fetch info about modules from the process
    let mut client_module = process.module_by_name("client.dll")?;
    info!("Got Client Module:\n {:?}", client_module);
    //let clientDataSect = process.module_section_by_name(&clientModule, ".data")?;
    let mut engine_module = process.module_by_name("engine.dll")?;
    info!("Got Engine Module:\n {:?}", engine_module);

    //let bat = process.batcher();

    // info!("Dumping Client Module");
    // let client_buf = process
    //     .read_raw(clientModule.base, clientModule.size as usize)
    //     .data_part()?;

    // info!("Dumping Engine Module");
    // let engine_buf = process
    //     .read_raw(engineModule.base, engineModule.size as usize)
    //     .data_part()?;


    let mut game_data = init_gamedata(&mut process, engine_module.base, client_module.base);
    println!("{:?}", game_data);

    loop {
        // check if process is valid

        if process.state().is_dead() {
            process = {
                let mut ret_proc;
                loop {
                    info!("process dead. Waiting for new one.");
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    if let Ok(proc) = os.clone().into_process_by_name("csgo.exe") {
                        ret_proc = proc;
                        info!("process found. Waiting for modules to load.");
                        std::thread::sleep(std::time::Duration::from_secs(20));//todo make the modules wait until success

                        // now that we have a new working proc we also need to reset some stuff

                        // TODO: make the initialization such as getting client and engine module bases into a re usable function
                        // and call it here. and also make those global vars maybe

                        client_module = process.module_by_name("client.dll")?;
                        engine_module = process.module_by_name("engine.dll")?;

                        game_data = init_gamedata(&mut process, engine_module.base, client_module.base);

                        break;
                    }
                }
                ret_proc
            }
        }


        //clearscreen::clear()?;
        if  game_data.load_data(&mut process, client_module.base).is_err() {
            invalid_pause("game data");
        }

        // print out a list of currently non dormant entities
        // for (i, ent) in game_data.entity_list.entities.iter().enumerate() {
        //     if (ent.dormant &1) == 0 {
        //         println!("({}) || {:?}", i, ent);
        //     }
        // }

        //std::thread::sleep(Duration::from_millis(10));

        //let health: i32 = process.read(local_player.add(*offsets::NET_HEALTH)).data()?;
        if game_data.local_player.health <= 0 || game_data.local_player.lifestate != 0 {
            info!("player dead. Sleeping for a bit");
            std::thread::sleep(Duration::from_secs(5));
        }

        let mut framedata = render::FrameData::default();
        // send location data to renderer
        for (i, ent) in game_data.entity_list.entities.iter().enumerate() {
            if(ent.dormant &1 == 1) || ent.lifestate > 0 {continue}
            if i == game_data.local_player.ent_idx as usize {continue}
            let worldpos:glm::Vec3 = (ent.vec_origin + ent.vec_view_offset).into();
            if let Some(screenpos) = math::world_2_screen(&worldpos, &game_data.vm, None, None) {
                framedata.locations.push(render::PlayerLoc{
                    pos: screenpos,
                    team: ent.team_num,
                });

            }
        }
        tx.send(framedata)?;

        //println!("down: {} {} {}", keyboard.is_down(0x12), keyboard.is_down(0x20), keyboard.is_down(0x06));
        //features::bhop(&mut keyboard, &mut port);
        if !keyboard.is_down(0x06) {continue}

        if game_data.local_player.incross > 0 && game_data.local_player.incross <= 64 {
            //info!("incross: {}", game_data.local_player.incross);
            if let Some(enemy_team) = game_data.entity_list.get_team_for((game_data.local_player.incross as usize) -1) {
                //println!("enemy team: {}", enemy_team);
                if enemy_team != game_data.local_player.team_num && game_data.local_player.aimpunch_angle > -0.04 {
                    port.write(b"m0\n")?;
                    //print!("firing {}", game_data.local_player.aimpunch_angle);
                }
            }
        }
            
        
    }

    
    // let client_state_sig = "A1 ? ? ? ? 33 D2 6A 00 6A 00 33 C9 89 B0";
    // let entity_list_sig = "BB ? ? ? ? 83 FF 01 0F 8C ? ? ? ? 3B F8";
    // let local_player_sig = "8D 34 85 ? ? ? ? 89 15 ? ? ? ? 8B 41 08 8B 48 04 83 F9 FF";
    // let entity_list = find_signature(process.clone(), &clientModule, &client_buf, entity_list_sig, 1, 0)?;
    // let local_player_getter = find_signature(process.clone(), &clientModule, &client_buf, local_player_sig, 3, 4)?;
    // let client_state = find_signature(process.clone(), &engineModule, &engine_buf, client_state_sig, 1, 0)?;
    // println!("DwEntityList: {:#010x} playerget: {:#010x} clientState: {:#010x}", entity_list, local_player_getter, client_state);
    // let mut yeet: u64 = 0;
    // process.read_into(engineModule.base.add(5820380), &mut yeet).data_part()?;
    // println!("What client state should be: {:#}, {}", yeet, yeet);
    // if let Some(test) = regex_patscan(&client_buf) {
    //     println!("regex test found: {:#010x}", test);
    // }
    
    //process.read_into(addr, out)
    // let mut batcher = process.batcher();
    // batcher.read_into(addr, out)

    


    Ok(())
}

fn init_gamedata(proc: &mut (impl Process + MemoryView), engine_base: Address, client_base: Address) -> GameData {
    let mut gd_ret;
    loop {
        if let Ok(gd) = gamedata::GameData::new(proc, engine_base, client_base) {
            gd_ret = gd;
            break;
        } else {
            invalid_pause("initialization game data");
        }
    }
    gd_ret
}

fn invalid_pause(name: &str) {
    info!("{} invalid. Sleeping for a bit", name);
    std::thread::sleep(Duration::from_secs(5));
}

trait SigScanner {
    fn find_signature(
        &mut self,
        module: &ModuleInfo,
        module_buf: &Vec<u8>,
        signature: &str,
        offset: usize,
        extra: usize
    );
}

// impl SigScanner for Win32Process<dyn Process + MemoryView> {
//     fn find_signature(
//         &mut self,
//         module: &ModuleInfo,
//         module_buf: &Vec<u8>,
//         signature: &str,
//         offset: usize,
//         extra: usize
//     ) {
//         todo!()
//     }
// }

fn regex_patscan(module_buf: &[u8]) -> Option<usize> {
    use ::regex::bytes::*;
    // "A1 ? ? ? ? 33 D2 6A 00 6A 00 33 C9 89 B0" clientstate
    //"8D 34 85 ? ? ? ? 89 15 ? ? ? ? 8B 41 08 8B 48 04 83 F9 FF"
    //let re = Regex::new("(?-u)\\x8D\\x34\\x85(?s:.)(?s:.)(?s:.)(?s:.)\\x89\\x15(?s:.)(?s:.)(?s:.)(?s:.)\\x8B\\x41\\x08\\x8B\\x48\\x04\\x83\\xF9\\xFF").expect("malformed marker sig");
    let re = Regex::new("(?-u)\\xA1(?s:.)(?s:.)(?s:.)(?s:.)\\x33\\xD2\\x6A\\x00\\x6A\\x00\\x33\\xC9\\x89\\xB0").expect("malformed marker sig");
    let buff_offs = re.find_iter(module_buf).next()?.start();
    Some(buff_offs as usize)
}

// fn keyboard_test(os: Win32Kernel<>) {
//     let mut keyboard = os.into_keyboard()?;
//     loop {
//         println!("space down: {:?}", keyboard.is_down(0x20));
//         std::thread::sleep(std::time::Duration::from_millis(1000));
//     }
// }

fn find_signature(
    mut proc: impl Process + MemoryView,
    module: &ModuleInfo,
    module_buf: &Vec<u8>,
    signature: &str,
    offset: usize,
    extra: usize
) -> Result<usize> {
    let locs = scan(Cursor::new(module_buf), signature)
        .expect("could not find any instances of scanned sig due to an error");
    match locs.len() {
        0 => {
            return Err(Error(ErrorOrigin::VirtualMemory, ErrorKind::NotFound).log_error("no locations found from memory scan"));
        },
        1 => {},
        _=> {
            warn!("More than one memory location found from pattern scan. Signature may be out of date.");
        }
    }
    let mut location = locs[0] + offset;
    info!("location before reading mem: {:#010x}", location);
    proc.read_into(module.base.add(location), &mut location).data_part()?;
    info!("location after reading mem: {:#010x}", location);
    location = location + extra;
    info!("location + extra: {:#010x}", location);
    info!("Found client pattern: {:#X}", location);
    Ok(location)
}

fn parse_args() -> ArgMatches {
    Command::new("lyche")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("verbose").short('v').multiple_occurrences(true))
        // .arg(
        //     Arg::new("connector")
        //         .long("connector")
        //         .short('c')
        //         .takes_value(true)
        //         .required(false)
        //         .multiple_values(true),
        // )
        // .arg(
        //     Arg::new("os")
        //         .long("os")
        //         .short('o')
        //         .takes_value(true)
        //         .required(true)
        //         .multiple_values(true),
        // )
        .get_matches()
}

fn extract_args(matches: &ArgMatches) {
    let log_level = match matches.occurrences_of("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };
    simplelog::TermLogger::init(
        log_level.to_level_filter(),
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();
}



// some test stuff:

    // // Print process list, formatted
    // println!(
    //     "{:>5} {:>10} {:>10} {:<}",
    //     "PID", "SYS ARCH", "PROC ARCH", "NAME"
    // );

    // for p in process_list {
    //     println!(
    //         "{:>5} {:^10} {:^10} {} ({}) ({:?})",
    //         p.pid, p.sys_arch, p.proc_arch, p.name, p.command_line, p.state
    //     );
    // }
    

    // print list of modules
    // let modules = process.module_list()?;
    // for m in modules {
    //     println!(
    //         "{:#010x} {:^24} {:^8} {} ({})\n({:?})",
    //         m.base, m.address, m.size, m.arch, m.name, m.path
    //     )
    // }