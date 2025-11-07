use bevy::{
    color::palettes::tailwind::SLATE_900,  prelude::*, text::TextLayoutInfo
};
use bevy_tokio_tasks::TokioTasksPlugin;

mod bingo;
mod loader;
mod server;
mod text;
mod text_spawner;
// mod countdown;

use loader::{Config, TextSource, PresetManager};
use bingo::BingoState;

fn main() {
    let preset_manager: PresetManager = loader::unwrap_all_presets();
    let conf: Config = loader::unwrap_conf();
    
    // デフォルトのプリセット（最初に見つかったもの、またはdefault）を取得
    let default_preset_name = preset_manager.presets.keys().next().cloned()
        .unwrap_or_else(|| "default".to_string());
    let default_texts = preset_manager.presets.get(&default_preset_name)
        .cloned()
        .unwrap_or_else(|| vec![TextSource {
            content: "No presets available".to_string(),
            duration: 5.0,
        }]);
    
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(TokioTasksPlugin::default())
        .insert_resource(ClearColor(Color::Srgba(SLATE_900)))
        .insert_resource(preset_manager)
        .insert_resource(TextQueue {
            texts: default_texts,
            current_index: 0,
            current_preset: default_preset_name,
        })
        .insert_resource(conf)
        .init_resource::<ScrollingState>()
        .init_resource::<ScrollingSpeed>()
        .init_resource::<Fonts>()
        .init_resource::<BingoState>()
        .add_systems(Startup, setup)
        .add_systems(Update, text_scroll)
        .add_systems(Update, text_loop)
        .add_systems(Update, check_text_completion)
        .add_systems(Update, handle_keyboard_action);
    
    // WebSocketサーバーをセットアップ
    server::setup_websocket_server(&mut app);
    
    app.run();
}

#[derive(Resource, Default)]
pub struct Fonts {
    text_font: TextFont
}
    
#[derive(Component)]
struct TextScroll;

#[derive(Resource)]
pub struct TextQueue {
    texts: Vec<TextSource>,
    current_index: usize,
    current_preset: String,
}

#[derive(Resource, Default)]
pub struct ScrollingState {
    is_active: bool,
}

#[derive(Resource, Default)]
pub struct ScrollingSpeed {
    speed: f32,
}

#[derive(Component)]
pub struct Showing;

#[derive(Component)]
pub struct LoopingText {
    pub original_x: f32,
    pub text_width: f32,
    pub loop_speed: f32,
}

fn setup(
    mut cmds: Commands,
    config: Res<Config>,
    asset_server: Res<AssetServer>,
    mut fonts: ResMut<Fonts>,
    mut bingo: ResMut<BingoState>
) {
    let font = asset_server.load("fonts/ipag.ttf");
    let text_font = TextFont {
        font: font,
      font_size: config.text_size,
        ..default()
    };
    let bingo_state = BingoState::new();
    fonts.text_font = text_font;
    bingo.numbers = bingo_state.numbers;
    bingo.index = bingo_state.index;
    cmds.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(config.camera_offset, 0.0, 0.0)),
    ));
}

fn text_scroll(
    time: Res<Time>,
    scrolling_state: Res<ScrollingState>,
    scrolling_speed: Res<ScrollingSpeed>,
    mut query: Query<&mut Transform, (With<TextScroll>, Without<LoopingText>)>,
) {
    if !scrolling_state.is_active {
        return;
    }

    for mut transform in &mut query {
        transform.translation.x -= scrolling_speed.speed * time.delta_secs()
    }
}

fn text_loop(
    time: Res<Time>,
    config: Res<Config>,
    mut query: Query<(&mut Transform, &LoopingText), With<Showing>>,
) {
    for (mut transform, looping_text) in &mut query {
        // 左に移動
        transform.translation.x -= looping_text.loop_speed * time.delta_secs();
        
        // 通常のスクロールと同じ判定ロジックを使用
        let text_left_edge = transform.translation.x + (looping_text.text_width + config.window_width) / 2.0 + 5.0;
        
        // テキストが完全に画面左端を通り過ぎたかチェック（テキスト全体が画面外に出るまで待つ）
        if text_left_edge < 0.0 {
            // 右端から再開
            transform.translation.x = looping_text.original_x;
        }
    }
}
     
fn handle_keyboard_action(
    keys: Res<ButtonInput<KeyCode>>,
    mut scrolling_state: ResMut<ScrollingState>,
    mut text_queue: ResMut<TextQueue>,
    mut cmds: Commands,
    mut bingo: ResMut<BingoState>,
    config: Res<Config>,
    fonts: Res<Fonts>,
    text_query: Query<Entity, With<Showing>>,
    mut scrolling_speed: ResMut<ScrollingSpeed>,
) {
    if keys.just_pressed(KeyCode::Enter) {
            for entity in text_query.iter() {
                cmds.entity(entity).despawn();
	    }

	if text_queue.texts[text_queue.current_index].duration == 0.0 {
	    text_spawner::spawn_static_text(&mut cmds, &text_queue.texts[text_queue.current_index].content.clone(), fonts.text_font.clone());
	} else {
	    text_spawner::spawn_text(
		&mut cmds,
		&text_queue.texts[text_queue.current_index].content.clone(),
		&text_queue.texts[text_queue.current_index].duration,
		fonts.text_font.clone(),
		&config,
		&mut *scrolling_speed,
        );
	}

        text_queue.current_index += 1;

	scrolling_state.is_active = true;
    };
    if keys.just_pressed(KeyCode::KeyB) {
	for entity in text_query.iter() {
	    cmds.entity(entity).despawn();
	}
	text_spawner::spawn_static_text(&mut cmds, &bingo.next().unwrap_or(0).to_string(), fonts.text_font.clone());
    }	
}

fn check_text_completion(
    mut cmds: Commands,
    config: Res<Config>,
    text_queue: ResMut<TextQueue>,
    mut scrolling_state: ResMut<ScrollingState>,
    query: Query<(Entity, &Transform, &TextLayoutInfo), (With<TextScroll>, Without<LoopingText>)>,
) {

    for (entity, transform, info) in query.iter() {
        let text_width = info.size.x;
        let text_left_edge =
            transform.translation.x + (text_width + config.window_width) / 2.0 + 5.0;

        // テキストが完全に画面左端を通り過ぎたかチェック（テキスト全体が画面外に出るまで待つ）
        if text_left_edge < 0.0 {
            // 現在のテキストエンティティを削除
            cmds.entity(entity).despawn();

            // スクロールを停止
            scrolling_state.is_active = false;
	    
            println!(
                "Next: {} ",
                text_queue.texts[text_queue.current_index].content
            );
            break; // 一度に一つのテキストのみ処理
        }
    }
}
