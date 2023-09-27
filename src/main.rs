mod datatypes;
mod utils;
mod offsets;
mod gamedata;
mod features;
mod render;
mod human_interface;
mod user_config;

use clap::{crate_authors, crate_version, Arg, ArgMatches, Command, ArgAction};
use gamedata::GameData;
use log::{info, Level};
use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use render::MapData;
use ::std::{time::{Duration, SystemTime}, sync::mpsc, collections::BTreeMap};

use human_interface::*;

use utils::thread::*;
use offsets::scanner::Scanner;

use user_config::default_config::{WeaponConfig, KeyBindings};
use datatypes::game::WeaponId;

//use crate::features::recoil_replay;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // parse args and act accordingly
    let matches = parse_args();
    let scan_sigs = matches.get_one::<bool>("scan").copied().unwrap_or(false);
    let espmod: Option<(Address, u32)> = matches.get_many::<u32>("espmod").map(|v| {
        let x = v.collect::<Vec<&u32>>();
        if x.len() < 2 {
            log::error!("Incorrect number of arguments to esp base arg. -e requires both module origin and size. See help for more info.");
            return (0.into(),0);// TODO MAKE THIS ERROR OUT OF THE PROGRAM OR RETURN NONE SOMEHOW
        }
        log::info!("got esp args: base addr {:?} size {:?}", x[0], x[1]);
        ((*x[0]).into(),*x[1])
    });

    let kernelesp: Option<(Address, umem)> = matches.get_many::<umem>("kernelesp").map(|v| {
        let x = v.collect::<Vec<&umem>>();
        if x.len() < 2 {
            log::error!("Incorrect number of arguments to esp base arg. -e requires both module origin and size. See help for more info.");
            return (0.into(),0);// TODO MAKE THIS ERROR OUT OF THE PROGRAM OR RETURN NONE SOMEHOW
        }
        log::info!("got kernel esp args: base addr {:?} size {:?}", x[0], x[1]);
        ((*x[0]).into(),*x[1])
    });

    extract_args(&matches);

    // vars for managing current config values
    let mut config = user_config::init_user_config("user_config")?;
    let config_watcher = user_config::config_watcher::ConfigWatcher::init("user_config")?;
    // currently held weapon config
    let mut weapon_config: WeaponConfig = WeaponConfig::default();
    // last held weapon for detecting if held weapon changes
    let mut last_weapon: WeaponId = WeaponId::None;
    // holds the key values for the keybinds
    let mut keybinds = KeyBindings::default();


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

    if scan_sigs {
        let mut modules = BTreeMap::new();
        modules.insert("client.dll".to_string(), client_module.to_owned());
        modules.insert("engine.dll".to_string(), engine_module.to_owned());
        let mut scanner = Scanner::init_with_info(modules);
        let mut sigs = scanner.scan_signatures(&mut process);
        let mut netvars = scanner.scan_netvars(&sigs);

        // write out the results to disk
        let out_path = "hazedumper/csgo";
        //let out_path = "csgo";
        let mut original_results = offsets::output::Results::load_from(&out_path).unwrap_or(offsets::output::Results::new(sigs.clone(), netvars.clone()));
        original_results.update(&mut sigs, &mut netvars);
        original_results.dump_all(&out_path).expect("Dump results");
        // let results = offsets::output::Results::new(sigs, netvars);
        // results.dump_all(&out_path).expect("Dump results");
    }

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
    
    // init Esp or return None if failed (don't run ESP process if init failed)
    #[cfg(feature = "esp")]
    let mut esp = features::Esp::new(&mut process, espmod).ok();
    #[cfg(feature = "esp")]
    if esp.is_none() {
        println!("failed to init esp");
    }
    #[cfg(feature = "esp")]
    let mut kernel_esp = features::kernel_esp::KernEsp::new(os.clone(), kernelesp, None).ok();
    #[cfg(feature = "esp")]
    if kernel_esp.is_none() {
        println!("failed to init kern esp");
    }
    

    let mut atrigger = features::AlgebraTrigger::new();
    //let mut recoil_data = features::RecoilRecorder::new();
    #[cfg(feature = "bhop_sus")]
    let mut bhop_sus = features::SusBhop::new();
    //#[cfg(feature = "walls")]
    //let mut walls = features::Walls::new();
    //features::turn_on_walls(&mut process, client_module.base)?;

    'mainloop : loop {
        // check if process is valid
        let delta = match time.elapsed() {
            Ok(t) => t.as_secs_f64(),
            Err(e) => e.duration().as_secs_f64(),
        };
        time = SystemTime::now();

        // reload config values if file was changed
        if config_watcher.watch(&mut config) {
            // config was updated, update some vars
            keybinds = config.get("keybinds").unwrap_or_default();
        }

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

        // update per weapon config if weapon id changed
        if game_data.entity_list.local_player.weapon_id != last_weapon { //TODO ALSO REFRESH THIS IF CONFIG FILE WAS RELOADED
            // update weapon config to be the currently loaded configs data 
            println!("Weapon held switched getting config for weapon {}", game_data.entity_list.local_player.weapon_id);
            weapon_config = config.get(
                format!("weapons.{}", game_data.entity_list.local_player.weapon_id.to_string())
                .as_str()
            )
            .unwrap_or(
                WeaponConfig {
                    aimbot: config.get("aimbot_defaults")?,
                    trigger: config.get("trigger_defaults")?,
                }
            );
            // update last weapon
            last_weapon = game_data.entity_list.local_player.weapon_id;
        }

        let mut framedata = render::FrameData::default();
        framedata.connected = true;
        framedata.velocity = game_data.entity_list.local_player.vec_velocity.magnitude();
        framedata.local_position = render::PlayerLoc{
            world_pos: game_data.entity_list.local_player.vec_origin,
            head_pos: game_data.entity_list.local_player.vec_origin + game_data.entity_list.local_player.vec_view_offset,
            rotation: game_data.view_angles.xy(),
            team: game_data.entity_list.local_player.team_num,
            name: "local".to_string(),
        };
        // send location data to renderer
        for (i, ent) in game_data.entity_list.entities.iter().enumerate() {
            if(ent.dormant &1 == 1) || ent.lifestate > 0 {continue}
            if i == game_data.local_player_idx as usize {continue}
            if game_data.entity_list.local_player.observing_id == 0 || i == game_data.entity_list.local_player.observing_id as usize -1 {continue}
            //if ent.spotted_by_mask & (1 << game_data.local_player.ent_idx) > 0 {continue}

            framedata.locations.push(render::PlayerLoc{
                world_pos: ent.vec_origin,
                head_pos: ent.head_pos,
                rotation: Default::default(),
                team: ent.team_num,
                name: ent.name.clone(),
            });
        }
        if tx.send(framedata).is_err() {
            info!("Failed to send to graphics window. Was likely exited. Ending process.");
            break 'mainloop;
        }

        if game_data.entity_list.local_player.health > 0 || game_data.entity_list.local_player.lifestate == 0 {
            #[cfg(feature = "bhop_sus")]
            bhop_sus.bhop_sus(&mut keyboard, &mut process, &game_data, client_module.base)?;
            //#[cfg(feature = "walls")]
            //walls.toggle_walls_button(&mut keyboard, &mut process, client_module.base)?;
            #[cfg(feature = "aimbot")]
            if config.get::<bool>("aimbot_enabled").unwrap_or(false) {
                aimbot.aimbot(&mut keyboard, &mut human, &game_data);
            }
            //atrigger.algebra_trigger(&mut keyboard, &mut human, &game_data, delta);
            if config.get::<bool>("trigger_enabled").unwrap_or(false) {// && keyboard.is_down(keybinds.trigger as i32) {
                atrigger.update_data_then_trigger(&mut human, &mut game_data, &weapon_config.trigger, delta, &mut process);
            }
            //features::incross_trigger(&mut keyboard, &mut human, &game_data);
            // collect recoil data for weapons
            //recoil_data.process_frame(&game_data, false);

            //recoil_replay(&game_data, &recoil_data, &mut human);

            // run any mouse moves that acumulate from the above features
            human.process_smooth_mouse()?;
            //features::shoot_speed_test(&mut keyboard, &mut human);
            
        }

        #[cfg(feature = "esp")]
        {
            if let Some(e) = &mut esp {
                e.render_esp(&mut process, &game_data);
            }

            if let Some(e) = &mut kernel_esp {
                e.render_esp(&game_data);
            }
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
        .arg(Arg::new("verbose").short('v').action(ArgAction::Count))
        .arg(
            Arg::new("scan")
                .long("scan")
                .short('s')
                .action(clap::ArgAction::SetTrue)
                .required(false)
                .help("if provided then signatures from config.json will be scanned and offsets saved before running")
        )
        .arg(
            Arg::new("espmod")
                .long("espmod")
                .short('e')
                .action(clap::ArgAction::Set)
                //.multiple_values(true)
                .number_of_values(2)
                .value_parser(clap_num::maybe_hex::<u32>)
                .required(false)
                .help("if provided, then the injected ESP Module will be retreived from the module base address and size specified ex: '-e 0xb9ff0a9fff 0x300000'")
        )
        .arg(
            Arg::new("kernelesp")
                .long("kernelesp")
                .short('k')
                .action(clap::ArgAction::Set)
                //.multiple_values(true)
                .number_of_values(2)
                .value_parser(clap_num::maybe_hex::<umem>)
                .required(false)
                .help("if provided, then the KERNEL ESP Module will be scanned for from the module base address and size specified ex: '-k 0xb9ff0a9fff 0x300000'")
        )
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
    let log_level = match matches.get_count("verbose") {
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