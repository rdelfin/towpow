use crate::{animation::AnimationId, components::Tower, prefabs, resources};
use amethyst::{
    animation::{
        get_animation_set, AnimationCommand, AnimationControlSet, AnimationSet, EndControl,
    },
    assets::ProgressCounter,
    core::transform::Transform,
    ecs::{Entities, Join, ReadStorage, WriteStorage},
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::{Builder, World, WorldExt},
    renderer::{camera::Camera, sprite::SpriteRender},
    window::ScreenDimensions,
    GameData, SimpleState, SimpleTrans, StateData, StateEvent, Trans,
};
use log::info;

#[derive(Default)]
pub struct Game {
    pub progress_counter: Option<ProgressCounter>,
    pub items_done_last: Option<usize>,
}

impl SimpleState for Game {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { mut world, .. } = data;
        // Crates new progress counter
        self.progress_counter = Some(Default::default());

        let bullet_prefab_set =
            resources::BulletPrefabSet::new(&mut world, self.progress_counter.as_mut().unwrap());
        world.insert(bullet_prefab_set);
        prefabs::load_tower(&mut world, self.progress_counter.as_mut().unwrap());

        // Creates a new camera
        initialise_camera(&mut world);
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }
        }
        Trans::None
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // Checks if we are still loading data

        if let Some(ref progress_counter) = self.progress_counter {
            // Checks progress
            if progress_counter.is_complete() {
                info!("LOADED");
                let StateData { world, .. } = data;

                self.initialise_animations(world);

                // All data loaded
                self.progress_counter = None;
            } else {
                let errors = progress_counter.errors();
                if !errors.is_empty() {
                    println!("ERRORS: {:?}", errors);
                }

                let num_finished = progress_counter.num_finished();
                let print = match self.items_done_last {
                    Some(l) => num_finished != l,
                    None => true,
                };
                if print {
                    self.items_done_last = Some(num_finished);
                    let completion_pct =
                        100. * num_finished as f64 / progress_counter.num_assets() as f64;
                    info!(
                        "{:.2}% DONE ({} failed)",
                        completion_pct,
                        progress_counter.num_failed()
                    );
                }
            }
        }

        Trans::None
    }
}

impl Game {
    fn initialise_animations(&mut self, world: &mut World) {
        world.exec(
            |(entities, animation_sets, towers, mut control_sets): (
                Entities,
                ReadStorage<AnimationSet<AnimationId, SpriteRender>>,
                ReadStorage<Tower>,
                WriteStorage<AnimationControlSet<AnimationId, SpriteRender>>,
            )| {
                // For each entity that has AnimationSet
                for (entity, animation_set, _) in (&entities, &animation_sets, &towers).join() {
                    // Creates a new AnimationControlSet for the entity
                    let control_set = get_animation_set(&mut control_sets, entity).unwrap();

                    control_set.add_animation(
                        AnimationId::TowerLeft,
                        &animation_set.get(&AnimationId::TowerLeft).unwrap(),
                        EndControl::Stay,
                        1.0,
                        AnimationCommand::Start,
                    );
                    control_set.add_animation(
                        AnimationId::TowerUp,
                        &animation_set.get(&AnimationId::TowerUp).unwrap(),
                        EndControl::Stay,
                        1.0,
                        AnimationCommand::Init,
                    );
                    control_set.add_animation(
                        AnimationId::TowerDown,
                        &animation_set.get(&AnimationId::TowerDown).unwrap(),
                        EndControl::Stay,
                        1.0,
                        AnimationCommand::Init,
                    );
                    control_set.add_animation(
                        AnimationId::TowerRight,
                        &animation_set.get(&AnimationId::TowerRight).unwrap(),
                        EndControl::Stay,
                        1.0,
                        AnimationCommand::Init,
                    );
                }
            },
        );
    }
}

fn initialise_camera(world: &mut World) {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };

    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(1.0);

    world
        .create_entity()
        .with(camera_transform)
        .with(Camera::standard_2d(width / 4., height / 4.))
        .build();
}
