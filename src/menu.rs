use bevy::prelude::*;

use crate::AppState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>()
            .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(enter.system()))
            .add_system_set(SystemSet::on_update(AppState::Menu).with_system(update.system()))
            .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(exit.system()));
    }
}

struct MenuUi;
struct StartGameButton;

struct ButtonMaterials {
    play: Handle<ColorMaterial>,
    play_hover: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        ButtonMaterials {
            play: materials.add(asset_server.load("ui/button.png").into()),
            play_hover: materials.add(asset_server.load("ui/button_outline.png").into()),
        }
    }
}

fn enter(
    mut commands: Commands,
    button_colors: Res<ButtonMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .insert(MenuUi)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(400.0), Val::Px(200.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: button_colors.play.clone(),
                    ..Default::default()
                })
                .insert(StartGameButton);
        });
}

fn update(
    button_colors: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>),
        (Changed<Interaction>, With<StartGameButton>),
    >,
    mut states: ResMut<State<AppState>>,
) {
    for (interaction, mut material) in interaction_query.iter_mut() {
        // let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => states.replace(AppState::InGame).unwrap(),
            Interaction::Hovered => {
                *material = button_colors.play_hover.clone();
            }
            Interaction::None => {
                *material = button_colors.play.clone();
            }
        }
    }
}

fn exit(mut commands: Commands, query: Query<Entity, With<MenuUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
