use std::{convert::From, fmt};
use serde::{Serialize,Deserialize};

impl From<u32> for WeaponId {
    fn from(v: u32) -> Self {
        match v {
            x if x == WeaponId::Deagle as u32 => WeaponId::Deagle,
            x if x == WeaponId::Elite as u32 => WeaponId::Elite,
            x if x == WeaponId::Fiveseven as u32 => WeaponId::Fiveseven,
            x if x == WeaponId::Glock as u32 => WeaponId::Glock,
            x if x == WeaponId::Ak47 as u32 => WeaponId::Ak47,
            x if x == WeaponId::Aug as u32 => WeaponId::Aug,
            x if x == WeaponId::Awp as u32 => WeaponId::Awp,
            x if x == WeaponId::Famas as u32 => WeaponId::Famas,
            x if x == WeaponId::G3SG1 as u32 => WeaponId::G3SG1,
            x if x == WeaponId::GalilAr as u32 => WeaponId::GalilAr,
            x if x == WeaponId::M249 as u32 => WeaponId::M249,
            x if x == WeaponId::M4A1 as u32 => WeaponId::M4A1,
            x if x == WeaponId::Mac10 as u32 => WeaponId::Mac10,
            x if x == WeaponId::P90 as u32 => WeaponId::P90,
            x if x == WeaponId::ZoneRepulsor as u32 => WeaponId::ZoneRepulsor,
            x if x == WeaponId::Mp5sd as u32 => WeaponId::Mp5sd,
            x if x == WeaponId::Ump45 as u32 => WeaponId::Ump45,
            x if x == WeaponId::Xm1014 as u32 => WeaponId::Xm1014,
            x if x == WeaponId::Bizon as u32 => WeaponId::Bizon,
            x if x == WeaponId::Mag7 as u32 => WeaponId::Mag7,
            x if x == WeaponId::Negev as u32 => WeaponId::Negev,
            x if x == WeaponId::Sawedoff as u32 => WeaponId::Sawedoff,
            x if x == WeaponId::Taser as u32 => WeaponId::Taser,
            x if x == WeaponId::Hkp2000 as u32 => WeaponId::Hkp2000,
            x if x == WeaponId::Mp7 as u32 => WeaponId::Mp7,
            x if x == WeaponId::Mp9 as u32 => WeaponId::Mp9,
            x if x == WeaponId::Nova as u32 => WeaponId::Nova,
            x if x == WeaponId::P250 as u32 => WeaponId::P250,
            x if x == WeaponId::Shield as u32 => WeaponId::Shield,
            x if x == WeaponId::Scar20 as u32 => WeaponId::Scar20,
            x if x == WeaponId::Sg553 as u32 => WeaponId::Sg553,
            x if x == WeaponId::Ssg08 as u32 => WeaponId::Ssg08,
            x if x == WeaponId::Flashbang as u32 => WeaponId::Flashbang,
            x if x == WeaponId::HeGrenade as u32 => WeaponId::HeGrenade,
            x if x == WeaponId::SmokeGrenade as u32 => WeaponId::SmokeGrenade,
            x if x == WeaponId::Molotov as u32 => WeaponId::Molotov,
            x if x == WeaponId::Decoy as u32 => WeaponId::Decoy,
            x if x == WeaponId::IncGrenade as u32 => WeaponId::IncGrenade,
            x if x == WeaponId::C4 as u32 => WeaponId::C4,
            x if x == WeaponId::Healthshot as u32 => WeaponId::Healthshot,
            x if x == WeaponId::M4a1s as u32 => WeaponId::M4a1s,
            x if x == WeaponId::Usps as u32 => WeaponId::Usps,
            x if x == WeaponId::Cz75a as u32 => WeaponId::Cz75a,
            x if x == WeaponId::Revolver as u32 => WeaponId::Revolver,
            x if x == WeaponId::TaGrenade as u32 => WeaponId::TaGrenade,
            x if x == WeaponId::Axe as u32 => WeaponId::Axe,
            x if x == WeaponId::Hammer as u32 => WeaponId::Hammer,
            x if x == WeaponId::Spanner as u32 => WeaponId::Spanner,
            x if x == WeaponId::Firebomb as u32 => WeaponId::Firebomb,
            x if x == WeaponId::Diversion as u32 => WeaponId::Diversion,
            x if x == WeaponId::FragGrenade as u32 => WeaponId::FragGrenade,
            x if x == WeaponId::Snowball as u32 => WeaponId::Snowball,
            x if x == WeaponId::BumpMine as u32 => WeaponId::BumpMine,

            // return all types of knife as base knife
            x if x == WeaponId::GhostKnife as u32 => WeaponId::Knife,
            x if x == WeaponId::Knife as u32 => WeaponId::Knife,
            x if x == WeaponId::GoldenKnife as u32 => WeaponId::Knife,
            x if x == WeaponId::KnifeT as u32 => WeaponId::Knife,
            x if x >= 500 => WeaponId::Knife,
            _ => WeaponId::None,
        }
    }
}

// writes out the debug name as a string for normal formatting
// also allows .to_string() to be used
// https://stackoverflow.com/questions/32710187/how-do-i-get-an-enum-as-a-string
impl fmt::Display for WeaponId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug,Serialize, Deserialize, Eq, PartialEq, Copy, Clone, Hash)]
#[serde(tag = "weapontype")]
pub enum WeaponId {
    None = 0,
    Deagle = 1,
    Elite,
    Fiveseven,
    Glock,
    Ak47 = 7,
    Aug,
    Awp,
    Famas,
    G3SG1,
    GalilAr = 13,
    M249,
    M4A1 = 16,
    Mac10,
    P90 = 19,
    ZoneRepulsor,
    Mp5sd = 23,
    Ump45,
    Xm1014,
    Bizon,
    Mag7,
    Negev,
    Sawedoff,
    Tec9,
    Taser,
    Hkp2000,
    Mp7,
    Mp9,
    Nova,
    P250,
    Shield,
    Scar20,
    Sg553,
    Ssg08,
    GoldenKnife,
    Knife,
    Flashbang = 43,
    HeGrenade,
    SmokeGrenade,
    Molotov,
    Decoy,
    IncGrenade,
    C4,
    Healthshot = 57,
    KnifeT = 59,
    M4a1s,
    Usps,
    Cz75a = 63,
    Revolver,
    TaGrenade = 68,
    Axe = 75,
    Hammer,
    Spanner = 78,
    GhostKnife = 80,
    Firebomb,
    Diversion,
    FragGrenade,
    Snowball,
    BumpMine,
    Bayonet = 500,
    ClassicKnife = 503,
    Flip = 505,
    Gut,
    Karambit,
    M9Bayonet,
    Huntsman,
    Falchion = 512,
    Bowie = 514,
    Butterfly,
    Daggers,
    Paracord,
    SurvivalKnife,
    Ursus = 519,
    Navaja,
    NomadKnife,
    Stiletto = 522,
    Talon,
    SkeletonKnife = 525,
    NameTag = 1200,
    Sticker = 1209,
    MusicKit = 1314,
    SealedGraffiti = 1348,
    Graffiti = 1349,
    OperationHydraPass = 1352,
    BronzeOperationHydraCoin = 4353,
    Patch = 4609,
    Berlin2019SouvenirToken = 4628,
    GloveStuddedBrokenfang = 4725,
    Stockholm2021SouvenirToken = 4802,
    GloveStuddedBloodhound = 5027,
    GloveT,
    GloveCT,
    GloveSporty,
    GloveSlick,
    GloveLeatherWrap,
    GloveMotorcycle,
    GloveSpecialist,
    GloveHydra
}