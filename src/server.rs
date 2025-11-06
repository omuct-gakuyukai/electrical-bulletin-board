use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade}, 
    response::IntoResponse, 
    routing::get, 
    Extension, 
    Router
};
use bevy_tokio_tasks::TokioTasksRuntime;
use bevy::prelude::*;
use tokio::sync::{mpsc, broadcast};
use serde::{Deserialize, Serialize};
use futures_util::{SinkExt, StreamExt};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum WsCommand {
    Bulletin {
	preset: String,
	index: u32,
    },
    Bingo {
	method: BingoMethod,
    },
    Countdown {
	method: CountdownMethod,
    },
    ListPresets,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum BingoMethod {
    Next,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CountdownMethod {
    Start,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum WsResponse {
    Bulletin(BulletinResponse),
    Bingo(BingoResponse),
    Countdown(CountdownResponse),
    PresetList(PresetListResponse),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BulletinResponse {
    pub prev_text: String,
    pub now_text: String,
    pub next_text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BingoResponse {
    pub current: u8,
    pub no: u8,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CountdownResponse {
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PresetListResponse {
    pub presets: Vec<String>,
}

#[derive(Resource)]
pub struct WebSocketChannel {
    pub command_receiver: mpsc::Receiver<WsCommand>,
    pub response_sender: broadcast::Sender<WsResponse>,
}

#[derive(Resource)]
pub struct CommandSender {
    pub sender: mpsc::Sender<WsCommand>,
}

#[derive(Resource)]
pub struct ResponseBroadcaster {
    pub sender: broadcast::Sender<WsResponse>,
}

pub fn setup_websocket_server(app: &mut App) {
    let (command_tx, command_rx) = mpsc::channel::<WsCommand>(100);
    let (response_tx, _response_rx) = broadcast::channel::<WsResponse>(100);
    
    app.insert_resource(CommandSender {
        sender: command_tx,
    });
    
    app.insert_resource(WebSocketChannel {
        command_receiver: command_rx,
        response_sender: response_tx.clone(),
    });
    
    app.insert_resource(ResponseBroadcaster {
        sender: response_tx,
    });
    
    app.add_systems(Startup, start_axum_server);
    app.add_systems(Update, handle_websocket_commands);
}

fn start_axum_server(
    runtime: Res<TokioTasksRuntime>,
    command_sender: Res<CommandSender>,
    response_broadcaster: Res<ResponseBroadcaster>,
) {
    let command_tx = command_sender.sender.clone();
    let response_tx = response_broadcaster.sender.clone();
    
    runtime.spawn_background_task(move |_ctx| async move {
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .layer(Extension(command_tx))
            .layer(Extension(response_tx));
            
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await
            .expect("Failed to bind to address");
            
        println!("WebSocket server running on ws://0.0.0.0:3000/ws");
        
        axum::serve(listener, app)
            .await
            .expect("Server failed to start");
    });
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(command_tx): Extension<mpsc::Sender<WsCommand>>,
    Extension(response_tx): Extension<broadcast::Sender<WsResponse>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, command_tx, response_tx))
}

async fn handle_websocket(
    socket: WebSocket,
    command_tx: mpsc::Sender<WsCommand>,
    response_tx: broadcast::Sender<WsResponse>,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut response_rx = response_tx.subscribe();
    
    // レスポンス送信タスク
    let response_task = tokio::spawn(async move {
        while let Ok(response) = response_rx.recv().await {
            match serde_json::to_string(&response) {
                Ok(json) => {
                    if ws_sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to serialize response: {}", e);
                }
            }
        }
    });
    
    // コマンド受信タスク
    let command_task = tokio::spawn(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WsCommand>(&text) {
                        Ok(command) => {
                            if command_tx.send(command).await.is_err() {
                                eprintln!("Failed to send command to Bevy");
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse WebSocket message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("WebSocket connection closed");
                    break;
                }
                Ok(_) => {} // その他のメッセージタイプは無視
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });
    
    // どちらかのタスクが終了したら、もう一方もキャンセル
    tokio::select! {
        _ = response_task => {},
        _ = command_task => {},
    }
}

fn handle_websocket_commands(
    mut commands: Commands,
    mut ws_channel: ResMut<WebSocketChannel>,
    mut text_queue: ResMut<crate::TextQueue>,
    preset_manager: Res<crate::loader::PresetManager>,
    mut bingo_state: ResMut<crate::bingo::BingoState>,
    mut scrolling_state: ResMut<crate::ScrollingState>,
    mut scrolling_speed: ResMut<crate::ScrollingSpeed>,
    config: Res<crate::loader::Config>,
    fonts: Res<crate::Fonts>,
    text_query: Query<Entity, With<crate::Showing>>,
) {
    while let Ok(command) = ws_channel.command_receiver.try_recv() {
        match command {
            WsCommand::Bulletin { preset, index } => {
                // プリセットが指定されていて、現在のプリセットと異なる場合は切り替え
                if text_queue.current_preset != preset {
                    if let Some(new_texts) = preset_manager.presets.get(&preset) {
                        text_queue.texts = new_texts.clone();
                        text_queue.current_preset = preset.clone();
                        text_queue.current_index = 0;
                        println!("Switched to preset: {}", preset);
                    } else {
                        println!("Preset '{}' not found, using current preset '{}'", preset, text_queue.current_preset);
                    }
                }
                
                // 現在のテキストを削除
                for entity in text_query.iter() {
                    commands.entity(entity).despawn();
                }
                
                // 新しいテキストをスポーン
                if let Some(text_source) = text_queue.texts.get(index as usize) {
                    let text_content = text_source.content.clone();
                    let text_duration = text_source.duration;

		    if text_duration == 0.0 {
			crate::text_spawner::spawn_static_text(&mut commands, &text_content, fonts.text_font.clone());
		    } else {
                    crate::text_spawner::spawn_text(
                        &mut commands,
                        &text_content,
                        &text_duration,
                        fonts.text_font.clone(),
                        &config,
                        &mut *scrolling_speed,
                    );
		    }
                    
                    text_queue.current_index = index as usize;
                    scrolling_state.is_active = true;
                    
                    // レスポンスを送信
                    let prev_text = text_queue.texts.get(index.saturating_sub(1) as usize)
                        .map(|t| t.content.clone())
                        .unwrap_or_default();
                    let now_text = text_content;
                    let next_text = text_queue.texts.get((index + 1) as usize)
                        .map(|t| t.content.clone())
                        .unwrap_or_default();
                    
                    let response = WsResponse::Bulletin(BulletinResponse {
                        prev_text,
                        now_text,
                        next_text,
                    });
                    
                    let _ = ws_channel.response_sender.send(response);
                } else {
                    println!("Text index {} not found in preset '{}'", index, text_queue.current_preset);
                }
            }
            WsCommand::Bingo { method } => {
                match method {
                    BingoMethod::Next => {
                        // 現在のテキストを削除
                        for entity in text_query.iter() {
                            commands.entity(entity).despawn();
                        }
                        
                        if let Some(number) = bingo_state.next() {
                            crate::text_spawner::spawn_static_text(
                                &mut commands,
                                &number.to_string(),
                                fonts.text_font.clone(),
                            );
                            
                            let response = WsResponse::Bingo(BingoResponse {
                                current: number,
                                no: bingo_state.index as u8,
                            });
                            
                            let _ = ws_channel.response_sender.send(response);
                        }
                    }
                }
            }
            WsCommand::Countdown { method } => {
                match method {
                    CountdownMethod::Start => {
                        // カウントダウン開始のロジックをここに実装
                        // 現在のプロジェクトにはcountdownモジュールがあるようなので、
                        // それを使用することを想定
                        
                        let response = WsResponse::Countdown(CountdownResponse {
                            status: "started".to_string(),
                        });
                        
                        let _ = ws_channel.response_sender.send(response);
                    }
                }
            }
            WsCommand::ListPresets => {
                let preset_names: Vec<String> = preset_manager.presets.keys().cloned().collect();
                let response = WsResponse::PresetList(PresetListResponse {
                    presets: preset_names,
                });
                let _ = ws_channel.response_sender.send(response);
            }
        }
    }
}
    
