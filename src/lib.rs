//! Creates and configures the Orbitron planet.
//!
//! This module exposes the [`create_planet`] function, which the orchestrator
//! calls to spawn an instance of the Orbitron planet. It sets up:
//! - the planet type B,
//! - its AI implementation, which implements the `PlanetAI` trait,
//! - resource generation for `Hydrogyn` and `Oxygen`
//! - combination rules for `Water`
//! - and communication channels to/from orchestrator and explorers.
//!
//! The resulting configuration is passed to [`Planet::new`], which returns a
//! fully-initialized [`Planet`] instance or reports configuration errors.
#![allow(rustdoc::private_intra_doc_links)]
use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::logging::*;
use common_game::protocols::messages::*;
use crossbeam_channel::{Receiver, Sender};

mod ai;
use ai::orbitron::Orbitron;

/// Creates and initializes an Orbitron planet.
///
/// # Parameters
/// - `from_orchestrator`: channel receiving messages sent **to** the planet by the orchestrator  
/// - `to_orchestrator`: channel used by the planet to send messages **back** to the orchestrator  
/// - `from_explorer`: channel receiving messages sent by explorers  
/// - `planet_id`: unique numeric identifier assigned by the orchestrator
///
/// # Behavior
/// This function configures:
/// - `PlanetType::B` as the type of the Orbitron planet  
/// - Hydrogen + Oxygen as basic resource generation rules  
/// - Water as the combination rule  
/// - [`Orbitron`] as the AI controlling this planet  
///
/// The function returns a fully constructed [`Planet`] instance.  
pub fn create_planet(
    from_orchestrator: Receiver<OrchestratorToPlanet>,
    to_orchestrator: Sender<PlanetToOrchestrator>,
    from_explorer: Receiver<ExplorerToPlanet>,
    planet_id: u32,
) -> Planet {
    let planet_type = PlanetType::B;
    // Basic resources this planet can generate on its own.
    let gen_rules = vec![BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
    // Complex resources that can be formed from combinations.
    let comb_rules = vec![ComplexResourceType::Water];
    // AI logic controlling the planet's behavior.
    let ai: Box<Orbitron> = Box::new(Orbitron::new());

    let planet = Planet::new(
        planet_id,
        planet_type,
        ai,
        gen_rules,
        comb_rules,
        (from_orchestrator, to_orchestrator),
        from_explorer,
    )
    .unwrap();

    // Optional: log planet creation
    let mut payload = Payload::new();
    payload.insert("planet_id".into(), planet_id.to_string());
    payload.insert("planet_type".into(), format!("{planet_type:?}"));
    LogEvent::new(
        ActorType::User,
        2_u32,
        ActorType::Planet,
        planet_id.to_string(),
        EventType::InternalPlanetAction,
        Channel::Info,
        payload,
    )
    .emit();

    planet
}
