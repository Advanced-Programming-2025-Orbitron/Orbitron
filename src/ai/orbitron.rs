use common_game::components::energy_cell::EnergyCell;
use common_game::components::planet::{PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{BasicResourceType, Combinator, Generator};
use common_game::components::rocket::Rocket;
use common_game::protocols::messages::*;

pub struct MyPlanetAI {}

impl MyPlanetAI {
    pub fn new() -> Self {
        Self {}
    }
}

impl PlanetAI for MyPlanetAI {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        match msg {
            OrchestratorToPlanet::Sunray(sunray) => None,
            OrchestratorToPlanet::InternalStateRequest => None,

            _ => None,
        }
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        None
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        if state.has_rocket() {
            return state.take_rocket();
        }

        if !state.can_have_rocket() {
            return None;
        }

        for i in 0..state.cells_count() {
            if state.cell(i).is_charged() && state.build_rocket(i).is_ok() {
                return state.take_rocket();
            }
        }

        None
    }

    fn start(&mut self, state: &PlanetState) {}

    fn stop(&mut self, state: &PlanetState) {}
}
