use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::messages::*;
use crossbeam_channel::{Receiver, Sender};

mod ai;
use ai::orbitron::Orbitron;

pub fn create_planet(
    from_orchestrator: Receiver<OrchestratorToPlanet>,
    to_orchestrator: Sender<PlanetToOrchestrator>,
    from_explorer: Receiver<ExplorerToPlanet>,
    planet_id: u32,
) -> Planet {
    let planet_type = PlanetType::B;
    let gen_rules = vec![BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
    let comb_rules = vec![ComplexResourceType::Water];
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
