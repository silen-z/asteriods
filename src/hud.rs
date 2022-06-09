use super::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Healthbar;

#[derive(Component)]
pub struct Score;

pub struct UiMaterials {
    font: Handle<Font>,
    transparent: Handle<ColorMaterial>,
    health_bar: Handle<ColorMaterial>,
    health_color: Handle<ColorMaterial>,
    score_panel: Handle<Image>,
}

impl FromWorld for UiMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();

        UiMaterials {
            font: asset_server.load("ui/AGENCYB.ttf"),
            transparent: materials.add(Color::NONE.into()),
            health_bar: materials.add(asset_server.load("ui/health_bar.png").into()),
            health_color: materials.add(Color::rgb_u8(147, 14, 58).into()),
            score_panel: asset_server.load("ui/score_panel.png").into(),
        }
    }
}

pub fn init_hud(mut cmd: Commands, materials: Res<UiMaterials>) {
    cmd.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        color: Color::NONE.into(),
        ..Default::default()
    })
    .with_children(|parent| {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(202.0), Val::Px(36.0)),
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                image: materials.score_panel.clone().into(),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "156 152 632",
                            TextStyle {
                                font_size: 28.0,
                                color: Color::rgb_u8(0, 0, 0),
                                ..Default::default()
                            },
                            TextAlignment::default(),
                        ),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle {
                            text: Text::with_section(
                                "156 152 632",
                                TextStyle {
                                    font_size: 28.0,
                                    color: Color::rgb_u8(147, 14, 58),
                                    ..Default::default()
                                },
                                TextAlignment::default(),
                            ),
                            style: Style {
                                position_type: PositionType::Relative,
                                position: Rect {
                                    top: Val::Px(-1.),
                                    left: Val::Px(-1.),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                    });
            });
    });

    // commands
    //     .spawn(TextBundle {
    //         style: Style {
    //             align_self: AlignSelf::FlexStart,
    //             position_type: PositionType::Absolute,
    //             position: Rect {
    //                 top: Val::Px(16.0),
    //                 left: Val::Px(16.0),
    //                 ..Default::default()
    //             },
    //             ..Default::default()
    //         },
    //         text: Text {
    //             value: "Score:".to_string(),
    //             font: materials.font.clone(),
    //             style: TextStyle {
    //                 font_size: 60.0,
    //                 color: Color::rgb_u8(147, 14, 58),
    //                 ..Default::default()
    //             },
    //         },
    //         ..Default::default()
    //     })
    //     .with(Score)
    //     .with(CleanupAfterGame)
    //     .spawn(NodeBundle {
    //         style: Style {
    //             size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    //             align_items: AlignItems::FlexEnd,
    //             justify_content: JustifyContent::Center,
    //             ..Default::default()
    //         },
    //         material: materials.transparent.clone(),
    //         ..Default::default()
    //     })
    //     .with(CleanupAfterGame)
    //     .with_children(|parent| {
    //         parent
    //             .spawn(NodeBundle {
    //                 style: Style {
    //                     size: Size::new(Val::Px(800.0), Val::Px(32.0)),
    //                     margin: Rect::all(Val::Px(16.0)),
    //                     ..Default::default()
    //                 },
    //                 material: materials.transparent.clone(),
    //                 ..Default::default()
    //             })
    //             .with(CleanupAfterGame)
    //             .with_children(|parent| {
    //                 parent
    //                     .spawn(NodeBundle {
    //                         style: Style {
    //                             position_type: PositionType::Absolute,
    //                             position: Rect {
    //                                 top: Val::Px(4.0),
    //                                 left: Val::Px(4.0),
    //                                 ..Default::default()
    //                             },
    //                             size: Size::new(Val::Px(792.0), Val::Px(24.0)),
    //                             ..Default::default()
    //                         },
    //                         material: materials.health_color.clone(),
    //                         ..Default::default()
    //                     })
    //                     .with(CleanupAfterGame)
    //                     .with(Healthbar)
    //                     .spawn(NodeBundle {
    //                         style: Style {
    //                             size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    //                             ..Default::default()
    //                         },
    //                         material: materials.health_bar.clone(),
    //                         ..Default::default()
    //                     })
    //                     .with(CleanupAfterGame);
    //             });
    //     });
}

pub fn hud_healthbar(
    player_ship: Res<PlayerSpaceship>,
    ships: Query<(&Spaceship, &Hitpoints)>,
    mut healthbar: Query<&mut Style, With<Healthbar>>,
    mut score: Query<&mut Text, With<Score>>,
) {
    if let Ok((ship, hp)) = ships.get(player_ship.0) {
        for mut style in healthbar.iter_mut() {
            style.max_size.width = Val::Percent(hp.0 as f32);
        }

        for mut text in score.iter_mut() {
            text.sections[0].value = ship.score.to_string();
        }
    }
}
