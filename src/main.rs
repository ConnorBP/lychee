mod datatypes;
mod utils;
mod offsets;
mod gamedata;
mod features;
mod render;
mod human_interface;
mod bsp_parser;

use clap::{crate_authors, crate_version, Arg, ArgMatches, Command};
use gamedata::GameData;
use log::{info, Level};
use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use render::MapData;
use ::std::{time::{Duration, SystemTime}, sync::mpsc, collections::BTreeMap};

use human_interface::*;

use utils::thread::*;
use offsets::scanner::Scanner;

//use crate::features::recoil_replay;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // parse args and act accordingly
    let matches = parse_args();
    extract_args(&matches);

    let (tx, map_tx) = render::start_window_render()?;

    // a "human" we get to tell what to do
    let mut human = HumanInterface::new()?;

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
        let proc_info;
        loop {
            info!("Waiting for process handle");
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
    let mut client_module = wait_for(process.module_by_name("client.dll"),Duration::from_secs(10));
    info!("Got Client Module:\n {:?}", client_module);
    //let clientDataSect = process.module_section_by_name(&clientModule, ".data")?;
    let mut engine_module = wait_for(process.module_by_name("engine.dll"), Duration::from_secs(5));
    info!("Got Engine Module:\n {:?}", engine_module);

    let mut modules = BTreeMap::new();
    modules.insert("client.dll".to_string(), client_module.to_owned());
    modules.insert("engine.dll".to_string(), engine_module.to_owned());
    let mut scanner = Scanner::init_with_info(modules);
    scanner.scan_signatures(&mut process);

    //let bat = process.batcher();

    // info!("Dumping Client Module");
    // let client_buf = process
    //     .read_raw(clientModule.base, clientModule.size as usize)
    //     .data_part()?;

    // info!("Dumping Engine Module");
    // let engine_buf = process
    //     .read_raw(engine_module.base, engine_module.size as usize)
    //     .data_part()?;

    // offsets::test(engine_buf.as_bytes());

    // init game data or panic if the process is closed before game data is valid
    let mut game_data = init_gamedata(&mut process, engine_module.base, client_module.base, map_tx.clone())?;
    info!("{:?}", game_data);

    // processing time delta
    let mut time = SystemTime::now();

    // store features that need to retain data
    #[cfg(feature = "aimbot")]
    let mut aimbot = features::AimBot::new();

    let mut atrigger = features::AlgebraTrigger::new();
    //let mut recoil_data = features::RecoilRecorder::new();
    #[cfg(feature = "bhop_sus")]
    let mut bhop_sus = features::SusBhop::new();

    'mainloop : loop {
        // check if process is valid
        let delta = match time.elapsed() {
            Ok(t) => t.as_secs_f64(),
            Err(e) => e.duration().as_secs_f64(),
        };
        time = SystemTime::now();

        if process.state().is_dead() {
            // if process dies set connected to false
            let framedata = render::FrameData{
                connected: false,
                ..Default::default()
            };
            if tx.send(framedata).is_err() {
                info!("Failed to send to graphics window. Was likely exited. Ending process.");
                break 'mainloop;
            }
            // now wait for the new process
            process = {
                let mut ret_proc;
                'waitforproc : loop {
                    info!("process dead. Waiting for new one.");
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    if let Ok(proc) = os.clone().into_process_by_name("csgo.exe") {
                        ret_proc = proc;
                        info!("process found. Waiting for modules to load.");

                        // now that we have a new working proc we also need to reset some stuff

                        // TODO: make the initialization such as getting client and engine module bases into a re usable function
                        // TODO: DO SO BY MAKING ALL OF MAINS STATE A PART OF ONE STRUCT WITH AN INIT FUNC IN IT

                        client_module = wait_for(ret_proc.module_by_name("client.dll"),Duration::from_secs(10));
                        info!("Got Client Module:\n {:?}", client_module);
                        //let clientDataSect = process.module_section_by_name(&clientModule, ".data")?;
                        engine_module = wait_for(ret_proc.module_by_name("engine.dll"), Duration::from_secs(5));
                        info!("Got Engine Module:\n {:?}", engine_module);

                        if let Ok(gd) = init_gamedata(&mut ret_proc, engine_module.base, client_module.base, map_tx.clone()) {
                            game_data = gd;
                        } else {
                            // if the process is closed thus invalidating gamedata and our process handle
                            // then go back to waiting for process handle
                            continue 'waitforproc;
                        }

                        break;
                    }
                }
                ret_proc
            }
        }

        if game_data.load_data(&mut process, client_module.base).is_err() {
            continue 'mainloop;
        }

        let mut framedata = render::FrameData::default();
        framedata.connected = true;
        framedata.local_position = render::PlayerLoc{
            world_pos: game_data.local_player.vec_origin,
            rotation: game_data.local_player.view_angles.xy(),
            team: game_data.local_player.team_num,
            name: "local".to_string(),
        };
        // send location data to renderer
        for (i, ent) in game_data.entity_list.entities.iter().enumerate() {
            if(ent.dormant &1 == 1) || ent.lifestate > 0 {continue}
            if i == game_data.local_player.ent_idx as usize {continue}
            if game_data.local_player.observing_id == 0 || i == game_data.local_player.observing_id as usize -1 {continue}
            //if ent.spotted_by_mask & (1 << game_data.local_player.ent_idx) > 0 {continue}

            framedata.locations.push(render::PlayerLoc{
                world_pos: ent.vec_origin,
                rotation: Default::default(),
                team: ent.team_num,
                name: ent.name.clone(),
            });
        }
        if tx.send(framedata).is_err() {
            info!("Failed to send to graphics window. Was likely exited. Ending process.");
            break 'mainloop;
        }

        if game_data.local_player.health > 0 || game_data.local_player.lifestate == 0 {
            #[cfg(feature = "bhop_sus")]
            bhop_sus.bhop_sus(&mut keyboard, &mut process, &game_data, client_module.base)?;
            #[cfg(feature = "aimbot")]
            aimbot.aimbot(&mut keyboard, &mut human, &game_data);
            //atrigger.algebra_trigger(&mut keyboard, &mut human, &game_data, delta);
            atrigger.update_data_then_trigger(&mut keyboard, &mut human, &mut game_data, delta, &mut process);
            //features::incross_trigger(&mut keyboard, &mut human, &game_data);
            // collect recoil data for weapons
            //recoil_data.process_frame(&game_data, false);

            //recoil_replay(&game_data, &recoil_data, &mut human);

            // run any mouse moves that acumulate from the above features
            //human.process_smooth_mouse();
            //features::shoot_speed_test(&mut keyboard, &mut human);
        }
        // auto send unclick commands to the arduino since we now need to specify mouse down and up commands
        human.process_unclicks()?;
    }

    Ok(())
}

fn init_gamedata(proc: &mut (impl Process + MemoryView), engine_base: Address, client_base: Address, map_tx: mpsc::Sender<MapData>) -> Result<GameData> {
    let gd_ret;
    loop {
        // this loop waits for a user to join a game for the first time before it exists.
        // So if someone closes the game from the main menu it wouldn't figure out if the proc was dead
        // this should fix that
        if proc.state().is_dead() {
            return Err(Error(ErrorOrigin::OsLayer, ErrorKind::NotFound).log_error("Process was closed during init."));
        }

        match gamedata::GameData::new(proc, engine_base, client_base, map_tx.clone()) {
            Ok(gd) => {
                gd_ret = gd;
                break;
            },
            Err(e) => {
                invalid_pause(format!("initialization game data {:?}", e).as_str());
            }
        }
    }
    Ok(gd_ret)
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