use crate::{loader::Config, ScrollingSpeed, Showing, TextScroll, LoopingText};
use bevy::{
    camera::visibility::NoFrustumCulling,
    color::palettes::tailwind::{SLATE_900, YELLOW_300},
    prelude::*,
};

pub fn spawn_text(
    cmds: &mut Commands,
    text: &str,
    duration: &f32,
    text_font: TextFont,
    config: &Config,
    scrolling_speed: &mut ScrollingSpeed,
) {
    let text_offset = crate::text::calc_text_offset(text, config.text_size, config.window_width);
    println!("Offset: {}, Duration: {}", text_offset, duration);
    
    // duration が 1000 以上の場合はループ再生
    if *duration >= 1000.0 {
        let text_width = text_offset * 2.0; // 概算のテキスト幅
        let loop_speed = 500.0; // 500px/s
        
        cmds.spawn((
            Text2d::new(text),
            text_font,
            TextColor(Color::Srgba(YELLOW_300)),
            TextBackgroundColor(Color::Srgba(SLATE_900)),
            Transform::from_translation(Vec3::new(config.window_width / 2.0 + text_width / 2.0 + 50.0, 0.0, 0.0)),
            TextLayout::default(),
            LoopingText {
                original_x: text_offset,
                text_width,
                loop_speed,
            },
            Showing,
        ))
        .insert(NoFrustumCulling);
    } else {
        // 通常の1回だけのスクロール
        cmds.spawn((
            Text2d::new(text),
            text_font,
            TextColor(Color::Srgba(YELLOW_300)),
            TextBackgroundColor(Color::Srgba(SLATE_900)),
            Transform::from_translation(Vec3::new(text_offset, 0.0, 0.0)),
            TextLayout::default(),
            TextScroll,
            Showing,
        ))
        .insert(NoFrustumCulling);
        scrolling_speed.speed =
            crate::text::calc_speed(text_offset * 2.0, duration, config.window_width);
    }
}

pub fn spawn_static_text(
    cmds: &mut Commands,
    text: &str,
    text_font: TextFont,
) {
    cmds.spawn((
        Text2d::new(text),
        text_font,
        TextColor(Color::Srgba(YELLOW_300)),
        TextBackgroundColor(Color::Srgba(SLATE_900)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        TextLayout::default(),
        Showing,
    ))
    .insert(NoFrustumCulling);
}
