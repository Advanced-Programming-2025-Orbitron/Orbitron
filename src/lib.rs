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

    Planet::new(
        planet_id,
        planet_type,
        ai,
        gen_rules,
        comb_rules,
        (from_orchestrator, to_orchestrator),
        from_explorer,
    )
    .unwrap()
}

// Test for create planet sections

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;

    // Helper function to create test channels
    fn setup_test_channels() -> (
        Receiver<OrchestratorToPlanet>,
        Sender<PlanetToOrchestrator>,
        Receiver<ExplorerToPlanet>,
        Sender<OrchestratorToPlanet>,
        Receiver<PlanetToOrchestrator>,
        Sender<ExplorerToPlanet>,
    ) {
        let (tx_orch_to_planet, rx_orch_to_planet) = unbounded::<OrchestratorToPlanet>();
        let (tx_planet_to_orch, rx_planet_to_orch) = unbounded::<PlanetToOrchestrator>();
        let (tx_expl_to_planet, rx_expl_to_planet) = unbounded::<ExplorerToPlanet>();

        (
            rx_orch_to_planet,
            tx_planet_to_orch,
            rx_expl_to_planet,
            tx_orch_to_planet,
            rx_planet_to_orch,
            tx_expl_to_planet,
        )
    }
    // UNIT tests for creating planet
    #[test]
    fn test_create_planet_returns_valid_planet() {
        let (rx_orch, tx_orch, rx_expl, _, _, _) = setup_test_channels();
        let planet_id = 42;
        let planet = create_planet(rx_orch, tx_orch, rx_expl, planet_id);
        // Planet id
        assert_eq!(planet.id(), planet_id);
        // Does planet returning  planet type B?
        assert_eq!(format!("{:?}", planet.planet_type()), "B");
    }
}
