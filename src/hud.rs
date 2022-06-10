use super::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Healthbar;

#[derive(Component)]
pub struct Score;

pub struct UiMaterials {
    font: Handle<Font>,
    health_bar: Handle<Image>,
    score_panel: Handle<Image>,
}

impl FromWorld for UiMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        UiMaterials {
            font: asset_server.load("ui/AGENCYB.ttf"),
            health_bar: asset_server.load("ui/health_bar.png").into(),
            score_panel: asset_server.load("ui/score_panel.png").into(),
        }
    }
}

pub fn init_hud(mut cmd: Commands, materials: Res<UiMaterials>) {
    cmd.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::ColumnReverse,
            ..default()
        },
        color: Color::NONE.into(),
        ..default()
    })
    .with_children(|parent| {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(202.0), Val::Px(36.0)),
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                image: materials.score_panel.clone().into(),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "0",
                            TextStyle {
                                font: materials.font.clone().into(),
                                font_size: 28.0,
                                color: Color::rgb_u8(0, 0, 0),
                                ..default()
                            },
                            TextAlignment::default(),
                        ),
                        ..default()
                    })
                    .insert(Score)
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(TextBundle {
                                text: Text::with_section(
                                    "0",
                                    TextStyle {
                                        font: materials.font.clone().into(),
                                        font_size: 28.0,
                                        color: Color::rgb_u8(147, 14, 58),
                                        ..default()
                                    },
                                    TextAlignment::default(),
                                ),

                                style: Style {
                                    position_type: PositionType::Relative,
                                    position: Rect {
                                        top: Val::Px(-1.),
                                        left: Val::Px(-1.),
                                        ..default()
                                    },
                                    ..default()
                                },
                                ..default()
                            })
                            .insert(Score);
                    });
            });

        // parent
        //     .spawn_bundle(TextBundle {
        //         text: Text::with_section("Score:", default(), default()),
        //         style: Style {
        //             align_self: AlignSelf::FlexStart,
        //             position_type: PositionType::Absolute,
        //             position: Rect {
        //                 top: Val::Px(16.0),
        //                 left: Val::Px(16.0),
        //                 ..default()
        //             },
        //             ..default()
        //         },
        //         ..default()
        //     })
        //     .insert(Score)
        //     .insert(CleanupAfterGame);

        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(800.0), Val::Px(32.0)),
                    margin: Rect::all(Val::Px(16.0)),
                    ..default()
                },

                color: Color::NONE.into(),
                ..default()
            })
            .insert(CleanupAfterGame)
            .with_children(|parent| {
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: Rect {
                                top: Val::Px(4.0),
                                left: Val::Px(4.0),
                                ..default()
                            },
                            size: Size::new(Val::Px(792.0), Val::Px(24.0)),
                            ..default()
                        },
                        color: Color::rgb_u8(147, 14, 58).into(),
                        ..default()
                    })
                    .insert(CleanupAfterGame)
                    .insert(Healthbar);

                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            ..default()
                        },
                        image: materials.health_bar.clone().into(),
                        ..default()
                    })
                    .insert(CleanupAfterGame);
            });
    });
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
