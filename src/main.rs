use bevy::{
    camera::visibility::NoFrustumCulling,
    color::palettes::{css::BLACK, tailwind::YELLOW_300},
    prelude::*,
    text::{TextBounds, TextLayout, TextLayoutInfo},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, adjust_text_initial_offset)
        .add_systems(Update, text_scroll)
        .run();
}

#[derive(Component)]
struct TextScroll;

#[derive(Component)]
struct InitialOffsetSet;

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/ipaexg.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 1080.0,
        ..default()
    };
    cmds.spawn(Camera2d);
    cmds.spawn((
        Text2d::new("学友会執行委員会 情報通信課"),
        text_font.clone(),
        TextColor(Color::Srgba(YELLOW_300)),
        TextBackgroundColor(BLACK.into()),
        Transform::from_translation(Vec3::ZERO),
        TextLayout::default(),
        TextScroll,
    ))
    .insert(NoFrustumCulling);
}

fn text_scroll(time: Res<Time>, mut query: Query<&mut Transform, With<TextScroll>>) {
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

        let initial_offset_x: f32 = screen_width + text_width + 10.0;

        transform.translation.x = initial_offset_x;
        cmds.entity(entity).insert(InitialOffsetSet);
    }
}
