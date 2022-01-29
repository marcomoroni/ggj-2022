use bevy::prelude::*;

const BACKGROUND_COLOR: Color = Color::rgb(0.05, 0.05, 0.05);

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .run();
}
