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
use common_game::protocols::orchestrator_planet::*;
use common_game::protocols::planet_explorer::*;
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};

mod ai;
use ai::orbitron::Orbitron;

const ORCHESTRATOR_ID: ID = 0;

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
    planet_id: ID,
) -> Planet {
    let planet_type = PlanetType::B;
    // Basic resources this planet can generate on its own.
    let gen_rules = vec![BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
    // Complex resources that can be formed from combinations.
    let comb_rules = vec![ComplexResourceType::Water];
    // AI logic controlling the planet's behavior.
    let ai: Box<Orbitron> = Box::new(Orbitron::new(planet_id));

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

    // log planet creation
    let mut payload = Payload::new();
    payload.insert("gen_rules".into(), "Hydrogen, Oxygen".into());
    payload.insert("comb_rules".into(), "Water".into());
    payload.insert("Message".into(), "New planet orbitron created".into());
    LogEvent::new(
        Some(Participant::new(ActorType::Orchestrator, ORCHESTRATOR_ID)),
        Some(Participant::new(ActorType::Planet, planet_id)),
        EventType::MessageOrchestratorToPlanet,
        Channel::Info,
        payload,
    )
    .emit();

    planet
}

// Test for create planet sections
#[cfg(test)]
mod tests {
    use super::*;
    use common_game::components::planet::PlanetState;
    use common_game::components::resource::{Combinator, Generator};
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
        // Planet should have a planet_id
        assert_eq!(planet.id(), planet_id);
        // Planet type should be B
        assert_eq!(format!("{:?}", planet.planet_type()), "B");
    }
    // Test for Type B constraints
    #[test]
    fn test_create_planet_has_correct_type_b_constraints() {
        let (rx_orch, tx_orch, rx_expl, _, _, _) = setup_test_channels();
        let planet = create_planet(rx_orch, tx_orch, rx_expl, 1);
        let available_recipes: std::collections::HashSet<BasicResourceType> =
            planet.generator().all_available_recipes();
        // Orbitron should have one energy cell
        assert_eq!(planet.state().cells_count(), 1);
        //Orbitron  should not contain rocket
        assert!(!planet.state().can_have_rocket());
        // Resource generation should contain only  Hydrogen and Oxygen
        assert_eq!(available_recipes.len(), 2);
        assert!(available_recipes.contains(&BasicResourceType::Hydrogen));
        assert!(available_recipes.contains(&BasicResourceType::Oxygen));
    }
    #[test]
    fn test_create_planet_has_correct_combination_rules() {
        let (rx_orch, tx_orch, rx_expl, _, _, _) = setup_test_channels();
        let planet = create_planet(rx_orch, tx_orch, rx_expl, 1);
        let available_combinations = planet.combinator().all_available_recipes();
        // Should have Water combination
        assert_eq!(available_combinations.len(), 1);
        assert!(available_combinations.contains(&ComplexResourceType::Water));
    }
    fn handle_explorer_msg(
        state: &PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        match msg {
            // This variant is used to ask the Planet for the available BasicResourceTypes
            ExplorerToPlanet::SupportedResourceRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }
            // This variant is used to ask the Planet for the available ComplexResourceTypes
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: combinator.all_available_recipes(),
                })
            }
            // this function returns number of cells that are charged.
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: _ } => {
                let mut cnt: u32 = 0;
                for cell in state.cells_iter() {
                    if cell.is_charged() {
                        cnt += 1;
                    }
                }
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: cnt,
                })
            }
            _ => None,
        }
    }
    fn planet_create() -> Planet {
        let (rx_orch, tx_orch, rx_expl, _, _, _) = setup_test_channels();
        let planet_id = 42;
        let planet = create_planet(rx_orch, tx_orch, rx_expl, planet_id);
        planet
    }
    #[test]
    fn test_supported_resource_request() {
        let planet = planet_create();
        let msg = ExplorerToPlanet::SupportedResourceRequest { explorer_id: 1 };
        let response =
            handle_explorer_msg(planet.state(), planet.generator(), planet.combinator(), msg);
        match response {
            Some(PlanetToExplorer::SupportedResourceResponse { resource_list }) => {
                assert_eq!(resource_list, planet.generator().all_available_recipes());
            }
            _ => panic!("Unexpected response"),
        }
    }
    #[test]
    fn test_supported_combination_request() {
        let planet = planet_create();
        let msg = ExplorerToPlanet::SupportedCombinationRequest { explorer_id: 2 };
        let response =
            handle_explorer_msg(planet.state(), planet.generator(), planet.combinator(), msg);
        match response {
            Some(PlanetToExplorer::SupportedCombinationResponse { combination_list }) => {
                assert_eq!(
                    combination_list,
                    planet.combinator().all_available_recipes()
                );
            }
            _ => panic!("Unexpected response"),
        }
    }
    #[test]
    fn test_available_energy_cell_request() {
        let planet = planet_create();
        let cnt = planet
            .state()
            .cells_iter()
            .filter(|cell| cell.is_charged())
            .count() as u32;
        let msg = ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 3 };
        let response =
            handle_explorer_msg(planet.state(), planet.generator(), planet.combinator(), msg);
        match response {
            Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: c }) => {
                assert_eq!(cnt, c);
            }
            _ => panic!("Unexpected response"),
        }
    }
}
