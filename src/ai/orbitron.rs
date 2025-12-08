//! # Orbitron – Planet AI Module
//!
//! This module implements the [PlanetAI] trait for the Orbitron planet.  
//! It defines how the planet reacts to all types of messages coming from both
//! the Orchestrator and Explorers, and how it handles internal game
//! logic such as resource generation, resource combination, energy management,
//! rocket construction, and survival after asteroid impacts.
//!
//! ## Responsibilities
//!
//! The Orbitron AI controls:
//!
//! - Sunray handling
//!   Charges energy cells when possible, otherwise replies with `SunrayAck`.
//!
//! - Internal state reporting*
//!   Returns a snapshot of the current [PlanetState] when requested.
//!
//! - Explorer interactions
//!   * Supported recipes from the [Generator] and [Combinator]  
//!   * Resource generation requests  
//!   * Resource combination (including error reporting)  
//!   * Energy cell availability
//!
//! - Asteroid survival logic  
//!   Attempts to build a [Rocket] and return it.  
//!   If no rocket is returned, the planet is destroyed.
//!
//! - Lifecycle control  
//!   Handles `StartPlanetAI` and `StopPlanetAI` messages, enabling
//!   or disabling the decision-making logic.
use common_game::components::planet::{PlanetAI, PlanetState};
use common_game::components::resource::{
    BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest, Generator,
    GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::logging::*;
use common_game::protocols::messages::*;

/// Helper functions to convert messages and responses into string names
fn orchestrator_to_planet_name(msg: &OrchestratorToPlanet) -> String {
    match msg {
        OrchestratorToPlanet::Sunray(_) => "Sunray".into(),
        OrchestratorToPlanet::InternalStateRequest => "InternalStateRequest".into(),
        _ => "Other".into(),
    }
}

fn planet_to_orchestrator_name(msg: &PlanetToOrchestrator) -> String {
    match msg {
        PlanetToOrchestrator::SunrayAck { .. } => "SunrayAck".into(),
        PlanetToOrchestrator::InternalStateResponse { .. } => "InternalStateResponse".into(),
        _ => "Other".into(),
    }
}

fn explorer_to_planet_name(msg: &ExplorerToPlanet) -> String {
    match msg {
        ExplorerToPlanet::SupportedResourceRequest { .. } => "SupportedResourceRequest".into(),
        ExplorerToPlanet::SupportedCombinationRequest { .. } => {
            "SupportedCombinationRequest".into()
        }
        ExplorerToPlanet::GenerateResourceRequest { .. } => "GenerateResourceRequest".into(),
        ExplorerToPlanet::CombineResourceRequest { .. } => "CombineResourceRequest".into(),
        ExplorerToPlanet::AvailableEnergyCellRequest { .. } => "AvailableEnergyCellRequest".into(),
    }
}

fn planet_to_explorer_name(msg: &PlanetToExplorer) -> String {
    match msg {
        PlanetToExplorer::SupportedResourceResponse { .. } => "SupportedResourceResponse".into(),
        PlanetToExplorer::SupportedCombinationResponse { .. } => {
            "SupportedCombinationResponse".into()
        }
        PlanetToExplorer::GenerateResourceResponse { .. } => "GenerateResourceResponse".into(),
        PlanetToExplorer::CombineResourceResponse { .. } => "CombineResourceResponse".into(),
        PlanetToExplorer::AvailableEnergyCellResponse { .. } => {
            "AvailableEnergyCellResponse".into()
        }
        _ => "Other".into(),
    }
}

/// Represents the AI controller for the Orbitron planet.
///
/// The `is_stopped` flag indicates whether the planet's AI is currently
/// inactive and should ignore incoming logic or requests.
pub struct Orbitron {
    is_stopped: bool,
}

/// Creates a new `Orbitron` AI instance.
///
/// By default, the AI starts in the stopped state and will only
/// begin processing once explicitly started.
impl Orbitron {
    pub fn new() -> Self {
        Self { is_stopped: true }
    }
}

impl PlanetAI for Orbitron {
    /// Handler for **all** messages received by an orchestrator (receiving
    /// end of the [OrchestratorToPlanet] channel).
    ///
    /// [OrchestratorToPlanet::Sunray]-This variant is used to handle Sunray msg
    /// # Returns
    /// SunrayAck indicates Sunray is not consumed due to full energy cells
    /// None indicates energy cell received Sunray
    ///
    /// [OrchestratorToPlanet::InternalStateRequest]-This variant is used to handle InternalStateRequest msg
    /// # Returns
    /// Planet id and planet state
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        // LOG incoming orchestrator message
        let mut p = Payload::new();
        p.insert("msg".into(), orchestrator_to_planet_name(&msg));
        LogEvent::new(
            ActorType::Orchestrator,
            '0',
            ActorType::Planet,
            state.id().to_string(),
            EventType::MessageOrchestratorToPlanet,
            Channel::Info,
            p,
        )
        .emit();

        let response = match msg {
            OrchestratorToPlanet::Sunray(sunray) => {
                let res = state.charge_cell(sunray);
                match res {
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
        };

        // Log the response if any
        if let Some(ref r) = response {
            let mut p = Payload::new();
            p.insert("response".into(), planet_to_orchestrator_name(r));
            LogEvent::new(
                ActorType::Planet,
                state.id(),
                ActorType::Orchestrator,
                '0',
                EventType::MessagePlanetToOrchestrator,
                Channel::Info,
                p,
            )
            .emit();
        }

        response
    }
    /// Handles messages from explorers.
    ///
    /// - Provides supported basic and complex resource types
    /// - Generates requested basic resources (Hydrogen or Oxygen).  
    ///   First, we check whether there is any charged cell (the `full_cell` function does this).  
    ///   If there is, we then check whether the requested `BasicResourceType` is Hydrogen or Oxygen.  
    ///   If it is, we generate it; otherwise, the function returns `None`.
    /// - Generates Water as the only supported complex resource.  
    ///   As before, we must check whether there is a charged cell.  
    ///   Since the planet can only generate water, if the requested complex resource type is `Water`,
    ///   we proceed with generation; otherwise, we return an error message.
    /// - Returns the number of available charged energy cells.
    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        // Logging incoming explorer message
        let mut p = Payload::new();
        p.insert("msg".into(), explorer_to_planet_name(&msg));
        LogEvent::new(
            ActorType::Explorer,
            1_u32,
            ActorType::Planet,
            state.id().to_string(),
            EventType::MessageExplorerToPlanet,
            Channel::Info,
            p,
        )
        .emit();

        let response = match msg {
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
            // This variant is used to ask the Planet to generate a BasicResource
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: _,
                resource,
            } => {
                // First, we need to check whether there is any charged cell (the `full_cell` function does this).
                // If there is, we then check whether the requested `BasicResourceType` is Hydrogen or Oxygen.
                // If it is, we generate it; otherwise, the function returns `None`.
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

            // This variant is used to ask the Planet to generate a ComplexResource using the ComplexResourceRequest]
            ExplorerToPlanet::CombineResourceRequest {
                explorer_id: _,
                msg,
            } => {
                // Same as previous function, we neeed to know if we have any charged cell or not.
                let cell = state.full_cell();

                let ret: Result<ComplexResource, (String, GenericResource, GenericResource)> =
                    match msg {
                        // Here we match the requested complex resource type.
                        // Since our planet can only generate water, if the requested complex resource type is Water,
                        // we check whether there is any charged cell.
                        // Otherwise, we return an error message.
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
        };

        if let Some(ref r) = response {
            let mut p = Payload::new();
            p.insert("response".into(), planet_to_explorer_name(r));
            LogEvent::new(
                ActorType::Planet,
                state.id(),
                ActorType::Explorer,
                '1',
                EventType::MessagePlanetToExplorer,
                Channel::Info,
                p,
            )
            .emit();
        }

        response
    }
    /// This handler will be invoked when a [OrchestratorToPlanet::Asteroid]
    /// message is received.
    ///
    /// # Returns
    /// In order to survice, planet try to build rocket.
    /// After this attempt an owned [Rocket] must be returned from this method;
    /// if `None` is returned instead, the planet will  be destroyed by the orchestrator
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        // Log the incoming asteroid event
        let mut p = Payload::new();
        p.insert("event".into(), "AsteroidImpact".into());
        LogEvent::new(
            ActorType::Orchestrator,
            0_u32,
            ActorType::Planet,
            state.id().to_string(),
            EventType::InternalPlanetAction,
            Channel::Warning,
            p,
        )
        .emit();

        let _ = state.build_rocket(0);
        let rocket = state.take_rocket();

        let mut p = Payload::new();
        if rocket.is_some() {
            p.insert("result".into(), "RocketBuilt".into());
        } else {
            p.insert("result".into(), "RocketFailed".into());
        }
        LogEvent::new(
            ActorType::Planet,
            state.id(),
            ActorType::Orchestrator,
            '0',
            EventType::InternalPlanetAction,
            Channel::Warning,
            p,
        )
        .emit();
        rocket
    }

    /// This method will be invoked when a [OrchestratorToPlanet::StartPlanetAI]
    /// is received, but only if the planet is currently in a stopped state.
    ///
    /// Start messages received when planet is already running are ignored.
    fn start(&mut self, state: &PlanetState) {
        self.is_stopped = false;

        let mut p = Payload::new();
        p.insert("event".into(), "StartPlanetAI".into());
        LogEvent::new(
            ActorType::Orchestrator,
            0_u32,
            ActorType::Planet,
            state.id().to_string(),
            EventType::InternalPlanetAction,
            Channel::Info,
            p,
        )
        .emit();
    }

    /// This method will be invoked when a [OrchestratorToPlanet::StopPlanetAI]
    /// is received, but only if the planet is currently in a running state.
    ///
    /// Stop messages received when planet is already stopped are ignored.
    fn stop(&mut self, state: &PlanetState) {
        self.is_stopped = true;

        let mut p = Payload::new();
        p.insert("event".into(), "StopPlanetAI".into());
        LogEvent::new(
            ActorType::Orchestrator,
            0_u32,
            ActorType::Planet,
            state.id().to_string(),
            EventType::InternalPlanetAction,
            Channel::Info,
            p,
        )
        .emit();
    }
}
