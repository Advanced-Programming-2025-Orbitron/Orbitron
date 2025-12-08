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
            OrchestratorToPlanet::Sunray(sunray) => {
                let response: Option<common_game::components::sunray::Sunray> =
                    state.charge_cell(sunray);
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
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }

            ExplorerToPlanet::CombineResourceRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedCombinationResponse { combination_list: 
                    
                })
            }

            _ => None,
        }
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        let _ = state.build_rocket(1);
        state.take_rocket()
    }

    fn start(&mut self, state: &PlanetState) {}

    fn stop(&mut self, state: &PlanetState) {}
}
