use bevy::prelude::*;

use crate::{AppState, APP_STATE_STAGE};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>()
            .on_state_enter(APP_STATE_STAGE, AppState::Menu, enter.system())
            .on_state_update(APP_STATE_STAGE, AppState::Menu, update.system())
            .on_state_exit(APP_STATE_STAGE, AppState::Menu, exit.system());
    }
}

struct MenuUi;
struct StartGameButton;

struct ButtonMaterials {
    play: Handle<ColorMaterial>,
    play_hover: Handle<ColorMaterial>,
}

impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        let asset_server = resources.get_mut::<AssetServer>().unwrap();

        ButtonMaterials {
            play: materials.add(asset_server.load("ui/button.png").into()),
            play_hover: materials.add(asset_server.load("ui/button_outline.png").into()),
        }
    }
}

fn enter(
    commands: &mut Commands,
    button_colors: Res<ButtonMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with(MenuUi)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(400.0), Val::Px(200.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: button_colors.play.clone(),
                    ..Default::default()
                })
                .with(StartGameButton);
        });
}

fn update(
    button_colors: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>),
        (Mutated<Interaction>, With<StartGameButton>),
    >,
    mut states: ResMut<State<AppState>>,
) {
    for (interaction, mut material) in interaction_query.iter_mut() {
        // let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => states.set_next(AppState::InGame).unwrap(),
            Interaction::Hovered => {
                *material = button_colors.play_hover.clone();
            }
            Interaction::None => {
                *material = button_colors.play.clone();
            }
        }
    }
}

fn exit(commands: &mut Commands, query: Query<Entity, With<MenuUi>>) {
    for entity in query.iter() {
        commands.despawn_recursive(entity);
    }
}
