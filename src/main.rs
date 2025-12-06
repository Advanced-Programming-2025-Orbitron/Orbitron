// src/main.rs

use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use std::sync::mpsc;

mod ai;
use ai::orbitron::MyPlanetAI;

fn main() {
    // let rx_orchestrator =
    //     mpsc::Receiver::<common_game::protocols::messages::OrchestratorToPlanet>::try_from_stdin()
    //         .expect("Failed to get orchestrator receiver");
    // let tx_orchestrator =
    //     mpsc::Sender::<common_game::protocols::messages::PlanetToOrchestrator>::try_from_stdout()
    //         .expect("Failed to get orchestrator sender");
    // let rx_explorer =
    //     mpsc::Receiver::<common_game::protocols::messages::ExplorerToPlanet>::try_from_fd(3)
    //         .expect("Failed to get explorer receiver");

    // let planet = create_planet(rx_orchestrator, tx_orchestrator, rx_explorer);

    // if let Err(e) = planet.run() {
    //     eprintln!("Planet {} crashed: {}", planet.id(), e);
    // }
}

pub fn create_planet(
    rx_orchestrator: mpsc::Receiver<common_game::protocols::messages::OrchestratorToPlanet>,
    tx_orchestrator: mpsc::Sender<common_game::protocols::messages::PlanetToOrchestrator>,
    rx_explorer: mpsc::Receiver<common_game::protocols::messages::ExplorerToPlanet>,
    tx_explorer: mpsc::Sender<common_game::protocols::messages::PlanetToExplorer>,
) -> Planet {
    let planet_type = PlanetType::B;
    let gen_rules = vec![BasicResourceType::Hydrogen, BasicResourceType::Oxygen, BasicResourceType::Carbon, BasicResourceType::Silicon];
    let comb_rules = vec![ComplexResourceType::Water];

    let ai = Box::new(MyPlanetAI::new());

    Planet::new(
        0,
        planet_type,
        ai,
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        (rx_explorer, tx_explorer),
    )
    .expect("Invalid planet configuration – check constraints!")
}
