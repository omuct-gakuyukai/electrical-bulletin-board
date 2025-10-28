use bevy::{
    camera::visibility::NoFrustumCulling,
    color::palettes::{css::BLACK, tailwind::YELLOW_300},
    prelude::*,
    text::{TextBounds, TextLayout, TextLayoutInfo},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(TextQueue {
            texts: vec![
                "大阪公立大学工業高等専門学校".to_string(),
                "学友会執行委員会 情報通信課".to_string(),
                "学友会執行委員会 総務課展示電気係".to_string(),
                "学友会執行委員会 音響課".to_string(),
            ],
            current_index: 0,
        })
        .init_resource::<ScrollingState>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_mouse_click)
        .add_systems(Update, adjust_text_initial_offset)
        .add_systems(Update, text_scroll)
        .add_systems(Update, check_text_completion)
        .run();
}

#[derive(Component)]
struct TextScroll;

#[derive(Component)]
struct InitialOffsetSet;

#[derive(Component)]
struct ScrollingActive;

#[derive(Resource)]
struct TextQueue {
    texts: Vec<String>,
    current_index: usize,
}

#[derive(Resource, Default)]
struct ScrollingState {
    is_active: bool,
}

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>, text_queue: Res<TextQueue>) {
    let font = asset_server.load("fonts/ipaexg.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 1080.0,
        ..default()
    };
    cmds.spawn(Camera2d);
    
    // 最初のテキストを表示（スクロールは無効状態で開始）
    spawn_text(&mut cmds, &text_queue.texts[0], text_font);
    
    // クリック指示テキストを表示
    let instruction_font = TextFont {
        font: font.clone(),
        font_size: 64.0,
        ..default()
    };
    cmds.spawn((
        Text2d::new(""),
        instruction_font,
        TextColor(Color::Srgba(YELLOW_300)),
        Transform::from_translation(Vec3::new(0.0, -400.0, 1.0)),
        TextLayout::default(),
    ));
}

fn spawn_text(cmds: &mut Commands, text: &str, text_font: TextFont) {
    let screen_width: f32 = 1920.0;
    
    cmds.spawn((
        Text2d::new(text),
        text_font,
        TextColor(Color::Srgba(YELLOW_300)),
        TextBackgroundColor(BLACK.into()),
        Transform::from_translation(Vec3::new(0.0, 2000.0, 0.0)),
        TextLayout::default(),
        TextScroll,
    ))
    .insert(NoFrustumCulling);
}

fn text_scroll(
    time: Res<Time>, 
    scrolling_state: Res<ScrollingState>,
    mut query: Query<&mut Transform, (With<TextScroll>, With<ScrollingActive>)>
) {
    if !scrolling_state.is_active {
        return;
    }
    
    for mut transform in &mut query {
        transform.translation.x -= 5000.0 * time.delta_secs()
    }
}

fn adjust_text_initial_offset(
    mut cmds: Commands,
    mut query: Query<
        (Entity, &mut Transform, &TextLayoutInfo),
        (With<TextScroll>, Without<InitialOffsetSet>),
    >,
) {
    let screen_width: f32 = 1920.0;

    for (entity, mut transform, info) in query.iter_mut() {
        let text_width = info.size.x;

        // テキストサイズが計算されるまで待つ
        if text_width <= 0.0 {
            continue;
        }

        let act_size = text_width * transform.scale.truncate();
        println!("text_width: {} / {}", text_width, act_size);

        // テキストの右端が画面右端より少し外に出るように配置
        let initial_offset_x: f32 = screen_width + (text_width / 2.0) + 10.0;

        transform.translation.x = initial_offset_x;
        transform.translation.y = 0.0;
        cmds.entity(entity).insert(InitialOffsetSet);
    }
}

fn handle_mouse_click(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut scrolling_state: ResMut<ScrollingState>,
    mut text_queue: ResMut<TextQueue>,
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    waiting_text_query: Query<Entity, (With<TextScroll>, With<InitialOffsetSet>, Without<ScrollingActive>)>,
    active_text_query: Query<Entity, (With<TextScroll>, With<ScrollingActive>)>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if !scrolling_state.is_active {
            // 待機中のテキストがある場合、スクロール開始
            scrolling_state.is_active = true;
            
            for entity in waiting_text_query.iter() {
                cmds.entity(entity).insert(ScrollingActive);
            }
            
            println!("スクロール開始！");
        } else {
            // スクロール中の場合、現在のテキストを削除して次のテキストを表示
            for entity in active_text_query.iter() {
                cmds.entity(entity).despawn();
            }
            
            // 次のテキストインデックスに進む
            text_queue.current_index = (text_queue.current_index + 1) % text_queue.texts.len();
            
            // スクロールを停止
            scrolling_state.is_active = false;
            
            // 次のテキストを表示
            let font = asset_server.load("fonts/ipaexg.ttf");
            let text_font = TextFont {
                font: font.clone(),
                font_size: 1080.0,
                ..default()
            };
            spawn_text(&mut cmds, &text_queue.texts[text_queue.current_index].clone(), text_font);
            
            println!("次のテキストにスキップ: {}", text_queue.texts[text_queue.current_index]);
        }
    }
}

fn check_text_completion(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    mut text_queue: ResMut<TextQueue>,
    mut scrolling_state: ResMut<ScrollingState>,
    query: Query<(Entity, &Transform, &TextLayoutInfo), (With<TextScroll>, With<InitialOffsetSet>, With<ScrollingActive>)>,
) {
    let font = asset_server.load("fonts/ipaexg.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 1080.0,
        ..default()
    };

    for (entity, transform, info) in query.iter() {
        let text_width = info.size.x;
        let text_left_edge = transform.translation.x + text_width / 2.0 + 1300.0 ;
        
        // テキストが完全に画面左端を通り過ぎたかチェック（テキスト全体が画面外に出るまで待つ）
        if text_left_edge < 0.0 {
            // 現在のテキストエンティティを削除
            cmds.entity(entity).despawn();
            
            // 次のテキストインデックスに進む
            text_queue.current_index = (text_queue.current_index + 1) % text_queue.texts.len();
            
            // スクロールを停止
            scrolling_state.is_active = false;
            
            // 次のテキストを表示
            spawn_text(&mut cmds, &text_queue.texts[text_queue.current_index].clone(), text_font);
            
            println!("Next: {} ", text_queue.texts[text_queue.current_index]);
            break; // 一度に一つのテキストのみ処理
        }
    }
}
