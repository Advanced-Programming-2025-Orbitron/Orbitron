//! Creates and configures the Orbitron planet.
//!
//! This module exposes the [`create_planet`] function, which the orchestrator
//! calls to spawn an instance of the Orbitron planet. It sets up:
//! - the planet type,
//! - its AI implementation, which implements the [`PlanetAI`] trait,
//! - resource generation and combination rules,
//! - and communication channels to/from orchestrator and explorers.
//!
//! The resulting configuration is passed to [`Planet::new`], which returns a
//! fully-initialized [`Planet`] instance or reports configuration errors.
use common_game::components::planet::{Planet, PlanetAI, PlanetType};
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
/// If the configuration violates any game constraints, the function will panic.
///
/// # Panics
/// Panics with `"Invalid planet configuration – check constraints!"`  
/// if `Planet::new` rejects the provided rules or AI.
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
    .expect("Invalid planet configuration – check constraints!")
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
    // Test for  Combination rules
    #[test]
    fn test_create_planet_has_correct_combination_rules() {
        let (rx_orch, tx_orch, rx_expl, _, _, _) = setup_test_channels();

        let planet = create_planet(rx_orch, tx_orch, rx_expl, 1);

        let available_combinations = planet.combinator().all_available_recipes();

        // Should have Water combination
        assert_eq!(available_combinations.len(), 1);
        assert!(available_combinations.contains(&ComplexResourceType::Water));
    }
    #[test]
    #[test]
    fn test_create_planet_can_generate_resources() {
        let (rx_orch, tx_orch, rx_expl, tx_to_planet, rx_from_planet, _) = setup_test_channels();
        let planet_id = 20;

        let mut planet = create_planet(rx_orch, tx_orch, rx_expl, planet_id);

        // Spawn planet in a thread
        let handle = thread::spawn(move || planet.run());

        // Start the planet
        tx_to_planet
            .send(OrchestratorToPlanet::StartPlanetAI)
            .unwrap();
        let _ = rx_from_planet.recv_timeout(Duration::from_millis(100));

        // Charge a cell
        tx_to_planet
            .send(OrchestratorToPlanet::Sunray(Sunray::new()))
            .unwrap();

        // Wait for SunrayAck
        match rx_from_planet.recv_timeout(Duration::from_millis(100)) {
            Ok(PlanetToOrchestrator::SunrayAck { planet_id: id }) => {
                assert_eq!(id, planet_id);
            }
            _ => panic!("Expected SunrayAck"),
        }

        // Clean up
        tx_to_planet.send(OrchestratorToPlanet::KillPlanet).unwrap();

        let _ = handle.join();
    }

    #[test]
    fn test_create_planet_type_b_cannot_survive_asteroid() {
        let (rx_orch, tx_orch, rx_expl, tx_to_planet, rx_from_planet, _) = setup_test_channels();
        let planet_id = 42;

        let mut planet = create_planet(rx_orch, tx_orch, rx_expl, planet_id);

        // Spawn planet in a thread
        let handle = thread::spawn(move || planet.run());

        // Start the planet
        tx_to_planet
            .send(OrchestratorToPlanet::StartPlanetAI)
            .unwrap();
        let _ = rx_from_planet.recv_timeout(Duration::from_millis(100));

        // Charge the cell
        tx_to_planet
            .send(OrchestratorToPlanet::Sunray(Sunray::new()))
            .unwrap();
        let _ = rx_from_planet.recv_timeout(Duration::from_millis(100));

        // Send asteroid
        tx_to_planet
            .send(OrchestratorToPlanet::Asteroid(Asteroid::new()))
            .unwrap();

        // Type B cannot have rockets, so should return None
        match rx_from_planet.recv_timeout(Duration::from_millis(100)) {
            Ok(PlanetToOrchestrator::AsteroidAck {
                planet_id: id,
                rocket,
            }) => {
                assert_eq!(id, planet_id);
                assert!(
                    rocket.is_none(),
                    "Type B should not be able to build rockets"
                );
            }
            _ => panic!("Expected AsteroidAck"),
        }
        // Clean up
        tx_to_planet.send(OrchestratorToPlanet::KillPlanet).unwrap();

        let _ = handle.join();
    }
}
