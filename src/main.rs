use std::time::Duration;

use bevy::{prelude::*, window::PresentMode};
use bevy::input::common_conditions::input_just_pressed;

use bevy_render::{
    batching::gpu_preprocessing::{GpuPreprocessingMode, GpuPreprocessingSupport},
    RenderApp,
};

const CAT_SPEED: f32 = 250.0;

fn main() {
     let mut app = App::new();
    app
    .add_plugins(
        DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                position: WindowPosition::Centered(MonitorSelection::Primary), 
                resolution: Vec2::new(1024., 1024.).into(), 
                title: "UIA Cat".into(),
                present_mode: PresentMode::AutoVsync, 
                ..Default::default()
            }),
            ..Default::default()
        }).set(ImagePlugin::default_nearest())
    )
    .add_systems(Startup, setup)
    .add_systems(Update, move_cat)
    .add_systems(Update, execute_animations)
    .add_systems(
        Update,
        trigger_animation::<Cat>.run_if(input_just_pressed(KeyCode::Space)),
    );

    app
    .sub_app_mut(RenderApp)
    .insert_resource(GpuPreprocessingSupport {
        max_supported_mode: GpuPreprocessingMode::None,
    });

    app.run();

}

#[derive(Component)]
struct Cat;

#[derive(Component)]
struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
    is_playing: bool,
}

impl AnimationConfig {
    fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
            is_playing: false,
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

fn trigger_animation<S: Component>(mut animation: Single<&mut AnimationConfig, With<S>>) {
    // We create a new timer when the animation is triggered
    animation.frame_timer = AnimationConfig::timer_from_fps(animation.fps);
    animation.is_playing = true;
}

fn execute_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite), With<Cat>>) {
    for (mut config, mut sprite) in &mut query {
        // We track how long the current sprite has been displayed for
        if !config.is_playing {
            continue;
        }
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index == config.last_sprite_index {
                    // ...and it IS the last frame, then we move back to the first frame and stop.
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    atlas.index = config.first_sprite_index;
                    config.is_playing = false;
                } else {
                    // ...and it is NOT the last frame, then we move to the next frame...
                    atlas.index += 1;
                    // ...and reset the frame timer to start counting all over again
                    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    assert_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>
) {

    let texture: Handle<Image> = assert_server.load("oia-uia-sprite-table.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(320), 10, 6, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_config = AnimationConfig::new(0, 59, 60);
    commands.spawn(Camera2d::default());
    commands.spawn(
        (
            Sprite {
                image: texture, 
                texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config.first_sprite_index,
                }),
                ..Default::default()
            },
            Cat {},
            Transform::IDENTITY.with_scale(Vec3::splat(0.5)),
            animation_config,
        )
    );
    commands.insert_resource(ClearColor(Color::srgb(0.5, 0.7, 0.5)));
}

fn move_cat(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cat_transform: Single<(&mut Transform, &mut Sprite), With<Cat>>,
    time: Res<Time>,
) {
    let mut direction_y = 0.0;
    let mut direction_x = 0.0;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction_y += 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyS) {
        direction_y -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyA) {
        direction_x -= 1.0;
        cat_transform.1.flip_x = true;
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        direction_x += 1.0;
        cat_transform.1.flip_x = false;
    }

    // Calculate the new horizontal paddle position based on player input
    cat_transform.0.translation.x += direction_x * CAT_SPEED * time.delta_secs();

    cat_transform.0.translation.y += direction_y * CAT_SPEED * time.delta_secs();
}