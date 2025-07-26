use super::*;

#[cfg(feature = "third_person")]
mod third_person;
#[cfg(feature = "top_down")]
mod top_down;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(Screen::Title), add_skybox_to_camera);

    #[cfg(feature = "third_person")]
    app.add_plugins(third_person::plugin);

    #[cfg(feature = "top_down")]
    app.add_plugins(top_down::plugin);
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        SceneCamera,
        IsDefaultUiCamera,
        Camera3d::default(),
        Camera {
            hdr: true,
            ..Default::default()
        },
        Msaa::Sample4,
        Transform::from_xyz(100., 50., 100.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
