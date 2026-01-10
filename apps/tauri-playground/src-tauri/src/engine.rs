use crate::types::PlaygroundNodeData;
use ferroflux_sdk::FerroFluxClient;
use flow_canvas::model::GraphState;
use tokio::sync::{mpsc, oneshot};

pub enum EngineCommand {
    Init(oneshot::Sender<Result<(), String>>),
    Deploy(
        GraphState<PlaygroundNodeData>,
        oneshot::Sender<Result<(), String>>,
    ),
    GetTemplates(oneshot::Sender<Result<Vec<crate::types::NodeTemplate>, String>>),
    ReloadDefinitions(oneshot::Sender<Result<(), String>>),
    SimulateNode(
        flow_canvas::model::Uuid,
        String,                                               // definition_id
        std::collections::HashMap<String, serde_json::Value>, // config
        serde_json::Value,
        std::collections::HashMap<String, ferroflux_core::components::shadow::MockConfig>,
        oneshot::Sender<Result<serde_json::Value, String>>,
    ),
}

pub fn spawn_engine_thread(mut engine_rx: mpsc::Receiver<EngineCommand>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let mut client: Option<FerroFluxClient<PlaygroundNodeData>> = None;
            while let Some(cmd) = engine_rx.recv().await {
                match cmd {
                    // ... (other commands same)
                    EngineCommand::Init(tx) => {
                        if client.is_none() {
                            match FerroFluxClient::start().await {
                                Ok((c, _handle)) => {
                                    client = Some(c);
                                    let _ = tx.send(Ok(()));
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(e.to_string()));
                                }
                            }
                        } else {
                            let _ = tx.send(Ok(()));
                        }
                    }
                    EngineCommand::Deploy(graph, tx) => {
                        if let Some(c) = client.as_mut() {
                            let res = c.sync_graph(&graph).await;
                            let _ = tx.send(res.map_err(|e| e.to_string()));
                        } else {
                            let _ = tx.send(Err("Client not initialized".to_string()));
                        }
                    }
                    EngineCommand::GetTemplates(tx) => {
                        if let Some(c) = client.as_ref() {
                            match c.get_node_templates().await {
                                Ok(t) => {
                                    let _ = tx.send(Ok(t));
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(e.to_string()));
                                }
                            }
                        } else {
                            let _ = tx.send(Err("Client not initialized".to_string()));
                        }
                    }
                    EngineCommand::ReloadDefinitions(tx) => {
                        if let Some(c) = client.as_ref() {
                            let res = c.reload_definitions().await;
                            let _ = tx.send(res.map_err(|e| e.to_string()));
                        } else {
                            let _ = tx.send(Err("Client not initialized".to_string()));
                        }
                    }
                    EngineCommand::SimulateNode(node_id, def_id, config, payload, mocks, tx) => {
                        if let Some(c) = client.as_mut() {
                            let res = c
                                .simulate_and_wait(node_id, def_id, config, payload, mocks)
                                .await;
                            let _ = tx.send(res.map_err(|e| e.to_string()));
                        } else {
                            let _ = tx.send(Err("Client not initialized".to_string()));
                        }
                    }
                }
            }
        });
    });
}
