use bevy::{
    time::{Timer, TimerMode},
    prelude::*,
};

#[derive(Resource, Default)]
pub struct CountdownTimer {
    pub timer: Timer,
    pub initial_seconds: f32,
    pub is_active: bool,
    pub last_displayed_number: i32,
    pub just_finished: bool, // カウントダウン終了を通知するフラグ
}

#[derive(Component)]
pub struct CountdownText;

#[derive(Component)]
pub struct FadeComponent {
    pub fade_in_duration: f32,
    pub fade_out_duration: f32,
    pub display_duration: f32,
    pub current_time: f32,
    pub phase: FadePhase,
}

#[derive(Debug, PartialEq)]
pub enum FadePhase {
    FadeIn,
    Display,
    FadeOut,
    Complete,
}

impl CountdownTimer {
    pub fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            initial_seconds: seconds,
            is_active: false,
            last_displayed_number: -1,
            just_finished: false,
        }
    }
    
    pub fn start(&mut self) {
        self.timer.reset();
        self.is_active = true;
        self.last_displayed_number = -1; // リセット
        self.just_finished = false;
    }
    
    pub fn stop(&mut self) {
        self.is_active = false;
        self.last_displayed_number = -1;
        self.just_finished = false;
    }
    
    pub fn remaining_seconds(&self) -> f32 {
        if self.is_active {
            self.timer.remaining_secs()
        } else {
            self.initial_seconds
        }
    }
}

pub fn setup_countdown_timer(mut commands: Commands) {
    commands.insert_resource(CountdownTimer::new(10.0));
}

pub fn countdown_system(
    time: Res<Time>,
    mut countdown_timer: ResMut<CountdownTimer>,
    mut commands: Commands,
    fonts: Res<crate::Fonts>,
    text_query: Query<Entity, With<CountdownText>>,
) {
    if !countdown_timer.is_active {
        return;
    }

    countdown_timer.timer.tick(time.delta());
    
    let remaining = countdown_timer.remaining_seconds();
    let current_number = if remaining > 0.0 {
        remaining.ceil() as i32
    } else {
        0
    };
    
    // 数字が変わった場合のみ更新
    if current_number != countdown_timer.last_displayed_number {
        // 既存のカウントダウンテキストを削除
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        
        let display_text = current_number.to_string();
        
        // 新しいカウントダウンテキストを表示（フェードイン/アウト付き）
        spawn_countdown_text(&mut commands, &display_text, fonts.text_font.clone());
        
        countdown_timer.last_displayed_number = current_number;
    }
    
    // タイマー終了チェック
    if countdown_timer.timer.is_finished() && countdown_timer.is_active {
        countdown_timer.stop();
        countdown_timer.just_finished = true;
        println!("Countdown finished! Ready for exit guidance.");
    }
}

pub fn fade_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut FadeComponent, &mut TextColor), With<CountdownText>>,
    mut commands: Commands,
) {
    let mut entities_to_remove = Vec::new();
    
    for (entity, mut fade, mut text_color) in query.iter_mut() {
        fade.current_time += time.delta_secs();
        
        let alpha = match fade.phase {
            FadePhase::FadeIn => {
                if fade.current_time >= fade.fade_in_duration {
                    fade.phase = FadePhase::Display;
                    fade.current_time = 0.0;
                    1.0
                } else {
                    fade.current_time / fade.fade_in_duration
                }
            }
            FadePhase::Display => {
                if fade.current_time >= fade.display_duration {
                    fade.phase = FadePhase::FadeOut;
                    fade.current_time = 0.0;
                }
                1.0
            }
            FadePhase::FadeOut => {
                if fade.current_time >= fade.fade_out_duration {
                    fade.phase = FadePhase::Complete;
                    0.0
                } else {
                    1.0 - (fade.current_time / fade.fade_out_duration)
                }
            }
            FadePhase::Complete => 0.0,
        };
        
        // アルファ値を適用
        let mut color = text_color.0;
        color.set_alpha(alpha.clamp(0.0, 1.0));
        text_color.0 = color;
        
        // フェードアウト完了時にエンティティを削除リストに追加
        if fade.phase == FadePhase::Complete {
            entities_to_remove.push(entity);
        }
    }
    
    // 遅延削除でエンティティを安全に削除
    for entity in entities_to_remove {
        match commands.get_entity(entity) {
            Ok(mut entity_commands) => {
                entity_commands.despawn();
            }
            Err(_) => {
                // エンティティが既に削除されている場合は無視
            }
        }
    }
}

fn spawn_countdown_text(
    commands: &mut Commands,
    text: &str,
    text_font: TextFont,
) {
    commands.spawn((
        Text2d::new(text),
        text_font,
        TextColor(Color::srgba(1.0, 1.0, 0.3, 0.0)), // 初期は透明
        Transform::from_xyz(0.0, 0.0, 0.0),
        TextLayout::default(),
        CountdownText,
        FadeComponent {
            fade_in_duration: 0.3,
            fade_out_duration: 0.3,
            display_duration: 0.4,
            current_time: 0.0,
            phase: FadePhase::FadeIn,
        },
        crate::Showing,
    ));
}

pub fn countdown_finished_system(
    mut countdown_timer: ResMut<CountdownTimer>,
    ws_channel: Option<ResMut<crate::server::WebSocketChannel>>,
) {
    if countdown_timer.just_finished {
        countdown_timer.just_finished = false;
        
        // WebSocketでカウントダウン終了を通知
        if let Some(ws_channel) = ws_channel {
            let response = crate::server::WsResponse::Countdown(crate::server::CountdownResponse {
                status: "finished".to_string(),
            });
            let _ = ws_channel.response_sender.send(response);
        }
    }
}
