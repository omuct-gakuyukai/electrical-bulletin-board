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
    pub just_finished: bool,
    pub current_number_start_time: f32, // 現在の数字が表示開始された時間
    pub total_elapsed_time: f32, // カウントダウン開始からの総経過時間
    pub mode: CountdownMode, // カウントダウンモード
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CountdownMode {
    #[default]
    Normal,      // 通常の等間隔
    Accelerated, // 線形加速（最初遅く、後半速く）
    Decelerated, // 線形減速（最初速く、後半遅く）
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
    pub fn new(seconds: f32, mode: CountdownMode) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            initial_seconds: seconds,
            is_active: false,
            last_displayed_number: -1,
            just_finished: false,
            current_number_start_time: 0.0,
            total_elapsed_time: 0.0,
            mode,
        }
    }
    
    pub fn start(&mut self) {
        self.timer.reset();
        self.is_active = true;
        self.last_displayed_number = -1;
        self.just_finished = false;
        self.current_number_start_time = 0.0;
        self.total_elapsed_time = 0.0;
    }
    
    pub fn stop(&mut self) {
        self.is_active = false;
        self.last_displayed_number = -1;
        self.just_finished = false;
        self.current_number_start_time = 0.0;
        self.total_elapsed_time = 0.0;
    }
    
    pub fn remaining_seconds(&self) -> f32 {
        if self.is_active {
            self.timer.remaining_secs()
        } else {
            self.initial_seconds
        }
    }
    
    // 加速度的カウントダウンでの現在の表示数字を計算
    pub fn get_accelerated_number(&self) -> i32 {
        if self.mode == CountdownMode::Normal || self.total_elapsed_time <= 0.0 {
            return if self.remaining_seconds() > 0.0 {
                self.remaining_seconds().ceil() as i32
            } else {
                0
            };
        }
        
        // 20秒間で10→0のカウントダウン
        let target_times = match self.mode {
            CountdownMode::Accelerated => Self::calculate_accelerated_times(),
            CountdownMode::Decelerated => Self::calculate_decelerated_times(),
            CountdownMode::Normal => return self.remaining_seconds().ceil() as i32,
        };
        
        let mut accumulated_time = 0.0;
        for (number, duration) in target_times.iter().enumerate() {
            accumulated_time += duration;
            if self.total_elapsed_time < accumulated_time {
                return 10 - number as i32;
            }
        }
        
        0 // 最後
    }
    
    // 線形加速（最初遅く、後半速く）
    fn calculate_accelerated_times() -> Vec<f32> {
        let mut times = Vec::new();
        
        // 初期値1.8秒、最終値1.3秒の線形加速
        let initial_time = 1.8; // 数字10の表示時間
        let final_time = 1.3;   // 数字1の表示時間
        
        for i in 0..10 {
            // 線形補間：i=0(数字10)で1.8秒、i=9(数字1)で1.3秒
            let progress = i as f32 / 9.0; // 0.0 ~ 1.0
            let time = initial_time + (final_time - initial_time) * progress;
            times.push(time);
        }
        
        times
    }
    
    // 線形減速（最初速く、後半遅く）
    fn calculate_decelerated_times() -> Vec<f32> {
        let mut times = Vec::new();
        
        // 初期値1.3秒、最終値1.8秒の線形減速
        let initial_time = 1.3; // 数字10の表示時間
        let final_time = 1.8;   // 数字1の表示時間
        
        for i in 0..10 {
            // 線形補間：i=0(数字10)で1.3秒、i=9(数字1)で1.8秒
            let progress = i as f32 / 9.0; // 0.0 ~ 1.0
            let time = initial_time + (final_time - initial_time) * progress;
            times.push(time);
        }
        
        times
    }
}

pub fn setup_countdown_timer(mut commands: Commands) {
    commands.insert_resource(CountdownTimer::new(10.0, CountdownMode::Normal));
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
    countdown_timer.total_elapsed_time += time.delta_secs();
    
    let current_number = if countdown_timer.mode == CountdownMode::Normal {
        let remaining = countdown_timer.remaining_seconds();
        if remaining > 0.0 {
            remaining.ceil() as i32
        } else {
            0
        }
    } else {
        countdown_timer.get_accelerated_number()
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
        countdown_timer.current_number_start_time = countdown_timer.total_elapsed_time;
        
        println!("Countdown: {} (elapsed: {:.2}s)", current_number, countdown_timer.total_elapsed_time);
    }
    
    // タイマー終了チェック（変動モードでは固定時間×10、通常モードは設定時間）
    let should_finish = if countdown_timer.mode != CountdownMode::Normal {
        countdown_timer.total_elapsed_time >= 15.5 // 1.3～1.8秒×10 ≈ 15.5秒
    } else {
        countdown_timer.timer.is_finished()
    };
    
    if should_finish && countdown_timer.is_active {
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
