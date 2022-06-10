use bevy::{prelude::*, ui::FocusPolicy};

use crate::AppState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonMaterials>()
            .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(enter))
            .add_system_set(SystemSet::on_update(AppState::Menu).with_system(update))
            .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(exit));
    }
}

#[derive(Component)]
struct MenuUi;

#[derive(Component)]
struct StartGameButton;

struct ButtonMaterials {
    play: Handle<Image>,
    play_hover: Handle<Image>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        ButtonMaterials {
            play: asset_server.load("ui/button.png"),
            play_hover: asset_server.load("ui/button_outline.png"),
        }
    }
}

fn enter(mut commands: Commands, button_colors: Res<ButtonMaterials>) {
    commands
        .spawn_bundle(NodeBundle {
            focus_policy: FocusPolicy::Pass,
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(MenuUi)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(400.0), Val::Px(200.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    image: button_colors.play.clone().into(),
                    ..default()
                })
                .insert(StartGameButton);
        });
}

fn update(
    button_colors: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut UiImage),
        (Changed<Interaction>, With<StartGameButton>),
    >,
    mut states: ResMut<State<AppState>>,
) {
    for (interaction, mut image) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                states.replace(AppState::InGame).unwrap();
            }
            Interaction::Hovered => {
                *image = button_colors.play_hover.clone().into();
            }
            Interaction::None => {
                *image = button_colors.play.clone().into();
            }
        }
    }
}

fn exit(mut commands: Commands, query: Query<Entity, With<MenuUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
