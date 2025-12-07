// src/main.rs

use common_game::components::planet::{Planet, PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{
    BasicResourceType, Combinator, ComplexResourceType, Generator,
};
use common_game::components::rocket::Rocket;
use common_game::protocols::messages::*;
use crossbeam_channel::{Receiver, Sender, bounded};

mod ai;
use ai::orbitron::OrbitronAI;

fn get_test_channels() -> (
    Receiver<OrchestratorToPlanet>,
    Sender<PlanetToOrchestrator>,
    Receiver<ExplorerToPlanet>,
) {
    let (tx_orch_to_planet, rx_orch_to_planet) = bounded::<OrchestratorToPlanet>(100);

    let (tx_planet_to_orch, rx_planet_to_orch) = bounded::<PlanetToOrchestrator>(100);

    let (tx_expl_to_planet, rx_expl_to_planet) = bounded::<ExplorerToPlanet>(100);

    (rx_orch_to_planet, tx_planet_to_orch, rx_expl_to_planet)
}

fn main() {
    let channels = get_test_channels();

    let mut planet = create_planet(channels.0, channels.1, channels.2);

    if let Err(e) = planet.run() {
        eprintln!("Planet {} crashed: {}", planet.id(), e);
    }
}

pub fn create_planet(
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
) -> Planet {
    let planet_type = PlanetType::B;
    let gen_rules = vec![
        BasicResourceType::Hydrogen,
        BasicResourceType::Oxygen,
        BasicResourceType::Carbon,
        BasicResourceType::Silicon,
    ];
    let comb_rules = vec![ComplexResourceType::Water];

    let ai: Box<OrbitronAI> = Box::new(OrbitronAI::new());

    Planet::new(
        0,
        planet_type,
        ai,
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    )
    .expect("Invalid planet configuration – check constraints!")
}
