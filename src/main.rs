use bevy::{
    prelude::*,
    color::palettes::css::{ YELLOW },
};

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.spawn(Camera2d::default());
    cmds.spawn((Text::new("学友会"),
		TextFont {
		    font: asset_server.load("fonts/ipaexg.ttf"),
		    font_size: 1200.0,
		    ..Default::default()
		},
		TextColor(YELLOW.into())
    ));
}

fn main() {
    App::new()
	.add_plugins(DefaultPlugins)
	.add_systems(Startup, setup)
	.run();
}
