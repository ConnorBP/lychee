mod datatypes;
mod utils;
mod offsets;
mod gamedata;
mod features;
mod render;
mod human_interface;
mod bsp_parser;

use clap::{crate_authors, crate_version, Arg, ArgMatches, Command};
use features::AlgebraTrigger;
use gamedata::GameData;
use log::{info, Level};
use memflow::prelude::v1::*;
use memflow_pcileech::PciLeech;
use memflow_win32::prelude::v1::*;
use render::{MapData, FrameData};
use ::std::{time::{Duration, SystemTime}, sync::mpsc::{self, Sender}};

use human_interface::*;

//use crate::features::recoil_replay;

/// Blocks thread until result returns success
fn wait_for<T>(result:Result<T>, delay: Duration) -> T 
{
    let ret;
    loop {
        if let Ok(val) = result {
            ret = val;
            break;
        }
        info!("waiting for valid result");
        std::thread::sleep(delay);
    }
    ret
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // parse args and act accordingly
    let matches = parse_args();
    extract_args(&matches);

    let mut lyche = LycheProgram::init()?;
    lyche.init_process()?;
    lyche.run()?;

    Ok(())
}

pub trait MutProcess: Process + MemoryView {}
impl<T: Process + MemoryView> MutProcess for T {}

struct LycheProgram<'a> {
    human: HumanInterface,
    tx: Sender<FrameData>,
    map_tx: Sender<MapData>,
    os: Win32Kernel<CachedPhysicalMemory<'a, PciLeech, cache::TimedCacheValidator>, CachedVirtualTranslate<DirectTranslate, cache::TimedCacheValidator>>,
    keyboard: Win32Keyboard<VirtualDma<CachedPhysicalMemory<'a, PciLeech, cache::TimedCacheValidator>, CachedVirtualTranslate<DirectTranslate, cache::TimedCacheValidator>, Win32VirtualTranslate>>,
    //process: Option<Win32Process<CachedPhysicalMemory<'a, PciLeech, cache::TimedCacheValidator>, CachedVirtualTranslate<DirectTranslate, cache::TimedCacheValidator>, Win32VirtualTranslate>>,
    process: Option<Box<dyn MutProcess + 'a>>,
    client_module: Option<ModuleInfo>,
    engine_module: Option<ModuleInfo>,
    game_data: Option<GameData>,
    
    
    time: SystemTime,

    // features
    atrigger: AlgebraTrigger,
    #[cfg(feature = "bhop_sus")]
    bhop_sus: SusBhop,
}

impl LycheProgram<'_> {
    fn init() -> std::result::Result<Self, Box<dyn std::error::Error>> {

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

        // processing time delta
        let mut time = SystemTime::now();

        // store features that need to retain data
        #[cfg(feature = "aimbot")]
        let mut aimbot = features::AimBot::new();

        let mut atrigger = features::AlgebraTrigger::new();
        //let mut recoil_data = features::RecoilRecorder::new();
        #[cfg(feature = "bhop_sus")]
        let mut bhop_sus = features::SusBhop::new();

        Ok(LycheProgram {
            human,
            tx,
            map_tx,
            os,
            keyboard,
            process: None,
            client_module: None,
            engine_module: None,
            game_data: None,


            time,

            // features
            atrigger,
            #[cfg(feature = "bhop_sus")]
            bhop_sus,

        })
    }

    fn init_process<'a>(&'a mut self) -> std::result::Result<(), Box<dyn std::error::Error + 'a>> {
        // get process info from victim computer

        let mut ret_proc;
        'waitforproc : loop {
            info!("Waiting for process handle.");
            std::thread::sleep(std::time::Duration::from_secs(5));
            
            if let Ok(proc) = self.os.process_by_name("csgo.exe") {
                ret_proc = proc;
                info!("process found. Waiting for modules to load.");

                // now that we have a new working proc we also need to reset some stuff
 
                // wait for can fail if the user closes csgo while we are still waiting for this module
                self.client_module = Some(wait_for(ret_proc.module_by_name("client.dll"),Duration::from_secs(10)));
                info!("Got Client Module:\n {:?}", self.client_module);
                //let clientDataSect = process.module_section_by_name(&clientModule, ".data")?;
                self.engine_module = Some(wait_for(ret_proc.module_by_name("engine.dll"), Duration::from_secs(5)));
                info!("Got Engine Module:\n {:?}", self.engine_module);

                // info!("Dumping Client Module");
                // let client_buf = process
                //     .read_raw(clientModule.base, clientModule.size as usize)
                //     .data_part()?;

                // info!("Dumping Engine Module");
                // let engine_buf = process
                //     .read_raw(engineModule.base, engineModule.size as usize)
                //     .data_part()?;

                break;
            }
        }
        self.process = Some(Box::new(ret_proc));

        info!("{:?}", self.game_data);
        Ok(())
    }

    fn run(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        'mainloop : loop {
            // check if process is valid
            let delta = match self.time.elapsed() {
                Ok(t) => t.as_secs_f64(),
                Err(e) => e.duration().as_secs_f64(),
            };
            self.time = SystemTime::now();
    
            if self.process.is_none() || self.process.as_mut().unwrap().state().is_dead() {
                // if process dies set connected to false
                let framedata = render::FrameData{
                    connected: false,
                    ..Default::default()
                };
                if self.tx.send(framedata).is_err() {
                    info!("Failed to send to graphics window. Was likely exited. Ending process.");
                    break 'mainloop;
                }
                // now wait for the new process
                self.init_process()?;
            }
            
            if let Some(gd) = &mut self.game_data {
                if gd.load_data(self.process.as_mut().unwrap(), self.client_module.as_ref().unwrap().base).is_err() {
                    continue 'mainloop;
                }

                let mut framedata = render::FrameData::default();
                framedata.connected = true;
                framedata.local_position = render::PlayerLoc{
                    world_pos: gd.local_player.vec_origin,
                    rotation: gd.local_player.view_angles.xy(),
                    team: gd.local_player.team_num,
                    name: "local".to_string(),
                };
                // send location data to renderer
                for (i, ent) in gd.entity_list.entities.iter().enumerate() {
                    if(ent.dormant &1 == 1) || ent.lifestate > 0 {continue}
                    if i == gd.local_player.ent_idx as usize {continue}
                    if gd.local_player.observing_id == 0 || i == gd.local_player.observing_id as usize -1 {continue}
                    //if ent.spotted_by_mask & (1 << game_data.local_player.ent_idx) > 0 {continue}
        
                    framedata.locations.push(render::PlayerLoc{
                        world_pos: ent.vec_origin,
                        rotation: Default::default(),
                        team: ent.team_num,
                        name: ent.name.clone(),
                    });
                }
                if self.tx.send(framedata).is_err() {
                    info!("Failed to send to graphics window. Was likely exited. Ending process.");
                    break 'mainloop;
                }
        
                if gd.local_player.health > 0 || gd.local_player.lifestate == 0 {
                    #[cfg(feature = "bhop_sus")]
                    bhop_sus.bhop_sus(&mut keyboard, &mut process, &game_data, client_module.base)?;
                    #[cfg(feature = "aimbot")]
                    aimbot.aimbot(&mut keyboard, &mut human, &game_data);
                    //atrigger.algebra_trigger(&mut keyboard, &mut human, &game_data, delta);
                    self.atrigger.update_data_then_trigger(&mut self.keyboard, &mut self.human, gd, delta, self.process.as_mut().unwrap());
                    //features::incross_trigger(&mut keyboard, &mut human, &game_data);
                    // collect recoil data for weapons
                    //recoil_data.process_frame(&game_data, false);
        
                    //recoil_replay(&game_data, &recoil_data, &mut human);
        
                    // run any mouse moves that acumulate from the above features
                    //human.process_smooth_mouse();
                    //features::shoot_speed_test(&mut self.keyboard, &mut self.human);
                }
                // auto send unclick commands to the arduino since we now need to specify mouse down and up commands
                self.human.process_unclicks()?;


            } else {
                // init gamedata
                if let Ok(gd) = init_gamedata(self.process.as_mut().unwrap(), self.engine_module.as_ref().unwrap().base, self.client_module.as_ref().unwrap().base, self.map_tx.clone()) {
                    self.game_data = Some(gd);
                } else {
                    continue 'mainloop;
                }
            }
        }
        Ok(())
    }
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

// fn regex_patscan(module_buf: &[u8]) -> Option<usize> {
//     use ::regex::bytes::*;
//     // "A1 ? ? ? ? 33 D2 6A 00 6A 00 33 C9 89 B0" clientstate
//     //"8D 34 85 ? ? ? ? 89 15 ? ? ? ? 8B 41 08 8B 48 04 83 F9 FF"
//     //let re = Regex::new("(?-u)\\x8D\\x34\\x85(?s:.)(?s:.)(?s:.)(?s:.)\\x89\\x15(?s:.)(?s:.)(?s:.)(?s:.)\\x8B\\x41\\x08\\x8B\\x48\\x04\\x83\\xF9\\xFF").expect("malformed marker sig");
//     let re = Regex::new("(?-u)\\xA1(?s:.)(?s:.)(?s:.)(?s:.)\\x33\\xD2\\x6A\\x00\\x6A\\x00\\x33\\xC9\\x89\\xB0").expect("malformed marker sig");
//     let buff_offs = re.find_iter(module_buf).next()?.start();
//     Some(buff_offs as usize)
// }

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