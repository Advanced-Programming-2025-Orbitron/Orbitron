use common_game::components::energy_cell::EnergyCell;
use common_game::components::planet::{PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{
    BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest,
    ComplexResourceType, Generator, GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::*;
pub struct OrbitronAI {
    is_started: bool,
    is_stopped: bool,
}

impl OrbitronAI {
    pub fn new() -> Self {
        Self {
            is_started: false,
            is_stopped: false,
        }
    }
}

impl PlanetAI for OrbitronAI {
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
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: combinator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: _,
                resource,
            } => {
                let cell = state.full_cell();

                let ret: Option<BasicResource> = match cell {
                    Some((cell, _)) => match resource {
                        BasicResourceType::Hydrogen => {
                            let hydrogen = generator.make_hydrogen(cell).ok()?;
                            Some(hydrogen.to_basic())
                        }
                        BasicResourceType::Oxygen => {
                            let oxygen = generator.make_oxygen(cell).ok()?;
                            Some(oxygen.to_basic())
                        }
                        BasicResourceType::Carbon => {
                            let carbon = generator.make_carbon(cell).ok()?;
                            Some(carbon.to_basic())
                        }
                        BasicResourceType::Silicon => {
                            let silicon = generator.make_silicon(cell).ok()?;
                            Some(silicon.to_basic())
                        }
                    },
                    None => None,
                };

                Some(PlanetToExplorer::GenerateResourceResponse { resource: ret })
            }

            ExplorerToPlanet::CombineResourceRequest {
                explorer_id: _,
                msg,
            } => {
                let cell = state.full_cell();

                let ret: Result<ComplexResource, (String, GenericResource, GenericResource)> =
                    match msg {
                        ComplexResourceRequest::Water(r1, r2) => match cell {
                            Some((cell, _)) => match combinator.make_water(r1, r2, cell) {
                                Ok(water) => Ok(water.to_complex()),
                                Err((str, r1e, r2e)) => {
                                    Err((str, r1e.to_generic(), r2e.to_generic()))
                                }
                            },
                            None => Err((
                                "Not enough energy cell!".to_string(),
                                r1.to_generic(),
                                r2.to_generic(),
                            )),
                        },

                        ComplexResourceRequest::Diamond(r1, r2) => Err((
                            "There isn't a recipe for Diamond".to_string(),
                            r1.to_generic(),
                            r2.to_generic(),
                        )),

                        ComplexResourceRequest::Life(r1, r2) => Err((
                            "There isn't a recipe for Life".to_string(),
                            r1.to_generic(),
                            r2.to_generic(),
                        )),

                        ComplexResourceRequest::Robot(r1, r2) => Err((
                            "There isn't a recipe for Robot".to_string(),
                            r1.to_generic(),
                            r2.to_generic(),
                        )),

                        ComplexResourceRequest::Dolphin(r1, r2) => Err((
                            "There isn't a recipe for Dolphin".to_string(),
                            r1.to_generic(),
                            r2.to_generic(),
                        )),

                        ComplexResourceRequest::AIPartner(r1, r2) => Err((
                            "There isn't a recipe for AIPartner".to_string(),
                            r1.to_generic(),
                            r2.to_generic(),
                        )),
                    };

                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: ret,
                })
            }

            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: u32 } => {
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: state.cells_count() as u32,
                })
            }
        }
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
