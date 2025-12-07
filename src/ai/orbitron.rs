use common_game::components::energy_cell::EnergyCell;
use common_game::components::planet::{PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{BasicResourceType, Combinator, Generator};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::*;
pub struct MyPlanetAI {
    is_started: bool,
    is_stopped: bool,
}

impl MyPlanetAI {
    pub fn new() -> Self {
        Self {
            is_started: false,
            is_stopped: false,
        }
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
            OrchestratorToPlanet::Sunray(sunray) => {
                let response = state.charge_cell(sunray);
                match response {
                    None => Some(PlanetToOrchestrator::SunrayAck {
                        planet_id: state.id(),
                    }),
                    Some(sunray) => None,
                }
            }

            OrchestratorToPlanet::InternalStateRequest => {
                Some(PlanetToOrchestrator::InternalStateResponse {
                    planet_id: state.id(),
                    planet_state: state.to_dummy(),
                })
            }
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
        None
    }

    fn start(&mut self, _state: &PlanetState) {
        self.is_started = true;
    }

    fn stop(&mut self, _state: &PlanetState) {
        self.is_stopped = true;
    }
}
