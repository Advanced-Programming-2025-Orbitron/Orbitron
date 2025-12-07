use common_game::components::planet::{PlanetAI, PlanetState};
use common_game::components::resource::{
    BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest, Generator,
    GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::protocols::messages::*;
pub struct Orbitron {
    is_stopped: bool,
}

impl Orbitron {
    pub fn new() -> Self {
        Self { is_stopped: true }
    }
}

impl PlanetAI for Orbitron {
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
                    Some(_) => None,
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
                let generated_resource = state.full_cell().and_then(|(cell, _)| match resource {
                    BasicResourceType::Hydrogen => generator
                        .make_hydrogen(cell)
                        .ok()
                        .map(|hydrogen| hydrogen.to_basic()),
                    BasicResourceType::Oxygen => generator
                        .make_oxygen(cell)
                        .ok()
                        .map(|oxygen| oxygen.to_basic()),
                    _ => None,
                });

                Some(PlanetToExplorer::GenerateResourceResponse {
                    resource: generated_resource,
                })
            }

            ExplorerToPlanet::CombineResourceRequest {
                explorer_id: _,
                msg,
            } => {
                let cell = state.full_cell();

                let ret: Result<ComplexResource, (String, GenericResource, GenericResource)> =
                    match msg {
                        ComplexResourceRequest::Water(resource_1, resource_2) => match cell {
                            Some((cell, _)) => combinator
                                .make_water(resource_1, resource_2, cell)
                                .map(|water| water.to_complex())
                                .map_err(|(err_str, return_resource_1, return_resource_2)| {
                                    (
                                        err_str,
                                        return_resource_1.to_generic(),
                                        return_resource_2.to_generic(),
                                    )
                                }),
                            None => Err((
                                "No charged energy cell found".to_string(),
                                resource_1.to_generic(),
                                resource_2.to_generic(),
                            )),
                        },

                        other => {
                            let variant_name = format!("{other:?}");

                            let (resource_1, resource_2) = match other {
                                ComplexResourceRequest::Diamond(r1, r2) => {
                                    (r1.to_generic(), r2.to_generic())
                                }
                                ComplexResourceRequest::Life(r1, r2) => {
                                    (r1.to_generic(), r2.to_generic())
                                }
                                ComplexResourceRequest::Robot(r1, r2) => {
                                    (r1.to_generic(), r2.to_generic())
                                }
                                ComplexResourceRequest::Dolphin(r1, r2) => {
                                    (r1.to_generic(), r2.to_generic())
                                }
                                ComplexResourceRequest::AIPartner(r1, r2) => {
                                    (r1.to_generic(), r2.to_generic())
                                }
                                ComplexResourceRequest::Water(_, _) => {
                                    unreachable!("Water is handled above")
                                }
                            };

                            Err((
                                format!("There isn't a recipe for {variant_name:?}"),
                                resource_1,
                                resource_2,
                            ))
                        }
                    };

                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: ret,
                })
            }

            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: state.cells_count() as u32,
                })
            }
        }
    }

    fn handle_asteroid(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        None
    }

    fn start(&mut self, _state: &PlanetState) {
        self.is_stopped = false;
    }

    fn stop(&mut self, _state: &PlanetState) {
        self.is_stopped = true;
    }
}
